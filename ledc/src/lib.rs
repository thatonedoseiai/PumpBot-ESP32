//! This crate handles the LED controls. It contains but does not export color-related utilities.

#![no_std]
#![no_main]

extern crate alloc;

use esp_hal::gpio::{AnyPin, Output, OutputConfig, Level, DriveMode};
use esp_hal::ledc::{timer::Timer, LowSpeed, channel::{Error, Channel, Number, ChannelIFace, config::Config}, Ledc};
use global_settings::rgb::RGB;
use alloc::sync::Arc;
use embedded_hal::pwm::SetDutyCycle;
use embassy_executor::Spawner;
use embassy_sync::once_lock::OnceLock;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Timer as ETimer, Duration as EDuration};

// --- Types and Configuration ---

type RwLock<T> = embassy_sync::rwlock::RwLock<CriticalSectionRawMutex, T>;

/// Represents an operating mode of the LED cycling engine.
#[derive(Clone, Copy, Debug)]
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
pub struct LedPeripherals<'a> {
    channel_r: Channel<'a, LowSpeed>,
    channel_g: Channel<'a, LowSpeed>,
    channel_b: Channel<'a, LowSpeed>,
}

impl LedPeripherals<'_> {
    /// Instantiates the collection of peripherals needed from the instances provided.
    pub fn new<'a>(
        led_r: AnyPin<'a>,
        led_g: AnyPin<'a>,
        led_b: AnyPin<'a>,
        ledc_driver: &Ledc<'a>,
        timer: &'a Timer<'a, LowSpeed>
    ) -> Result<LedPeripherals<'a>, Error> {
        let config = OutputConfig::default();
        let mut channel_r = ledc_driver.channel(Number::Channel0, Output::new(led_r, Level::Low, config));
        let mut channel_g = ledc_driver.channel(Number::Channel1, Output::new(led_g, Level::Low, config));
        let mut channel_b = ledc_driver.channel(Number::Channel2, Output::new(led_b, Level::Low, config));
        let channel_config = Config {
            timer: timer,
            duty_pct: 10,
            drive_mode: DriveMode::PushPull,
        };
        channel_r.configure(channel_config)?;
        channel_g.configure(channel_config)?;
        channel_b.configure(channel_config)?;

        Ok(LedPeripherals {
            channel_r, channel_g, channel_b,
        })
    }
}

static STATE: OnceLock<Arc<RwLock<ControllerState>>> = OnceLock::new();

#[embassy_executor::task]
async fn ledc_worker(mut p: LedPeripherals<'static>) {
    let max_duty = p.channel_r.max_duty_cycle();
    let mut tick: u32 = 0;
    let mut goingup = true;

    loop {
        let current = STATE.get().await.clone();
        let mode = {current.read().await.mode};

        let (r, g, b) = match mode {
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
        let apply = |val: u8, bright: u8| -> u16 {
            ((val as u32 * bright as u32 * max_duty as u32) / 65025) as u16
        };

        {
            let brightness = {current.read().await.brightness};
            p.channel_r.set_duty_cycle(apply(r, brightness)).unwrap();
            p.channel_g.set_duty_cycle(apply(g, brightness)).unwrap();
            p.channel_b.set_duty_cycle(apply(b, brightness)).unwrap();
        }
        let speed = {current.read().await.speed_ms};

        ETimer::after(EDuration::from_millis(speed)).await;
    }
}

impl LedController {
    /// Instantiates a new controller connected to the peripherals `p`, running in mode
    /// `initial_mode`. This method will also start an independent thread that runs the LEDs
    /// independently.
    pub fn new(
        spawner: Spawner,
        p: LedPeripherals<'static>,
        initial_mode: LedMode,
    ) -> Self {
        let inner_arc = Arc::new(RwLock::new(ControllerState {
            mode: initial_mode,
            brightness: 255,
            speed_ms: 10,
        }));
        STATE.init(inner_arc.clone()).unwrap();

        // Spawn the worker thread
        // thread::spawn(move || { });
        let _ = spawner.spawn(ledc_worker(p).unwrap());

        Self { state: inner_arc }
    }

    // TODO: add the errors or make it async
    /// Sets the current operating mode of the LED driver to `mode`.
    pub async fn set_mode(&self, mode: LedMode) {
        let mut s = self.state.write().await;
        s.mode = mode;
    }

    /// Sets the current brightness of the LEDs to `brightness`. 0 < `brightness` < 255
    pub async fn set_brightness(&self, brightness: u8) {
        // log::info!("LED STATE LOCKED");
        let mut s = self.state.write().await;
        s.brightness = brightness;
        // log::info!("LED STATE UNLOCKED");
    }

    /// Sets the period between updates to `speed_ms` milliseconds.
    pub async fn set_speed(&self, speed_ms: u64) {
        // log::info!("LED STATE LOCKED");
        let mut s = self.state.write().await;
        s.speed_ms = speed_ms;
        // log::info!("LED STATE UNLOCKED");
    }

    pub async fn get_mode(&self) -> LedMode {
        // log::info!("LED STATE LOCKED");
        let k = self.state.read().await.mode;
        // log::info!("LED STATE UNLOCKED");
        k
    }
}

// --- Utilities ---

// (a as f32 + (b as f32 - a as f32) * t) as u8
/// linearly interpolates between `a` and `b` at a percentage `t/255`.
fn lerp(a: u8, b: u8, t: u8) -> u8 {
    let a = a as u16;
    let b = b as u16;
    let t = t as u16;

    let result = (a * (255 - t) + b * t + 127) / 255;
    result as u8
    // let diff: u16 = if a > b { a - b } else { b - a } as u16;
    // (a as u16 + (diff * t as u16) / 255) as u8
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