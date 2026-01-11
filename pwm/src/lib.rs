// void init_pb_output_info(void);
// void done_pb_output_info(void);
// void output_set_power(int channel, char enable);
// void output_toggle(int channel);
// void output_set_value(int channel, int level);
// void output_set_value_timeout(int channel, int level, int ms);
// int output_get_value(int channel);
// char is_off(int channel);
// void output_add_value(int channel, int increment);
// char output_was_updated(int channel);
// void pwm_timeout_add_value(int channel, int increment);

use esp_idf_hal::gpio::{AnyIOPin, PinDriver, Output};
use esp_idf_hal::ledc::{CHANNEL3, CHANNEL4, CHANNEL5, CHANNEL6, TIMER1, LedcDriver, LedcTimerDriver, config::TimerConfig, Resolution};
use esp_idf_hal::timer::{TimerDriver, TIMER10, config::Config};
use esp_idf_hal::sys::EspError;
use esp_idf_hal::task::{notification::Notification, do_yield};
use esp_idf_hal::delay::FreeRtos;
use std::sync::{Arc, Mutex, RwLock, mpsc::{Sender, Receiver, channel}, Condvar, atomic::{AtomicU8, Ordering}, PoisonError, MutexGuard};
use priority_queue::PriorityQueue;
use std::fmt;
use std::thread;
use std::cmp::Reverse;
use std::num::NonZeroU32;
use log::{info, error};
use esp_idf_hal::prelude::FromValueType;

pub struct OutputPeripherals {
    output_pins: (AnyIOPin, AnyIOPin, AnyIOPin, AnyIOPin),
    channels: (CHANNEL3, CHANNEL4, CHANNEL5, CHANNEL6),
    timer: TIMER1,
}

impl OutputPeripherals {
    pub fn new(output_pins: (AnyIOPin, AnyIOPin, AnyIOPin, AnyIOPin), channels: (CHANNEL3, CHANNEL4, CHANNEL5, CHANNEL6), timer: TIMER1) -> Self {
        Self {
            output_pins, channels, timer
        }
    }
}

type Time = u64;
type Result<T, E = EspError> = std::result::Result<T, E>;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    On(u8),
    Off(u8),
    SetDuty(u8, u16),
    UnfreezePin(u8),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::On(x) => { write!(f, "CHANNEL {} ON", x) },
            Action::Off(x) => { write!(f, "CHANNEL {} OFF", x) },
            Action::SetDuty(x, v) => { write!(f, "CHANNEL {} => DUTY {}", x, v) },
            Action::UnfreezePin(x) => { write!(f, "UNFREEZE CHANNEL {}", x) },
        }
    }
}

struct OutputState<'a> {
    off_flag: [bool; 4],
    frozen_flag: [bool; 4],
    action_queue: PriorityQueue<Action, Reverse<Time>>,
    timer: TimerDriver<'a>,
}

impl fmt::Display for OutputState<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pwmmask: ({} {} {} {})", 
            self.off_flag[0],
            self.off_flag[1],
            self.off_flag[2],
            self.off_flag[3],
            )
    }
}

pub struct OutputCtl<'a> {
    state: Arc<Mutex<OutputState<'a>>>
}

impl OutputCtl<'static> {
    pub fn new(peripherals: OutputPeripherals, gptimer: TIMER10) -> Result<OutputCtl<'static>> {
        let config = Config::default()
                            .auto_reload(false)
                            .divider(40000);
        let mut timer: TimerDriver<'static> = TimerDriver::new(gptimer, &config)?;
        let flag = Arc::new(AtomicU8::new(0));
        let flagclone = flag.clone();
        let action_queue = PriorityQueue::new();

        unsafe {
            timer.subscribe(move || {
                flagclone.fetch_add(1, Ordering::SeqCst);
            })?;
        }

        timer.enable(true)?;

        let state: Arc<Mutex<OutputState<'static>>> = Arc::new(Mutex::new(OutputState {
            off_flag: [false; 4],
            action_queue,
            frozen_flag: [false; 4],
            timer: timer,
        }));

        let stateclone = state.clone();

        let _ = thread::Builder::new()
            .stack_size(3400)
            .spawn(move || -> Result<()> {
            let config = TimerConfig::new().resolution(Resolution::Bits14);
                //.frequency(5_u32.kHz().into());
            let timerdriver = LedcTimerDriver::new(peripherals.timer, &config)?;
            let mut outputs: Vec<LedcDriver> = Vec::new();
            outputs.push(LedcDriver::new(peripherals.channels.0, &timerdriver, peripherals.output_pins.0)?);
            outputs.push(LedcDriver::new(peripherals.channels.1, &timerdriver, peripherals.output_pins.1)?);
            outputs.push(LedcDriver::new(peripherals.channels.2, &timerdriver, peripherals.output_pins.2)?);
            outputs.push(LedcDriver::new(peripherals.channels.3, &timerdriver, peripherals.output_pins.3)?);
            for i in &mut outputs {
                i.disable()?;
            }
            // let mut outputs = peripherals.output_pins.into_iter().map(|(x, c)| {
            //                                  LedcDriver::new(x, &peripherals.timer, c)
            //                              })
            //                              .collect::<Result<Vec<LedcDriver>>>()?;
            let mut acts = Vec::new();
            loop {
                let mut result: u8 = flag.swap(0, Ordering::Relaxed);
                while result == 0 {
                    FreeRtos::delay_ms(10);
                    result = flag.swap(0, Ordering::Relaxed);
                }
                let mut frozen = [false;4];
                with_lock(&stateclone, |mut val| {
                    frozen.copy_from_slice(&val.frozen_flag);
                    // let action_queue = &mut val.action_queue;
                    if let Some((action, timestamp)) = val.action_queue.pop() {
                        acts.push(action);
                        loop {
                            match val.action_queue.peek() {
                                Some((&action, &time)) => {
                                    // let timer = &mut val.timer;
                                    let now = val.timer.counter()?;
                                    info!("now {} action buf {}", now, val.action_queue.len());
                                    if time.0 <= now {
                                        acts.push(action);
                                        val.action_queue.pop();
                                        continue;
                                    }
                                    val.action_queue.iter_mut().for_each(|(action, time)| { time.0 -= now });
                                    val.timer.set_counter(0)?;
                                    val.timer.set_alarm(time.0 - now)?;
                                    val.timer.enable_interrupt()?;
                                    val.timer.enable_alarm(true)?;
                                },
                                None => {
                                    val.timer.set_counter(0)?;
                                    // info!("timer being set to 0");
                                }
                            }
                            break;
                        }
                    }
                    Ok(())
                })?;
                while let Some(action) = acts.pop() {
                    info!("received action {}", action);
                    match action {
                        Action::On(pin) => {
                            outputs[pin as usize].enable()?;
                        },
                        Action::Off(pin) => {
                            outputs[pin as usize].disable()?;
                        },
                        Action::SetDuty(pin, amt) => {
                            // let duty = outputs[pin as usize].get_max_duty();
                            outputs[pin as usize].set_duty(amt as u32)?;
                        },
                        Action::UnfreezePin(pin) => {
                            let _ = with_lock(&stateclone, |mut val| {
                                val.frozen_flag[pin as usize] = false;
                                Ok(())
                            });
                        }
                    }
                }
            }
        });

        Ok(Self { state })
    }

    pub fn buffer_action(&self, action: Action, delay_ms: u64) -> Result<()> {
        with_lock(&self.state, |mut state: MutexGuard<OutputState>| {
            // with_lock(&state.action_queue, move |mut queue: MutexGuard<PriorityQueue<Action, Time>>| {
            let timestamp = state.timer.counter()?.wrapping_add(
                (state.timer.tick_hz() / 1000).wrapping_mul(delay_ms));
            let enable_alarm = state.action_queue.peek().is_none();
            // info!("buffering {}, ts: {} now: {} alm: {}", action, timestamp, state.timer.counter()?, enable_alarm);
            state.action_queue.push(action, Reverse(timestamp));
            if let Some((_, &t)) = state.action_queue.peek() {
                state.timer.set_alarm(t.0)?;
            }
            if enable_alarm {
                state.timer.enable_interrupt()?;
                state.timer.enable_alarm(true)?;
            }
            Ok(())
            // })
        })
    }

    pub const max_duty: u16 = 16383;
}


fn with_lock<T, F>(v: &Arc<Mutex<T>>, mut cb: F) -> Result<()>
    where F: FnMut(MutexGuard<T>) -> Result<()>,
          // T: fmt::Display,
{
    let res = v.lock();
    match res {
        Ok(val) => {
            cb(val)
        },
        Err(e) => {
            // let internal_data = e.into_inner();
            error!("ERROR! PWM LOCK POISONED.");
            panic!();
        }
    }
}
