//! This crate provides the basic functionality for reading the rotary encoder. A thread is started
//! to listen to the rotary encoder inputs, and events are sent over a queue to be used by any
//! other service. 
// use quadrature_encoder::{RotaryEncoder, RotaryMovement};

#![no_std]
#![no_main]

extern crate alloc;

// use esp_idf_hal::task::queue::Queue;
// use esp_idf_hal::gpio::{PinDriver, AnyIOPin, Pull};
// use esp_idf_hal::sys::EspError;
// use esp_idf_hal::delay::FreeRtos;
// use std::thread;
// use std::sync::Arc;
// use std::num::Wrapping;
// use std::fmt;
pub use rotary_encoder_embedded::{standard::StandardMode, Direction};
use quadrature_encoder::{RotaryEncoder, RotaryMovement};
use esp_hal::gpio::{AnyPin, Input, InputConfig, Pull};
use esp_hal::pcnt::{Pcnt, unit::{Unit, InvalidFilterThreshold}, channel::{CtrlMode, EdgeMode}};
use esp_hal::peripherals::PCNT;
// use esp_hal::delay::Delay;
use embassy_sync::channel::{Channel, Receiver};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_executor::Spawner;
use core::num::Wrapping;
use core::fmt;
use embassy_time::{Timer, Duration};
use alloc::sync::Arc;
use critical_section::Mutex;
use core::cell::RefCell;
use esp_hal::time::Instant;

/// Represents some rotation that happened with the rotary encoder.
#[derive(Clone, Copy, Debug)]
pub struct EncoderEvent {
    pub pos: Wrapping<i32>,
    pub dir: Direction,
}

impl EncoderEvent {
    const fn new(pos: Wrapping<i32>, dir: Direction) -> Self {
        EncoderEvent {pos, dir}
    }
}

impl fmt::Display for EncoderEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dir {
            Direction::Clockwise => write!(f, "(pos: {}, CW)", self.pos),
            Direction::Anticlockwise => write!(f, "(pos: {}, CCW)", self.pos),
            Direction::None => write!(f, "(pos: {}, N/A)", self.pos),
        }
    }
}

pub struct RotencDriver {
    prev_event: RefCell<Instant>,
}

impl RotencDriver {
    pub async fn receive(&self, accel_sensitivity: u8) -> i16 {
        EVENT_SIGNAL.wait().await;
        // let mut d = 0;
        // while d == 0 {
        //     d = self.get_delta();
        //     log::warn!("ROTENC: poll delta {}", d);
        //     Timer::after(Duration::from_millis(1000)).await;
        // }
        let d = self.get_delta();
        critical_section::with(|cs| {
            let u = UNIT0.borrow_ref(cs);
        });
        EVENT_SIGNAL.reset();
        let delta_time = self.prev_event.borrow().elapsed().as_millis();
        let accel_multiplier = if accel_sensitivity == 0 { 1 } else {
            ((120 - delta_time.min(120)).saturating_pow(2) * accel_sensitivity as u64 / 1000).clamp(1, 100) as i16
        };
        self.prev_event.replace(Instant::now());
        d.saturating_mul(accel_multiplier)
    }

    pub fn get_delta(&self) -> i16 {
        critical_section::with(|cs| {
            let mut u0 = UNIT0.borrow_ref(cs);
            if let Some(u) = u0.as_ref() {
                let k = u.value();
                u.clear();
                k
            } else {
                unreachable!("unit0 should already be initialized")
            }
        })
    }
}

// static EVENT_QUEUE: Channel<CriticalSectionRawMutex, EncoderEvent, 20> = Channel::new();
static EVENT_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();

// /// the rotary encoder thread
// #[embassy_executor::task]
// async fn rotenc_thread(pin_a: AnyPin<'static>, pin_b: AnyPin<'static>, p: PCNT<'static>, event_signal: Arc<Signal<CriticalSectionRawMutex, bool>>) {
//     let inputconfig = InputConfig::default().with_pull(Pull::Up);
//     let pin_a_driver = Input::new(pin_a, inputconfig);
//     let pin_b_driver = Input::new(pin_b, inputconfig);
//     let pcnt = Pcnt::new(p);
//     let u0 = pcnt.unit0;
//     u0.set_filter(Some(800)).expect("[error]: unable to set filter value in rotenc thread!");
//     let input_a = pin_a_driver.peripheral_input();
//     let input_b = pin_b_driver.peripheral_input();
//     let ch0 = &u0.channel0;
//     ch0.set_ctrl_signal(input_a.clone());
//     ch0.set_edge_signal(input_b.clone());
//     ch0.set_ctrl_mode(CtrlMode::Keep, CtrlMode::Reverse);
//     ch0.set_input_mode(EdgeMode::Increment, EdgeMode::Hold);

//     u0.clear();
//     u0.resume();
// }

static UNIT0: Mutex<RefCell<Option<Unit<'static, 0>>>> = Mutex::new(RefCell::new(None));

/// Starts the rotary encoder listener. `pin_a` represents the left-turning pin, and `pin_b`
/// represents the right-turning pin. Any events captured by the listener are sent over `queue`.
pub fn start_rotenc_thread(
    // queue: Arc<Queue<T>>,
    // spawner: Spawner,
    pin_a: AnyPin<'static>,
    pin_b: AnyPin<'static>,
    p: PCNT<'static>,
// ) -> Receiver<'static, CriticalSectionRawMutex, EncoderEvent, 20> {
) -> Result<RotencDriver, InvalidFilterThreshold> {

    // let e = Arc::new(Signal::new());
    // let _ = spawner.spawn(rotenc_thread(pin_a, pin_b, pcnt, e.clone()).unwrap());

    let inputconfig = InputConfig::default().with_pull(Pull::Up);
    let pin_a_driver = Input::new(pin_a, inputconfig);
    let pin_b_driver = Input::new(pin_b, inputconfig);
    let mut pcnt = Pcnt::new(p);
    pcnt.set_interrupt_handler(interrupt_handler);
    let u0 = pcnt.unit0;
    u0.set_filter(Some(800))?;
    let input_a = pin_a_driver.peripheral_input();
    let input_b = pin_b_driver.peripheral_input();
    let ch0 = &u0.channel0;
    ch0.set_ctrl_signal(input_a.clone());
    ch0.set_edge_signal(input_b.clone());
    ch0.set_ctrl_mode(CtrlMode::Keep, CtrlMode::Reverse);
    ch0.set_input_mode(EdgeMode::Increment, EdgeMode::Hold);
    u0.set_threshold0(Some(1));
    u0.set_threshold1(Some(-1));
    u0.clear();
    u0.listen();
    u0.resume();

    critical_section::with(|cs| UNIT0.borrow_ref_mut(cs).replace(u0));

    Ok(RotencDriver {
        prev_event: RefCell::new(Instant::now()),
        // rotenc_unit: u0,
    })

    // EVENT_QUEUE.receiver()
}

use core::borrow::Borrow;
#[esp_hal::handler]
fn interrupt_handler() {
    EVENT_SIGNAL.signal(true);
    critical_section::with(|cs| {
        let mut u0 = UNIT0.borrow_ref(cs);
        if let Some(u) = u0.as_ref() {
            u.reset_interrupt();
        }
    });
}
