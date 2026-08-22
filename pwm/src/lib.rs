#![no_std]
#![no_main]

extern crate alloc;

use esp_hal::gpio::{AnyPin, Output, Level, OutputConfig, DriveMode};
use esp_hal::ledc::{channel::{Channel, Error, Number, config::Config, ChannelIFace}, timer::Timer, LowSpeed};
use esp_hal::time::Instant;
use embassy_futures::{select::{select3, Either3}, block_on};
use embassy_sync::channel::{Channel as EChannel, Sender};
use embassy_sync::rwlock::RwLock;
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Timer as ETimer, Duration};
use embassy_executor::Spawner;
use embedded_hal::pwm::SetDutyCycle;
use core::cmp::{Ord, Ordering, PartialOrd};
use core::fmt;
use alloc::collections::binary_heap::BinaryHeap;
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::future;

type Time = u64;
const NUM_CHANNELS: usize = 4; // TODO: MAKE THIS MATTER

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Command {
    action: PwmAction,
    time: Time,
}

impl PartialOrd for Command {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.time.partial_cmp(&other.time)?.reverse())
    }
}

impl Ord for Command {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time).reverse()
    }
}

impl Command {
    pub const fn new(action: PwmAction, time: Time) -> Self {
        Command { action, time }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PwmNumber {
    Pwm0,
    Pwm1,
    Pwm2,
    Pwm3,
}

impl fmt::Display for PwmNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PwmNumber::Pwm0 => write!(f, "[PWM 0]"),
            PwmNumber::Pwm1 => write!(f, "[PWM 1]"),
            PwmNumber::Pwm2 => write!(f, "[PWM 2]"),
            PwmNumber::Pwm3 => write!(f, "[PWM 3]"),
        }
    }
}

pub struct PwmOutOfBounds;

impl TryFrom<u8> for PwmNumber {
    type Error = PwmOutOfBounds;
    fn try_from(val: u8) -> Result<PwmNumber, PwmOutOfBounds> {
        match val {
            0 => Ok(PwmNumber::Pwm0),
            1 => Ok(PwmNumber::Pwm1),
            2 => Ok(PwmNumber::Pwm2),
            3 => Ok(PwmNumber::Pwm3),
            _ => Err(PwmOutOfBounds)
        }
    }
}

impl From<PwmNumber> for usize {
    fn from(val: PwmNumber) -> usize {
        match val {
            PwmNumber::Pwm0 => 0,
            PwmNumber::Pwm1 => 1,
            PwmNumber::Pwm2 => 2,
            PwmNumber::Pwm3 => 3,
        }
    }
}

impl PwmNumber {
    pub async fn get_state(&self) -> PwmPinInfo {
        PIN_STATES[usize::from(*self)].read().await.clone()
    }
}

pub struct PwmResponse;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PwmAction {
    On(PwmNumber),
    Off(PwmNumber),
    SetDuty(PwmNumber, u16),
    SetDutyPct(PwmNumber, u8),
    IncDutyPct(PwmNumber, u8),
    DecDutyPct(PwmNumber, u8),
    UnfreezePin(PwmNumber),
    Toggle(PwmNumber),
    FreezePin(PwmNumber, Time),
}

impl fmt::Display for PwmAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PwmAction::On(x) => { write!(f, "CHANNEL {} ON", x) },
            PwmAction::Off(x) => { write!(f, "CHANNEL {} OFF", x) },
            PwmAction::SetDuty(x, v) => { write!(f, "CHANNEL {} => DUTY {}", x, v) },
            PwmAction::SetDutyPct(x, p) => { write!(f, "CHANNEL {} => SET DUTY {}%", x, p) },
            PwmAction::IncDutyPct(x, p) => { write!(f, "CHANNEL {}: INCREMENT DUTY BY {}%", x, p) },
            PwmAction::DecDutyPct(x, p) => { write!(f, "CHANNEL {}: DECREMENT DUTY BY {}%", x, p) },
            PwmAction::UnfreezePin(x) => { write!(f, "UNFREEZE CHANNEL {}", x) },
            PwmAction::Toggle(x) => { write!(f, "TOGGLE CHANNEL {}", x) },
            PwmAction::FreezePin(x, t) => { write!(f, "FREEZE CHANNEL {} FOR {} MS", x, t) }
        }
    }
}

pub struct Pwm<'a> {
    // channels: [Channel<LowSpeed>; 4],
    command_sender: Sender<'a, CriticalSectionRawMutex, Command, 8>,
    result_signal: Arc<Signal<CriticalSectionRawMutex, PwmResponse>>,
    max_duty: u16,
}

impl Pwm<'_> {
    pub fn new(
        spawner: &Spawner,
        pins: [AnyPin<'static>; NUM_CHANNELS],
        timer: &'static Timer<'static, LowSpeed>,
    ) -> Result<Self, Error> {
        let config = OutputConfig::default();
        let channel_config = Config {
            timer,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        };
        let mut channels: Vec<Channel<LowSpeed>> = pins
                    .into_iter()
                    .zip(CHANNEL_NUMBERS)
                    .map(|(pin, cnum)| {
                        Channel::new(cnum, Output::new(pin, Level::Low, config))
                    })
                    .collect();
        channels.iter_mut()
                .try_for_each(|chan| {
                    chan.configure(channel_config)
                })?;
        let result_signal = Arc::new(Signal::new());
        let max_duty = channels[0].max_duty_cycle() - 1; // NOTE: this is because of a BUG in
                                                         // esp-hal.
        spawner.spawn(pwm_runner(channels, result_signal.clone(), max_duty).unwrap());

        Ok(Pwm {
            command_sender: COMMAND_CHANNEL.sender(),
            result_signal,
            max_duty,
        })
    }

    pub const fn pct_to_duty(&self, pct: u8) -> u16 {
        pct_to_duty(self.max_duty as u32, pct)
    }

    pub async fn send_and_forget(&self, c: Command) {
        self.result_signal.reset();
        self.command_sender.send(c).await;
    }

    pub async fn send_await(&self, c: Command) {
        self.command_sender.send(c).await;
        self.result_signal.wait().await;
        self.result_signal.reset();
    }

    pub async fn wait_result(&self) {
        self.result_signal.wait().await;
        self.result_signal.reset();
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PwmPinState {
    Off,
    On,
}

#[derive(Copy, Clone, Debug)]
pub struct PwmPinInfo {
    pub state: PwmPinState,
    pub duty: u16,
    pub freeze_timer: Time,
    pub frozen: bool,
}

impl PwmPinInfo {
    const fn new() -> Self {
        PwmPinInfo {
            state: PwmPinState::Off,
            duty: 16383,
            freeze_timer: 0,
            frozen: false,
        }
    }

    /// gets the duty as an integer percentage from 0 to 100.
    /// assumes 14 bit bitdepth (because screw you)
    pub const fn get_duty_pct(&self) -> u8 {
        (self.duty / 163) as u8
    }
}

async fn maybe_run<T>(t: Option<impl Future<Output = T>>) -> T {
    match t {
        Some(f) => f.await,
        None => future::pending().await
    }
}

static PIN_STATES: [RwLock<CriticalSectionRawMutex, PwmPinInfo>; NUM_CHANNELS] = [const { RwLock::new(PwmPinInfo::new()) }; NUM_CHANNELS];
static COMMAND_CHANNEL: EChannel<CriticalSectionRawMutex, Command, 8> = EChannel::new();

const CHANNEL_NUMBERS: [Number; 4] = [Number::Channel3, Number::Channel4, Number::Channel5, Number::Channel6];
#[embassy_executor::task]
async fn pwm_runner(mut channels: Vec<Channel<'static, LowSpeed>>, done_signal: Arc<Signal<CriticalSectionRawMutex, PwmResponse>>, md: u16) {
    let mut command_queue: BinaryHeap<Command> = BinaryHeap::new();
    let max_duty = md as u32;

    loop {
        let now = Instant::now();
        // let mut state = PIN_STATES[usize::from(c)].write().await;
        let next_delay = command_queue.peek().map(|e| 
            e.time.saturating_sub(now.duration_since_epoch().as_millis()));
        let delay_event = match next_delay {
            Some(d) => Some(ETimer::after(Duration::from_millis(d))),
            None => None,
        };
        let next_unfreeze = PIN_STATES.iter().map(|f| block_on(f.read()).freeze_timer).min().unwrap();
        let unfreeze_event = if next_unfreeze > 0 {
            Some(ETimer::after(Duration::from_millis(next_unfreeze)))
        } else {
            None
        };
        let next_action = select3(COMMAND_CHANNEL.receive(), maybe_run(delay_event), maybe_run(unfreeze_event)).await;

        match next_action {
            Either3::First(c) => {
                command_queue.push(Command::new(
                    c.action,
                    c.time + now.duration_since_epoch().as_millis(),
                ));
            },
            Either3::Second(_) => {
                let command = command_queue.pop().unwrap();
                execute_command(&mut channels, command.action, max_duty).await;
                done_signal.signal(PwmResponse);
            },
            Either3::Third(_) => {
                PIN_STATES.iter().for_each(|f| {
                    let mut pin_state = block_on(f.write());
                    pin_state.freeze_timer -= next_unfreeze;
                    if pin_state.freeze_timer == 0 {
                        pin_state.frozen = false;
                    }
                })
            }
        }
    }
}

async fn execute_command(channels: &mut Vec<Channel<'_, LowSpeed>>, action: PwmAction, max_duty: u32) {
    match action {
        PwmAction::On(c) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            channels[usize::from(c)].set_duty_cycle(state.duty).unwrap();
            state.state = PwmPinState::On;
        },
        PwmAction::Off(c) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            channels[usize::from(c)].set_duty_cycle(0).unwrap();
            state.state = PwmPinState::Off;
        },
        PwmAction::SetDuty(c, duty) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            if state.state == PwmPinState::On {
                channels[usize::from(c)].set_duty_cycle(duty).unwrap();
            }
            state.duty = duty;
        },
        PwmAction::SetDutyPct(c, pct) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            let duty_pct = pct_to_duty(max_duty, pct);
            if state.state == PwmPinState::On {
                channels[usize::from(c)].set_duty_cycle(duty_pct).unwrap();
            }
            state.duty = 164 * pct as u16;
        },
        PwmAction::IncDutyPct(c, p) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            let cur_duty_pct = state.get_duty_pct();
            if cur_duty_pct < 100 {
                // state.duty = ((((cur_duty_pct + p) as u32).min(100) * 16383u32) / 100) as u16;
                state.duty = pct_to_duty(max_duty, (cur_duty_pct + p).min(100));
                if state.state == PwmPinState::On {
                    channels[usize::from(c)].set_duty_cycle(state.duty).unwrap();
                }
            }
        },
        PwmAction::DecDutyPct(c, p) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            let cur_duty_pct = state.get_duty_pct();
            if cur_duty_pct > 0 {
                if state.state == PwmPinState::On {
                    channels[usize::from(c)].set_duty(cur_duty_pct.saturating_sub(p)).unwrap();
                }
                state.duty = 164 * (cur_duty_pct.saturating_sub(p)) as u16;
            }
        },
        PwmAction::UnfreezePin(c) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            if state.frozen {
                state.frozen = false;
                channels[usize::from(c)].set_duty_cycle(
                    match state.state {
                        PwmPinState::On => state.duty,
                        PwmPinState::Off => 0,
                    }
                ).unwrap();
            }
            // state.state = PwmPinState::On;
        },
        PwmAction::Toggle(c) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            match state.state {
                PwmPinState::On => {
                    channels[usize::from(c)].set_duty_cycle(0).unwrap();
                    state.state = PwmPinState::Off;
                },
                PwmPinState::Off => {
                    channels[usize::from(c)].set_duty_cycle(state.duty).unwrap();
                    state.state = PwmPinState::On;
                },
            }
        },
        PwmAction::FreezePin(c, t) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            // channels[usize::from(c)].set_duty_cycle(0).unwrap();
            state.frozen = true;
            state.freeze_timer = t;
        }
    }
}

const fn pct_to_duty(max_duty: u32, pct: u8) -> u16 {
    ((max_duty * pct as u32) / 100) as u16
}

