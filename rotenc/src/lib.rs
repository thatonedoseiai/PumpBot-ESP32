//! # Rotary‑Encoder driver – Rust port
//!
//! * Uses `esp-idf-hal`/`esp-idf-sys` for GPIO and FreeRTOS primitives
//! * Data‑driven state machine – two constant tables (`TTABLE_HALF` / `TTABLE_FULL`)
//! * ISR implemented as a `extern "C"` function – the driver keeps a static pointer to the
//!   `RotaryEncoderInfo` instance so the ISR can access it
//! * Queue is a FreeRTOS queue of length 1 – the last event overwrites the previous one
//! * Minimal `unsafe`: only the ISR and the few FFI calls to the ESP‑IDF C API
//!
//! ## Usage
//! ```no_run
//! use esp_idf_hal::peripherals::Peripherals;
//! use esp_idf_hal::gpio::Gpio;
//! use esp_idf_hal::gpio::pin::GpioPin;
//! use esp_idf_hal::gpio::GpioExt;
//! use esp_idf_hal::gpio::Pin;
//! use esp_idf_hal::gpio::PinNumber;
//! use esp_idf_hal::gpio::PullMode;
//! use esp_idf_hal::gpio::InterruptMode;
//! use esp_idf_hal::gpio::GpioInputPin;
//! use esp_idf_sys::{gpio_num_t, GPIO_NUM_NC};
//! use core::mem::MaybeUninit;
//! use core::ptr::NonNull;
//! use core::ffi::c_void;
//! use core::sync::atomic::{AtomicU32, Ordering};
//!
//! // The struct must live at least as long as the ISR runs – e.g. a static or a Box
//! static mut ENCODER_INFO: MaybeUninit<RotaryEncoderInfo> = MaybeUninit::uninit();
//!
//! fn main() -> Result<(), esp_idf_sys::esp_err_t> {
//!     // Initialise the ESP‑IDF runtime (normally done by the framework)
//!     unsafe { esp_idf_sys::esp_err_t::ESP_OK };
//!     // Initialise the encoder
//!     unsafe {
//!         let info = ENCODER_INFO.as_mut_ptr();
//!         rotary_encoder_init(
//!             &mut *info,
//!             GPIO_NUM_14,   // pin A
//!             GPIO_NUM_15,   // pin B
//!             GPIO_NUM_NC,   // no button
//!         )?;
//!         // Create the queue and attach it
//!         let q = rotary_encoder_create_queue()?;
//!         rotary_encoder_set_queue(&mut *info, q)?;
//!     }
//!
//!     // In a real task you would read from the queue:
//!     //   let mut ev = RotaryEncoderEvent::default();
//!     //   xQueueReceive(queue, &mut ev as *mut _ as *mut c_void, portMAX_DELAY);
//!     Ok(())
//! }
//! ```
//!
//! The driver exposes the same public API as the original C component but in idiomatic Rust.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU16, Ordering};
use esp_idf_hal::gpio::{InterruptType, PinDriver, AnyIOPin, Input};
use esp_idf_hal::sys::EspError;
use std::sync::Mutex;
use esp_idf_hal::task::queue::Queue;

use esp_idf_hal::sys as sys; //ERROR: no external crate `esp_idf_sys`

/// Number of entries in the FreeRTOS queue – the original component uses a single‑item
/// queue to always keep the latest event.
const EVENT_QUEUE_LENGTH: usize = 1;

/// Size of the state transition tables – 7 rows × 4 columns
const TABLE_ROWS: usize = 7;
const TABLE_COLS: usize = 4;

/// Event flags returned by the state machine (encoded in bits 4‑5)
const DIR_NONE: u8 = 0x0;
const DIR_CW: u8 = 0x10;
const DIR_CCW: u8 = 0x20;

/// Common table state identifiers
const R_START: u8 = 0x0;
const H_CCW_BEGIN: u8 = 0x1;
const H_CW_BEGIN: u8 = 0x2;
const H_START_M: u8 = 0x3;
const H_CW_BEGIN_M: u8 = 0x4;
const H_CCW_BEGIN_M: u8 = 0x5;

/// Full‑step table identifiers
const F_CW_FINAL: u8 = 0x1;
const F_CW_BEGIN: u8 = 0x2;
const F_CW_NEXT: u8 = 0x3;
const F_CCW_BEGIN: u8 = 0x4;
const F_CCW_FINAL: u8 = 0x5;
const F_CCW_NEXT: u8 = 0x6;

/// Speed‑based multiplier threshold – matches the original driver
const ROTENC_THRESHOLD: u16 = 32;

/// **Half‑step state machine** – each entry is `state & 0x0F` → next state, `0x30` → event
static TTABLE_HALF: [[u8; TABLE_COLS]; TABLE_ROWS] = [  //ERROR: Expected array w/ size 7, found 6
    [H_START_M, H_CW_BEGIN, H_CCW_BEGIN, R_START],              // R_START (00)
    [H_START_M | DIR_CCW, R_START, H_CCW_BEGIN, R_START],       // H_CCW_BEGIN
    [H_START_M | DIR_CW, H_CW_BEGIN, R_START, R_START],         // H_CW_BEGIN
    [H_START_M, H_CCW_BEGIN_M, H_CW_BEGIN_M, R_START],          // H_START_M (11)
    [H_START_M, H_START_M, H_CW_BEGIN_M, R_START | DIR_CW],      // H_CW_BEGIN_M
    [H_START_M, H_CCW_BEGIN_M, H_START_M, R_START | DIR_CCW],    // H_CCW_BEGIN_M
    [0xff, 0xff, 0xff, 0xff],                                   //Padding data to match lengths
];

/// **Full‑step state machine** – only emits on 00
static TTABLE_FULL: [[u8; TABLE_COLS]; TABLE_ROWS] = [
    [R_START, F_CW_BEGIN, F_CCW_BEGIN, R_START],                // R_START
    [F_CW_NEXT, R_START, F_CW_FINAL, R_START | DIR_CW],          // F_CW_FINAL
    [F_CW_NEXT, F_CW_BEGIN, R_START, R_START],                  // F_CW_BEGIN
    [F_CW_NEXT, F_CW_BEGIN, F_CW_FINAL, R_START],               // F_CW_NEXT
    [F_CCW_NEXT, R_START, F_CCW_BEGIN, R_START],                // F_CCW_BEGIN
    [F_CCW_NEXT, F_CCW_FINAL, R_START, R_START | DIR_CCW],       // F_CCW_FINAL
    [F_CCW_NEXT, F_CCW_FINAL, F_CCW_BEGIN, R_START],            // F_CCW_NEXT
];

/// Direction of the last movement – matches the C enum
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RotaryEncoderDirection {
    NotSet = 0,
    Clockwise = 1,
    CounterClockwise = 2,
}

/// Current position / direction / multiplier
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RotaryEncoderState {
    /// Signed position – increases on CW, decreases on CCW
    pub position: i32,
    /// Last direction seen
    pub direction: RotaryEncoderDirection,
    /// Multiplier derived from the time between steps (see original code)
    pub multiplier: u16,
}

/// Queue event – the same layout as the C struct
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RotaryEncoderEvent {
    pub state: RotaryEncoderState,
}

/// Internal driver data – this struct is passed to the ISR via a raw pointer
pub struct RotaryEncoderInfo<'a> {
    /// GPIO numbers for the two quadrature signals
    pub pin_a: PinDriver<'a, AnyIOPin, Input>,
    pub pin_b: PinDriver<'a, AnyIOPin, Input>,
    /// Optional button pin (not used in the current ISR)
    pub pin_btn: PinDriver<'a, AnyIOPin, Input>,
    /// Optional queue – events are written here from the ISR
    pub queue: Option<Queue<RotaryEncoderEvent>>,
    /// Pointer to the active transition table (half‑ or full‑step)
    pub table: &'static [[u8; TABLE_COLS]; TABLE_ROWS],
    /// Current state machine state (4 bits are used)
    pub table_state: u8,
    /// Current position / direction / multiplier
    pub state: RotaryEncoderState,
}

impl RotaryEncoderInfo<'_> {
    const fn new() -> Self {
        todo!();
    }
}

/// Global atomic counter used by the ISR to compute the speed‑based multiplier
static ROTENC_MULTIPLIER_COUNTER: AtomicU16 = AtomicU16::new(0);
static ROTARY_ENCODER_INFO: Mutex<RotaryEncoderInfo> = Mutex::new(RotaryEncoderInfo::new());

/// ---------------------------------------------------------------------------
///  State‑machine logic – pure Rust, no unsafe
/// ---------------------------------------------------------------------------

/// Process the current pin levels and update the internal state machine.
/// Returns the event flag (DIR_CW / DIR_CCW / 0).
#[inline(always)]
fn process(info: &mut RotaryEncoderInfo) -> u8 {
    // Read pin levels – this is safe in ISR context (no heap allocation)
    let a = (info.pin_a.is_high()) as u8;
    let b = (info.pin_b.is_high()) as u8;
    let pin_state = (b << 1) | a;

    // Lookup next state in the active table
    let next = info.table[(info.table_state & 0x0F) as usize][pin_state as usize];
    info.table_state = next;
    // Return event flag (upper two bits)
    next & 0x30
}

/// ---------------------------------------------------------------------------
///  ISR – the only `extern "C"` function
/// ---------------------------------------------------------------------------

/// ISR called on any edge of pin A or pin B.  The `args` pointer is the
/// `&mut RotaryEncoderInfo` passed during `gpio_isr_handler_add`.
fn isr_rotenc() {
    // Cast the raw pointer back to our struct
    // let info = &mut *(args as *mut RotaryEncoderInfo);
    let mut info = ROTARY_ENCODER_INFO.lock().unwrap(); // how to handle the poisoned error?

    // Run the state machine
    let event = process(&mut info);

    let mut send_event = false;
    let delaydelta: u16;

    match event {
        DIR_CW => {
            // Update the atomic counter
            let current = ROTENC_MULTIPLIER_COUNTER.load(Ordering::SeqCst);
            delaydelta = current;
            let new_val = if current + 4 < ROTENC_THRESHOLD {
                current + 4
            } else {
                ROTENC_THRESHOLD
            };
            ROTENC_MULTIPLIER_COUNTER.store(new_val, Ordering::SeqCst);

            // Update driver state
            info.state.position = info.state.position.wrapping_add(1);
            info.state.direction = RotaryEncoderDirection::Clockwise;
            info.state.multiplier = (delaydelta >> 3) + 1;
            send_event = true;
        }
        DIR_CCW => {
            let current = ROTENC_MULTIPLIER_COUNTER.load(Ordering::SeqCst);
            delaydelta = current;
            let new_val = if current + 4 < ROTENC_THRESHOLD {
                current + 4
            } else {
                ROTENC_THRESHOLD
            };
            ROTENC_MULTIPLIER_COUNTER.store(new_val, Ordering::SeqCst);

            info.state.position = info.state.position.wrapping_sub(1);
            info.state.direction = RotaryEncoderDirection::CounterClockwise;
            info.state.multiplier = (delaydelta >> 3) + 1;
            send_event = true;
        }
        _ => {
            let current = ROTENC_MULTIPLIER_COUNTER.load(Ordering::SeqCst);
            delaydelta = current;
        }
    }

    // Queue the event if requested
    if send_event {
        if let Some(q) = &info.queue {
            let ev = RotaryEncoderEvent {
                state: RotaryEncoderState {
                    position: info.state.position,
                    direction: info.state.direction,
                    multiplier: (delaydelta >> 3) + 1,
                },
            };
            q.send_back(ev, 10);
        }
    }
}

/// ---------------------------------------------------------------------------
///  Helper functions – wrappers around ESP‑IDF C API
/// ---------------------------------------------------------------------------

/// Attach the ISR to a GPIO pin
fn attach_isr(pin: &mut PinDriver<AnyIOPin, Input>) -> Result<(), EspError> {
    pin.set_interrupt_type(InterruptType::PosEdge)?;
    unsafe {
        pin.subscribe(isr_rotenc)
    }
}

/// Remove the ISR from a GPIO pin
fn remove_isr(pin: &mut PinDriver<AnyIOPin, Input>) -> Result <(), EspError> { 
    pin.unsubscribe() 
}

/// ---------------------------------------------------------------------------
///  Public API – mirrors the original C functions
/// ---------------------------------------------------------------------------

/// Initialise the encoder – configure the pins, install ISRs and reset state.
///
/// `info` must outlive the ISR (e.g. a static or a `Box`).  The caller can
/// subsequently attach a queue with `rotary_encoder_set_queue`.
pub fn rotary_encoder_init<'a>(
    info: &mut RotaryEncoderInfo<'a>,
    pin_a: PinDriver<'a, AnyIOPin, Input>,
    pin_b: PinDriver<'a, AnyIOPin, Input>,
    pin_btn: PinDriver<'a, AnyIOPin, Input>,
) -> Result<(), EspError> {
    // unsafe {
    //     // GPIO reset & configuration
    //     sys::gpio_reset_pin(pin_a);
    //     sys::gpio_set_pull_mode(pin_a, sys::GPIO_PULLUP_ONLY);
    //     sys::gpio_set_direction(pin_a, sys::GPIO_MODE_INPUT);
    //     sys::gpio_set_intr_type(pin_a, sys::GPIO_INTR_ANYEDGE);

    //     sys::gpio_reset_pin(pin_b);
    //     sys::gpio_set_pull_mode(pin_b, sys::GPIO_PULLUP_ONLY);
    //     sys::gpio_set_direction(pin_b, sys::GPIO_MODE_INPUT);
    //     sys::gpio_set_intr_type(pin_b, sys::GPIO_INTR_ANYEDGE);

    //     // Optional button – currently unused in the ISR
    //     if pin_btn != sys::GPIO_NUM_NC {
    //         sys::gpio_reset_pin(pin_btn);
    //         sys::gpio_set_pull_mode(pin_btn, sys::GPIO_PULLUP_ONLY);
    //         sys::gpio_set_direction(pin_btn, sys::GPIO_MODE_INPUT);
    //         sys::gpio_set_intr_type(pin_btn, sys::GPIO_INTR_ANYEDGE);
    //     }
    // }

    // Store the pin numbers
    info.pin_a = pin_a;
    info.pin_b = pin_b;
    info.pin_btn = pin_btn; // can remove

    // Default to full‑step mode
    info.table = &TTABLE_FULL;
    info.table_state = R_START;

    // Reset position/direction
    info.state = RotaryEncoderState {
        position: 0,
        direction: RotaryEncoderDirection::NotSet,
        multiplier: 0,
    };
    info.queue = None;

    // Install ISRs for the two quadrature pins
    attach_isr(&mut info.pin_a)?;
    attach_isr(&mut info.pin_b)?;

    // Reset the speed counter
    ROTENC_MULTIPLIER_COUNTER.store(0, Ordering::SeqCst);
    Ok(())
}

/// Enable or disable half‑step mode (double the number of counts per revolution).
pub fn rotary_encoder_enable_half_steps(
    info: &mut RotaryEncoderInfo,
    enable: bool,
) -> Result<(), sys::esp_err_t> {
    info.table = if enable { &TTABLE_HALF } else { &TTABLE_FULL };
    info.table_state = R_START; // reset state machine
    Ok(())
}

/// Swap the sense of A and B – useful if the mechanical direction is opposite.
// pub fn rotary_encoder_flip_direction(
//     info: &mut RotaryEncoderInfo,
// ) -> Result<(), sys::esp_err_t> {
//     let tmp = info.pin_a;
//     info.pin_a = info.pin_b;
//     info.pin_b = tmp;
//     Ok(())
// }

/// Remove the ISRs installed by `rotary_encoder_init`.  The GPIOs stay configured.
pub fn rotary_encoder_uninit(info: &mut RotaryEncoderInfo) -> Result<(), EspError> {
    remove_isr(&mut info.pin_a)?;
    remove_isr(&mut info.pin_b)?;
    Ok(())
}

// /// Create a FreeRTOS queue for the encoder events.  The caller must store the handle.
// pub fn rotary_encoder_create_queue() -> Result<Queue, sys::esp_err_t> {

//     unsafe {
//         let q = sys::xQueueCreate(
//             EVENT_QUEUE_LENGTH as u32,
//             mem::size_of::<RotaryEncoderEvent>() as u32,
//         );
//         if !q.is_null() {
//             Ok(q)
//         } else {
//             Err(sys::ESP_FAIL)
//         }
//     }
// }

/// Attach a queue to an encoder instance – the ISR will overwrite the single entry.
pub fn rotary_encoder_set_queue(
    info: &mut RotaryEncoderInfo,
    queue: Queue<RotaryEncoderEvent>,
) -> Result<(), sys::esp_err_t> {
    info.queue = Some(queue);
    Ok(())
}

/// Return a snapshot of the current position / direction.  The caller can
/// read the struct directly if needed – this helper is only for the API.
pub fn rotary_encoder_get_state(
    info: &RotaryEncoderInfo,
) -> Result<RotaryEncoderState, sys::esp_err_t> {
    Ok(info.state)
}

/// Reset the position counter and clear the direction flag.
pub fn rotary_encoder_reset(info: &mut RotaryEncoderInfo) -> Result<(), sys::esp_err_t> {
    info.state.position = 0;
    info.state.direction = RotaryEncoderDirection::NotSet;
    Ok(())
}
