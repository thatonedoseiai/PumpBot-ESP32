use esp_idf_hal::ledc::*;
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::{AnyIOPin};
use ilidriver::RGB;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

// --- Types and Configuration ---


#[derive(Clone, Debug)]
pub enum LedMode {
    Solid(RGB),
    Fade(RGB, RGB),
    Rainbow,
    Off,
}

#[derive(Clone, Debug)]
struct ControllerState {
    mode: LedMode,
    brightness: f32, // 0.0 to 1.0
    speed_ms: u64,
}

// --- The API Handler ---

pub struct LedController {
    state: Arc<RwLock<ControllerState>>,
}

pub struct LedPeripherals {
    led_r: AnyIOPin,
    led_g: AnyIOPin,
    led_b: AnyIOPin,
    channel_r: CHANNEL0,
    channel_g: CHANNEL1,
    channel_b: CHANNEL2,
    timer: TIMER0
}

impl LedController {
    pub fn new(
        p: LedPeripherals,
        initial_mode: LedMode,
    ) -> Self {
        let state = Arc::new(RwLock::new(ControllerState {
            mode: initial_mode,
            brightness: 1.0,
            speed_ms: 10,
        }));

        let thread_state = state.clone();

        // Spawn the worker thread
        thread::spawn(move || {
            // let peripherals = Peripherals::take().unwrap();
            let config = TimerConfig::new().frequency(5.kHz().into());
            let timerdriver = LedcTimerDriver::new(p.timer, &config).unwrap();

            // Initialize Channels
            let mut ch_r: LedcDriver = LedcDriver::new(p.channel_r, &timerdriver, p.led_r).unwrap();
            let mut ch_g: LedcDriver = LedcDriver::new(p.channel_g, &timerdriver, p.led_g).unwrap();
            let mut ch_b: LedcDriver = LedcDriver::new(p.channel_b, &timerdriver, p.led_b).unwrap();

            let max_duty = ch_r.get_max_duty();
            let mut tick: f32 = 0.0;

            loop {
                let current = { thread_state.read().unwrap().clone() };
                
                let (r, g, b) = match current.mode {
                    LedMode::Off => (0, 0, 0),
                    LedMode::Solid(color) => (color.r, color.g, color.b),
                    LedMode::Fade(c1, c2) => {
                        let t = (tick.sin() + 1.0) / 2.0; // Oscillate 0 to 1
                        (
                            lerp(c1.r, c2.r, t),
                            lerp(c1.g, c2.g, t),
                            lerp(c1.b, c2.b, t),
                        )
                    }
                    LedMode::Rainbow => {
                        hsv_to_rgb(tick % 360.0, 1.0, 1.0)
                    }
                };

                // Apply brightness and set duty
                let apply = |val: u8| -> u32 {
                    ((val as f32 * current.brightness / 255.0) * max_duty as f32) as u32
                };

                ch_r.set_duty(apply(r)).unwrap();
                ch_g.set_duty(apply(g)).unwrap();
                ch_b.set_duty(apply(b)).unwrap();

                tick += 2.0; // Increment based on speed
                thread::sleep(Duration::from_millis(current.speed_ms));
            }
        });

        Self { state }
    }

    pub fn update_mode(&self, mode: LedMode) {
        let mut s = self.state.write().unwrap();
        s.mode = mode;
    }

    pub fn set_brightness(&self, brightness: f32) {
        let mut s = self.state.write().unwrap();
        s.brightness = brightness.clamp(0.0, 1.0);
    }

    pub fn set_speed(&self, speed_ms: u64) {
        let mut s = self.state.write().unwrap();
        s.speed_ms = speed_ms;
    }
}

// --- Utilities ---

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

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