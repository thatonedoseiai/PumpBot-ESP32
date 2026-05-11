//! This crate provides the basic functionality for reading the rotary encoder. A thread is started
//! to listen to the rotary encoder inputs, and events are sent over a queue to be used by any
//! other service. 

use esp_idf_hal::task::queue::Queue;
use esp_idf_hal::gpio::{PinDriver, AnyIOPin, Pull};
// use quadrature_encoder::{RotaryEncoder, RotaryMovement};
use esp_idf_hal::sys::EspError;
use esp_idf_hal::delay::FreeRtos;
use rotary_encoder_embedded::{standard::StandardMode, Direction};
use std::thread;
use std::sync::Arc;
use std::num::Wrapping;
use std::fmt;

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

/// Starts the rotary encoder listener. `pin_a` represents the left-turning pin, and `pin_b`
/// represents the right-turning pin. Any events captured by the listener are sent over `queue`.
pub fn start_rotenc_thread<T: From<EncoderEvent> + Send + Sync + Copy + 'static>(
    queue: Arc<Queue<T>>,
    pin_a: AnyIOPin,
    pin_b: AnyIOPin,
) -> Result<(), EspError> {
    let q_task = queue.clone();
    let mut pin_a_driver = PinDriver::input(pin_a)?;
    let mut pin_b_driver = PinDriver::input(pin_b)?;
    pin_a_driver.set_pull(Pull::Down)?;
    pin_b_driver.set_pull(Pull::Down)?;

    let _ = thread::spawn(move || {
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
                let _ = q_task.send_back(EncoderEvent::new(position, dir).into(), 10);
            }
            FreeRtos::delay_ms(10);
        }
    });
    
    Ok(())
}
