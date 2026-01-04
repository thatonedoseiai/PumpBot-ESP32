// #![no_std]   // ESP‑32 builds are no‑std; remove if you use std
#![no_main]  // If you build a full application

use esp_idf_hal::gpio::{PinDriver, AnyIOPin, Input, Pull};
// use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::task::queue::Queue;
// use esp_idf_hal::task::Task;
use esp_idf_hal::sys::EspError;
use esp_idf_hal::timer::{TimerDriver, TimerConfig, TIMER00};
use std::thread;
use std::boxed::Box;
use std::vec::Vec;
use std::sync::Arc;
use std::fmt;

// ---------------------------------------------------------------------------
// Configuration – feel free to change / make these const generics
// ---------------------------------------------------------------------------
const LONG_PRESS_DURATION_MS: u64 = 2000;          // default: 2 s
const LONG_PRESS_REPEAT_MS:   u64 = 50;            // default: 50 ms
const QUEUE_SIZE:            usize = 4;            // default: 4 entries
const TASK_STACK_SIZE:       usize = 3072;          // default stack size

// ---------------------------------------------------------------------------
// Public API – event types
// ---------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ButtonEventKind {
    Down = 1,
    Up   = 2,
    Held = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ButtonEvent {
    pub pin:   i32,
    pub event: ButtonEventKind,
}

impl fmt::Display for ButtonEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.event {
            ButtonEventKind::Down => write!(f, "({}, Down)", self.pin),
            ButtonEventKind::Up => write!(f, "({}, Up)", self.pin),
            ButtonEventKind::Held => write!(f, "({}, Held)", self.pin),
        }
    }
}

// ---------------------------------------------------------------------------
// Debounce state
// ---------------------------------------------------------------------------
const MASK: u16 = 0b1111000000111111;

struct Debounce<'a> {
    inverted:   bool,
    history:    u16,
    down_time:  u64,
    next_long_time: u64,
    // The pin is kept as a trait object so we can store any GPIO pin
    pin: Box<PinDriver<'a, AnyIOPin, Input>>,
}

impl Debounce<'_> {
    fn update_button(&mut self) {
        // level: 1 = high, 0 = low
        let level = self.pin.is_high();
        self.history = (self.history << 1) | if level { 1 } else { 0 };
    }

    fn button_rose(&mut self) -> bool {
        if (self.history & MASK) == 0b0000000000111111 {
            self.history = 0xffff;
            true
        } else {
            false
        }
    }

    fn button_fell(&mut self) -> bool {
        if (self.history & MASK) == 0b1111000000000000 {
            self.history = 0x0000;
            true
        } else {
            false
        }
    }

    fn button_down(&mut self) -> bool {
        if self.inverted { self.button_fell() } else { self.button_rose() }
    }

    fn button_up(&mut self) -> bool {
        if self.inverted { self.button_rose() } else { self.button_fell() }
    }
}

// ---------------------------------------------------------------------------
// Helper – create an input pin from a pin number
// ---------------------------------------------------------------------------
/// Creates a configured input pin from a pin number.
/// The function uses a large `match` because the HAL uses type‑level pins
/// – the same code pattern is repeated for all 40 GPIOs on the ESP‑32.
// fn create_input_pin(
//     gpio:     &mut Gpio,
//     pin_no:   u8,
//     pull_mode: PullMode,
// ) -> Result<Box<PinDriver<AnyIOPin, Input>>, EspError> {
//     use PullMode as HalPull;

//     let pin_obj = PinDriver::input(gpio, )

//     Ok(pin_obj)
// }

// ---------------------------------------------------------------------------
// Public entry points – mirroring button_init / pulled_button_init
// ---------------------------------------------------------------------------

/// Initialise a set of buttons.  All pins default to an active‑low pull‑up.
/// `pin_select` is a 64‑bit mask – e.g. `pin_bit(0) | pin_bit(4)`
/// Returns a FreeRTOS queue that will receive `ButtonEvent`s.
pub fn button_init(pin_select: Vec<AnyIOPin>, timerg: TIMER00) -> Result<Arc<Queue<ButtonEvent>>, EspError> {
    pulled_button_init(pin_select, timerg)
}

/// Initialise a set of buttons with a user‑supplied pull‑mode.
pub fn pulled_button_init(
    pin_select: Vec<AnyIOPin>,
    timerg: TIMER00,
) -> Result<Arc<Queue<ButtonEvent>>, EspError> {
    // 1️⃣  Grab the peripherals
    // let peripherals = Peripherals::take()
    //     .or_else(|_| Err(EspError::from_non_zero(NonZero::new(0x103))))?;
    // let mut gpio = peripherals.gpio;

    // 2️⃣  Count the pins (used only for pre‑allocating the vector)
    // let pin_count = pin_select.count_ones() as usize;

    // 3️⃣  Build the debounce list
    let mut debounces: Vec<Debounce> = Vec::with_capacity(pin_select.len());
    for pin in pin_select {
        // pin.set_pull(Pull::Down)?;
        let mut pin_obj = PinDriver::input(pin)?;
        pin_obj.set_pull(Pull::Up)?;
        debounces.push(Debounce {
            inverted:      true,              // active‑low buttons
            history:       0xffff,            // start with all bits set
            down_time:     0,
            next_long_time:0,
            pin:           Box::new(pin_obj),
        });
    }

    // 4️⃣  Create the event queue
    let queue: Arc<Queue<ButtonEvent>> = Arc::new(Queue::new(QUEUE_SIZE));
    let queue_task = queue.clone();            // clone for the task

    // 5️⃣  Create a timer for millis() – internally uses esp_timer
    // let timer = Timer::new();

    // 6️⃣  Spawn the background task
    // Task::new()
    //     .stack_size(TASK_STACK_SIZE)
    //     .priority(10)                // priority 10 = normal
    let _ = thread::spawn(move || {
        let timer = TimerDriver::new(timerg, &TimerConfig::new()).unwrap();
        loop {
            let time = timer.counter().unwrap();
            let now_ms = (1000 * time) / timer.tick_hz();

            for d in debounces.iter_mut() {
                d.update_button();

                if d.button_up() {
                    d.down_time = 0;
                    let ev = ButtonEvent { pin: d.pin.pin(), event: ButtonEventKind::Up };
                    let _ = queue_task.send_back(ev, 10);
                } else if d.down_time != 0 && now_ms >= d.next_long_time {
                    let ev = ButtonEvent { pin: d.pin.pin(), event: ButtonEventKind::Held };
                    let _ = queue_task.send_back(ev, 10);
                    d.next_long_time += LONG_PRESS_REPEAT_MS;
                } else if d.button_down() && d.down_time == 0 {
                    d.down_time = now_ms;
                    d.next_long_time = now_ms + LONG_PRESS_DURATION_MS;
                    let ev = ButtonEvent { pin: d.pin.pin(), event: ButtonEventKind::Down };
                    let _ = queue_task.send_back(ev, 10);
                }
            }

            // 10 ms delay – same as the original 10 / portTICK_PERIOD_MS
            esp_idf_hal::delay::FreeRtos::delay_ms(10);
        }
    });   // propagate any spawn error

    // 7  Return the queue to the caller
    Ok(queue)
}
