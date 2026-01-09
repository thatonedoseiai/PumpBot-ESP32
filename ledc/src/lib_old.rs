use esp_idf_hal::{
    gpio::AnyIOPin,
    ledc::{TIMER0, CHANNEL0, CHANNEL1, CHANNEL2, LedcDriver, LedcTimerDriver, config::TimerConfig},
    units::FromValueType,
    sys::EspError,
};
use ilidriver::RGB;
use std::sync::{
    // atomic::{AtomicBool, Ordering}, 
    Arc,
    mpsc::{channel, Sender, SendError, Receiver, TryRecvError},
    RwLock,
};
use std::{thread, thread::JoinHandle};
use std::any::Any;
use std::rc::Rc;

#[derive(Debug)]
pub struct Settings {
    pub RGB_mode: u8,
    pub RGB_colour: RGB,
    pub RGB_colour_2: RGB,
    pub RGB_brightness: u16,
    pub RGB_speed: u16,
}

pub struct LedPins {
    // r_dr: LedcDriver<'a>,
    // g_dr: LedcDriver<'a>,
    // b_dr: LedcDriver<'a>,
    r_dr: AnyIOPin,
    g_dr: AnyIOPin,
    b_dr: AnyIOPin,
    c0: CHANNEL0,
    c1: CHANNEL1,
    c2: CHANNEL2,
    timer: TIMER0
}

// impl LedPins<'_> {
//     fn new(r: AnyIOPin, g: AnyIOPin, b: AnyIOPin, c0: CHANNEL0, c1: CHANNEL1, c2: CHANNEL2, timer: TIMER0) -> Result<Self, EspError> {
//         let config = TimerConfig::new().frequency(25_u32.kHz().into());
//         let timer = Rc::new(LedcTimerDriver::new(timer, &config).unwrap());
//         let r_dr = LedcDriver::new(c0, timer.clone(), r)?;
//         let g_dr = LedcDriver::new(c1, timer.clone(), g)?;
//         let b_dr = LedcDriver::new(c2, timer, b)?;

//         Ok(LedPins { r_dr, g_dr, b_dr })
//     }
// }

pub enum LedError {
    ThreadSendError(SendError<LedThreadMessage>),
    ThreadJoinError(Box<dyn Any + Send + 'static>),
}

impl From<SendError<LedThreadMessage>> for LedError {
    fn from(val: SendError<LedThreadMessage>) -> Self {
        LedError::ThreadSendError(val)
    }
}

impl From<Box<dyn Any + Send + 'static>> for LedError {
    fn from(val: Box<dyn Any + Send + 'static>) -> Self {
        LedError::ThreadJoinError(val)
    }
}

enum LedThreadMessage {
    Restart,
    Stop
}

type Result<T, E = LedError> = std::result::Result<T, E>;

trait LedControl {
    type Config;
    fn init(c: Self::Config, l: LedPins) -> Self;
    fn phase_thread(c: Arc<RwLock<Self::Config>>, l: LedPins, r: Receiver<LedThreadMessage>) -> ();
    fn restart(&self) -> Result<()>;
    fn stop(self) -> Result<()>;
}

pub struct FadeConfig { 
    colour_1: RGB,
    colour_2: RGB,
    brightness: u32,
    speed: u16,
}

pub struct Fade { 
    fc: Arc<RwLock<FadeConfig>>,
    handle: JoinHandle<()>,
    threadMsgs: Sender<LedThreadMessage>,
}

impl LedControl for Fade {
    type Config = FadeConfig;

    fn init(config: FadeConfig, pins: LedPins) -> Fade {
        let (threadMsgs, receiver) = channel();
        let fc = Arc::new(RwLock::new(config));

        let fcthread = fc.clone();
        let handle = thread::spawn(move || 
            Fade::phase_thread(fcthread, pins, receiver));

        let fade = Fade {
            fc, 
            threadMsgs,
            handle
        };

        fade
    }

    fn phase_thread(conf: Arc<RwLock<FadeConfig>>, pins: LedPins, msgs: Receiver<LedThreadMessage>) -> () {
        let config = TimerConfig::new().frequency(25_u32.kHz().into());
        let timer = Rc::new(LedcTimerDriver::new(pins.timer, &config).unwrap());
        let rdriver = LedcDriver::new(pins.c0, timer.clone(), pins.r_dr);
        let gdriver = LedcDriver::new(pins.c1, timer.clone(), pins.g_dr);
        let bdriver = LedcDriver::new(pins.c2, timer, pins.b_dr);
        let mut config = conf.read().unwrap(); // TODO error handling
        let mut cycle = 0;
        let mut goingup = false;

        loop {
            let msg = msgs.try_recv();
            match msg {
                Ok(LedThreadMessage::Restart) => { 
                    config = conf.read().unwrap();
                    cycle = 0;
                },
                Ok(LedThreadMessage::Stop) => { break; },
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => { break; },
            };

            let c1arr = [u8;3]::from(config.colour_1);
            let c2arr = [u8;3]::from(config.colour_2);

            for dr in [rdriver, gdriver, bdriver] {
                
            }
        }
    }

    fn restart(&self) -> Result<()> {
        self.threadMsgs.send(LedThreadMessage::Restart)?;
        Ok(())
    }

    fn stop(self) -> Result<()> {
        self.threadMsgs.send(LedThreadMessage::Stop)?;
        let pins = self.handle.join()?;
        Ok(())
    }
}

// static SETTINGS: Settings = Settings {
//     RGB_mode: 0,
//     RGB_colour: RGB { r: 255, g: 0, b: 0 },
//     RGB_colour_2: RGB { r: 0, g: 255, b: 0 },
//     RGB_brightness: 64,
//     RGB_speed: 100,
// };

// static mut CURCOL: [u32; 3] = [0, 0, 0];
// static mut CURCYCLE: u8 = 0;
// static GOINGUP: AtomicBool = AtomicBool::new(true);

// fn fade_rgb_callback() {
//     let settings = &SETTINGS;
//     unsafe {
//         for i in 0..3 {
//             let diff = settings.RGB_colour.r - settings.RGB_colour_2.r;
//             let largerdiff = ((diff * settings.RGB_brightness) / (settings.RGB_speed << 10)) as u32;

//             if GOINGUP.load(Ordering::Relaxed) {
//                 CURCOL[i] += largerdiff;
//             } else {
//                 CURCOL[i] -= largerdiff;
//             }

//             if CURCOL[i] < 0 {
//                 CURCOL[i] = 0;
//             }
//             if CURCOL[i] > 16383 {
//                 CURCOL[i] = 16383;
//             }

//             // Assuming ledc_set_duty and ledc_update_duty are available in esp_idf_hal
//             // let channel = Channel::new(i + 4, LowSpeed);
//             // channel.set_duty(CURCOL[i]).unwrap();
//             // channel.update_duty().unwrap();
//         }
//     }
// }

// fn rainbow_rgb_callback() {
//     static mut CURCHAN: usize = 1;
//     static mut CURVAL: i32 = 0;

//     unsafe {
//         let settings = &SETTINGS;
//         CURVAL += if GOINGUP.load(Ordering::Relaxed) {
//             (12288 / settings.RGB_speed) as i32
//         } else {
//             -(12288 / settings.RGB_speed) as i32
//         };

//         if CURVAL <= 0 || CURVAL >= 16383 {
//             GOINGUP.store(!GOINGUP.load(Ordering::Relaxed), Ordering::Relaxed);
//             CURCHAN = (CURCHAN + 2) % 3;
//         }

//         if CURVAL > 16383 {
//             CURVAL = 16383;
//         }
//         if CURVAL < 0 {
//             CURVAL = 0;
//         }

//         // Assuming ledc_set_duty and ledc_update_duty are available in esp_idf_hal
//         // let channel = Channel::new(CURCHAN + 4, LowSpeed);
//         // channel.set_duty((CURVAL * settings.RGB_brightness) >> 14).unwrap();
//         // channel.update_duty().unwrap();
//     }
// }

// async fn rainbow_callback(_: &mut Gptimer, _: &GptimerEventCallbacks<'_>) {
//     rainbow_rgb_callback();
// }

// async fn fade_callback(_: &mut Gptimer, _: &GptimerEventCallbacks<'_>) {
//     fade_rgb_callback();
// }

// pub async fn rgb_update() {
//     if let Some(rainbow_timer) = unsafe { RainbowTimer::instance() } {
//         rainbow_timer.stop().unwrap();
//     }
//     if let Some(fade_timer) = unsafe { FadeTimer::instance() } {
//         fade_timer.stop().unwrap();
//     }

//     match SETTINGS.RGB_mode {
//         0 => {
//             // RGB_MODE_FADE
//             unsafe {
//                 CURCOL[0] = ((SETTINGS.RGB_colour.r * SETTINGS.RGB_brightness) >> 8) as u32;
//                 CURCOL[1] = ((SETTINGS.RGB_colour.g * SETTINGS.RGB_brightness) >> 8) as u32;
//                 CURCOL[2] = ((SETTINGS.RGB_colour.b * SETTINGS.RGB_brightness) >> 8) as u32;
//             }
//             unsafe { FadeTimer::instance() }.unwrap().start().unwrap();
//         }
//         1 => {
//             // RGB_MODE_RAINBOW
//             unsafe { RainbowTimer::instance() }.unwrap().start().unwrap();
//         }
//         2 => {
//             // RGB_MODE_OFF
//             for i in 0..3 {
//                 // Assuming ledc_stop is available in esp_idf_hal
//                 // let channel = Channel::new(i + 4, LowSpeed);
//                 // channel.stop(0).unwrap();
//             }
//         }
//         _ => {
//             // RGB_MODE_SOLID
//             for i in 0..3 {
//                 // Assuming ledc_set_duty and ledc_update_duty are available in esp_idf_hal
//                 // let channel = Channel::new(i + 4, LowSpeed);
//                 // channel.set_duty((SETTINGS.RGB_colour.r * (SETTINGS.RGB_brightness >> 8)) as u32).unwrap();
//                 // channel.update_duty().unwrap();
//             }
//         }
//     }
// }

// struct RainbowTimer {
//     timer: Gptimer,
// }

// impl RainbowTimer {
//     fn instance() -> Option<&'static mut Self> {
//         static mut INSTANCE: Option<RainbowTimer> = None;

//         unsafe {
//             INSTANCE.get_or_insert_with(|| {
//                 let config = GptimerConfig::new().enable(true);
//                 RainbowTimer {
//                     timer: Gptimer::new(config).unwrap(),
//                 }
//             })
//         }
//     }

//     fn stop(&mut self) -> Result<(), EspError> {
//         self.timer.stop()
//     }

//     fn start(&mut self) -> Result<(), EspError> {
//         let alarm_config = GptimerAlarmConfig::new().alarm_count(20_000).auto_reload(true);
//         self.timer.set_alarm_action(&alarm_config)?;
//         self.timer.register_event_callbacks(&GptimerEventCallbacks {
//             on_alarm: Some(rainbow_callback),
//             ..Default::default()
//         })?;
//         Ok(())
//     }
// }

// struct FadeTimer {
//     timer: Gptimer,
// }

// impl FadeTimer {
//     fn instance() -> Option<&'static mut Self> {
//         static mut INSTANCE: Option<FadeTimer> = None;

//         unsafe {
//             INSTANCE.get_or_insert_with(|| {
//                 let config = GptimerConfig::new().enable(true);
//                 FadeTimer {
//                     timer: Gptimer::new(config).unwrap(),
//                 }
//             })
//         }
//     }

//     fn stop(&mut self) -> Result<(), EspError> {
//         self.timer.stop()
//     }

//     fn start(&mut self) -> Result<(), EspError> {
//         let alarm_config = GptimerAlarmConfig::new().alarm_count(20_000).auto_reload(true);
//         self.timer.set_alarm_action(&alarm_config)?;
//         self.timer.register_event_callbacks(&GptimerEventCallbacks {
//             on_alarm: Some(fade_callback),
//             ..Default::default()
//         })?;
//         Ok(())
//     }
// }

// pub async fn rgb_init() {
//     unsafe { RainbowTimer::instance().unwrap(); }
//     unsafe { FadeTimer::instance().unwrap(); }
// }
