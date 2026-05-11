//! This crate handles the LED controls. It contains but does not export color-related utilities.

use esp_idf_hal::ledc::*;
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::{AnyIOPin};
use fontfile::RGB;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use embedded_hal::pwm::SetDutyCycle;
use log::info;

// --- Types and Configuration ---

/// Represents an operating mode of the LED cycling engine.
#[derive(Clone, Debug)]
pub enum LedMode {
    Solid(RGB),
    Fade(RGB, RGB),
    Rainbow,
    Off,
}

/// Represents the state of all the LEDs. `speed_ms` is the delay (in ms) between updates of the
/// LED colour/brightness, i.e. the period between LED updates.
#[derive(Clone, Debug)]
struct ControllerState {
    mode: LedMode,
    brightness: u8, // 0.0 to 1.0
    speed_ms: u64,
}

// --- The API Handler ---

/// The controller. There is only one LED controller instance for the entire OS.
pub struct LedController {
    state: Arc<RwLock<ControllerState>>,
}

/// A collection of all the peripherals needed to run the LED driver.
pub struct LedPeripherals {
    led_r: AnyIOPin,
    led_g: AnyIOPin,
    led_b: AnyIOPin,
    channel_r: CHANNEL0,
    channel_g: CHANNEL1,
    channel_b: CHANNEL2,
    timer: TIMER0
}

impl LedPeripherals {
    /// Instantiates the collection of peripherals needed from the instances provided.
    pub fn new(
        led_r: AnyIOPin,
        led_g: AnyIOPin,
        led_b: AnyIOPin,
        channel_r: CHANNEL0,
        channel_g: CHANNEL1,
        channel_b: CHANNEL2,
        timer: TIMER0
    ) -> Self {
        LedPeripherals {
            led_r, led_g, led_b,
            channel_r, channel_g, channel_b,
            timer
        }
    }
}

impl LedController {
    /// Instantiates a new controller connected to the peripherals `p`, running in mode
    /// `initial_mode`. This method will also start an independent thread that runs the LEDs
    /// independently.
    pub fn new(
        p: LedPeripherals,
        initial_mode: LedMode,
    ) -> Self {
        let state = Arc::new(RwLock::new(ControllerState {
            mode: initial_mode,
            brightness: 255,
            speed_ms: 10,
        }));

        let thread_state = state.clone();

        // Spawn the worker thread
        thread::spawn(move || {
            // let peripherals = Peripherals::take().unwrap();
            let config = TimerConfig::new().resolution(Resolution::Bits14);//.frequency(5.kHz().into());
            let timerdriver = LedcTimerDriver::new(p.timer, &config).unwrap();

            // Initialize Channels
            let mut ch_r: LedcDriver = LedcDriver::new(p.channel_r, &timerdriver, p.led_r).unwrap();
            let mut ch_g: LedcDriver = LedcDriver::new(p.channel_g, &timerdriver, p.led_g).unwrap();
            let mut ch_b: LedcDriver = LedcDriver::new(p.channel_b, &timerdriver, p.led_b).unwrap();

            let max_duty = ch_r.get_max_duty();
            let mut tick: u32 = 0;
            let mut goingup = true;

            loop {
                let current = { thread_state.read().unwrap().clone() };
                
                let (r, g, b) = match current.mode {
                    LedMode::Off => (0, 0, 0),
                    LedMode::Solid(color) => (color.r, color.g, color.b),
                    // LedMode::Solid(color) => (255, 0, 0),
                    LedMode::Fade(c1, c2) => {
                        if tick == 0 {
                            goingup = true
                        }
                        if tick == 255 {
                            goingup = false;
                        }
                        if goingup {
                            tick += 1;
                        } else {
                            tick -= 1;
                        }
                        (
                            lerp(c1.r, c2.r, tick as u8),
                            lerp(c1.g, c2.g, tick as u8),
                            lerp(c1.b, c2.b, tick as u8),
                        )
                    }
                    LedMode::Rainbow => {
                        tick = (tick + 1) % 360;
                        hsv_to_rgb((tick % 360) as f32, 1.0, 1.0)
                    }
                };

                // Apply brightness and set duty
                let apply = |val: u8| -> u32 {
                    (val as u32 * current.brightness as u32 * max_duty) / 65025
                };

                // info!("led: ({} {} {}), max: {}", apply(r), apply(g), apply(b), max_duty);

                ch_r.set_duty(apply(r)).unwrap();
                ch_g.set_duty(apply(g)).unwrap();
                ch_b.set_duty(apply(b)).unwrap();

                // tick += 2.0; // Increment based on speed
                // thread::sleep(Duration::from_millis(current.speed_ms));
                esp_idf_hal::delay::FreeRtos::delay_ms(current.speed_ms as u32);
            }
        });

        Self { state }
    }

    /// Sets the current operating mode of the LED driver to `mode`.
    pub fn update_mode(&self, mode: LedMode) {
        let mut s = self.state.write().unwrap();
        s.mode = mode;
    }

    /// Sets the current brightness of the LEDs to `brightness`. 0 < `brightness` < 255
    pub fn set_brightness(&self, brightness: u8) {
        let mut s = self.state.write().unwrap();
        s.brightness = brightness;
    }

    /// Sets the period between updates to `speed_ms` milliseconds.
    pub fn set_speed(&self, speed_ms: u64) {
        let mut s = self.state.write().unwrap();
        s.speed_ms = speed_ms;
    }
}

// --- Utilities ---

// (a as f32 + (b as f32 - a as f32) * t) as u8
/// linearly interpolates between `a` and `b` at a percentage `t/255`.
fn lerp(a: u8, b: u8, t: u8) -> u8 {
    let diff: u16 = if a > b { a - b } else { b - a } as u16;
    (a as u16 + (diff * t as u16) / 255) as u8
}

/// Converts HSV to RGB coloration.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    // Standard HSV to RGB conversion logic...
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 { (c, x, 0.0) }
    else if h < 120.0 { (x, c, 0.0) }
    else if h < 180.0 { (0.0, c, x) }
    else if h < 240.0 { (0.0, x, c) }
    else if h < 300.0 { (x, 0.0, c) }
    else { (c, 0.0, x) };
    
    (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
}