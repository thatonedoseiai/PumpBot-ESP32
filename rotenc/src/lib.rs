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
use rotary_encoder_embedded::{standard::StandardMode, Direction};
use esp_hal::gpio::{AnyPin, Input, InputConfig, Pull};
// use esp_hal::delay::Delay;
use embassy_sync::channel::{Channel, Receiver};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_executor::Spawner;
use core::num::Wrapping;
use core::fmt;
use embassy_time::{Timer, Duration};

/// Represents some rotation that happened with the rotary encoder.
#[derive(Clone, Copy, Debug)]
pub struct EncoderEvent {
    pos: Wrapping<u32>,
    dir: Direction,
}

impl EncoderEvent {
    fn new(pos: Wrapping<u32>, dir: Direction) -> Self {
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

static EVENT_QUEUE: Channel<CriticalSectionRawMutex, EncoderEvent, 10> = Channel::new();

/// the rotary encoder thread
#[embassy_executor::task]
async fn rotenc_thread(pin_a: AnyPin<'static>, pin_b: AnyPin<'static>) {
    let inputconfig = InputConfig::default().with_pull(Pull::Down);
    let pin_a_driver = Input::new(pin_a, inputconfig);
    let pin_b_driver = Input::new(pin_b, inputconfig);
    // let delay = Delay::new();

    let mut encoder = StandardMode::new();
    let mut position = Wrapping(0u32);

    loop {
        let dir = encoder.update(pin_a_driver.is_high(), pin_b_driver.is_high());
        match dir {
            Direction::Clockwise => {position += 1;}
            Direction::Anticlockwise => {position -= 1;}
            _ => {}
        }
        if dir != Direction::None {
            // let _ = q_task.send_back(EncoderEvent::new(position, dir).into(), 10);
            EVENT_QUEUE.send(EncoderEvent::new(position, dir)).await;
        }
        Timer::after(Duration::from_millis(100)).await;
        // delay.delay_millis(10);
        // FreeRtos::delay_ms(10);
    }
}

/// Starts the rotary encoder listener. `pin_a` represents the left-turning pin, and `pin_b`
/// represents the right-turning pin. Any events captured by the listener are sent over `queue`.
pub fn start_rotenc_thread(
    // queue: Arc<Queue<T>>,
    spawner: Spawner,
    pin_a: AnyPin<'static>,
    pin_b: AnyPin<'static>,
) -> Receiver<'static, CriticalSectionRawMutex, EncoderEvent, 10> {

    let _ = spawner.spawn(rotenc_thread(pin_a, pin_b).unwrap());

    EVENT_QUEUE.receiver()
}
