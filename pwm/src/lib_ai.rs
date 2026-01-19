use esp_idf_hal::ledc::{LedcDriver, LedcTimer, LedcChannel, LedcTimerDriver, LedcChannelDriver};
use esp_idf_hal::timer::{TimerDriver, TimerConfig};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::units::Hertz;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

// Priority queue item with timestamp
#[derive(Debug, Clone)]
pub struct PwmAction {
    pub timestamp: std::time::Instant,
    pub channel: usize,
    pub duty: u32,
    pub immediate: bool,
}

impl PartialEq for PwmAction {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl Eq for PwmAction {}

impl PartialOrd for PwmAction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.timestamp.cmp(&other.timestamp))
    }
}

impl Ord for PwmAction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

// PWM Controller struct
pub struct PwmController {
    ledc_driver: LedcDriver<'static>,
    timer_driver: LedcTimerDriver<'static>,
    channel_drivers: [Option<LedcChannelDriver<'static>>; 4],
    action_queue: Arc<Mutex<BinaryHeap<PwmAction>>>,
    should_stop: Arc<AtomicBool>,
    worker_thread: Option<thread::JoinHandle<()>>,
}

impl PwmController {
    pub fn new(
        peripherals: &Peripherals,
        timer: LedcTimer,
        channels: [LedcChannel; 4],
        pins: [esp_idf_hal::gpio::Pin<esp_idf_hal::gpio::Output>; 4],
        freq: Hertz,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize LEDC driver
        let mut ledc_driver = LedcDriver::new(peripherals.ledc, timer)?;
        ledc_driver.set_freq(freq)?;
        
        // Initialize channel drivers
        let mut channel_drivers = [None; 4];
        for (i, (channel, pin)) in channels.iter().zip(pins.iter()).enumerate() {
            let mut channel_driver = LedcChannelDriver::new(*channel, pin)?;
            channel_driver.set_duty(0)?;
            channel_drivers[i] = Some(channel_driver);
        }
        
        let action_queue = Arc::new(Mutex::new(BinaryHeap::new()));
        let should_stop = Arc::new(AtomicBool::new(false));
        
        let controller = Self {
            ledc_driver,
            timer_driver: ledc_driver.timer_driver(),
            channel_drivers,
            action_queue,
            should_stop,
            worker_thread: None,
        };
        
        Ok(controller)
    }
    
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let queue = self.action_queue.clone();
        let should_stop = self.should_stop.clone();
        let channel_drivers = self.channel_drivers.clone();
        
        let thread_handle = thread::spawn(move || {
            Self::worker_loop(queue, should_stop, channel_drivers);
        });
        
        self.worker_thread = Some(thread_handle);
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.should_stop.store(true, AtomicOrdering::Relaxed);
        
        if let Some(handle) = self.worker_thread.take() {
            handle.join().map_err(|_| "Failed to join worker thread")?;
        }
        
        Ok(())
    }
    
    pub fn turn_on(&mut self, channel: usize, duty: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.set_duty(channel, duty, false)
    }
    
    pub fn turn_off(&mut self, channel: usize) -> Result<(), Box<dyn std::error::Error>> {
        self.set_duty(channel, 0, false)
    }
    
    pub fn set_duty(&mut self, channel: usize, duty: u32, immediate: bool) -> Result<(), Box<dyn std::error::Error>> {
        if channel >= 4 {
            return Err("Invalid channel number".into());
        }
        
        let action = PwmAction {
            timestamp: std::time::Instant::now(),
            channel,
            duty,
            immediate,
        };
        
        {
            let mut queue = self.action_queue.lock().unwrap();
            queue.push(action);
        }
        
        Ok(())
    }
    
    pub fn turn_on_at(&mut self, channel: usize, duty: u32, delay: Duration) -> Result<(), Box<dyn std::error::Error>> {
        self.set_duty_at(channel, duty, delay, false)
    }
    
    pub fn turn_off_at(&mut self, channel: usize, delay: Duration) -> Result<(), Box<dyn std::error::Error>> {
        self.set_duty_at(channel, 0, delay, false)
    }
    
    pub fn set_duty_at(&mut self, channel: usize, duty: u32, delay: Duration, immediate: bool) -> Result<(), Box<dyn std::error::Error>> {
        if channel >= 4 {
            return Err("Invalid channel number".into());
        }
        
        let timestamp = std::time::Instant::now() + delay;
        let action = PwmAction {
            timestamp,
            channel,
            duty,
            immediate,
        };
        
        {
            let mut queue = self.action_queue.lock().unwrap();
            queue.push(action);
        }
        
        Ok(())
    }
    
    fn worker_loop(
        queue: Arc<Mutex<BinaryHeap<PwmAction>>>,
        should_stop: Arc<AtomicBool>,
        mut channel_drivers: [Option<LedcChannelDriver<'static>>; 4],
    ) {
        while !should_stop.load(AtomicOrdering::Relaxed) {
            let mut queue_lock = queue.lock().unwrap();
            
            if queue_lock.is_empty() {
                drop(queue_lock);
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            
            let action = queue_lock.pop().unwrap();
            drop(queue_lock);
            
            let now = std::time::Instant::now();
            if action.timestamp > now {
                // Sleep until action time
                let sleep_duration = action.timestamp.duration_since(now);
                thread::sleep(sleep_duration);
            }
            
            // Execute action
            if let Some(ref mut driver) = channel_drivers[action.channel] {
                driver.set_duty(action.duty).unwrap();
            }
        }
    }
}

// Example usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let peripherals = Peripherals::take().unwrap();
    
    // Initialize PWM channels
    let channels = [
        LedcChannel::Channel0,
        LedcChannel::Channel1,
        LedcChannel::Channel2,
        LedcChannel::Channel3,
    ];
    
    let pins = [
        PinDriver::output(peripherals.pins.gpio12)?,
        PinDriver::output(peripherals.pins.gpio13)?,
        PinDriver::output(peripherals.pins.gpio14)?,
        PinDriver::output(peripherals.pins.gpio15)?,
    ];
    
    // Create PWM controller
    let mut pwm_controller = PwmController::new(
        &peripherals,
        LedcTimer::Timer0,
        channels,
        pins,
        5000.Hz(), // 5kHz frequency
    )?;
    
    // Start the controller
    pwm_controller.start()?;
    
    // Example usage:
    // Turn on channel 0 with 50% duty cycle immediately
    pwm_controller.turn_on(0, 512)?; // Assuming 10-bit resolution
    
    // Turn off channel 1 after 2 seconds
    pwm_controller.turn_off_at(1, Duration::from_secs(2))?;
    
    // Set channel 2 to 75% duty cycle after 1 second
    pwm_controller.set_duty_at(2, 768, Duration::from_secs(1), false)?;
    
    // Keep the program running
    thread::sleep(Duration::from_secs(5));
    
    // Stop the controller
    pwm_controller.stop()?;
    
    Ok(())
}
