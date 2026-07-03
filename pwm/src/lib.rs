#![no_std]
#![no_main]

extern crate alloc;

use esp_hal::gpio::{AnyPin, Output, Level, OutputConfig};
use esp_hal::ledc::{channel::{Channel, Number}, LowSpeed};
use esp_hal::time::Instant;
use embassy_futures::select::{select, Either};
use embassy_sync::channel::{Channel as EChannel, Sender};
use embassy_sync::rwlock::RwLock;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Timer as ETimer, Duration};
use embassy_executor::Spawner;
use embedded_hal::pwm::SetDutyCycle;
use core::cmp::{Ord, Ordering, PartialOrd};
use core::fmt;
use core::future;
use alloc::collections::binary_heap::BinaryHeap;
use alloc::vec::Vec;

type Time = u64;
const NUM_CHANNELS: usize = 4; // TODO: MAKE THIS MATTER

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Command {
    action: PwmAction,
    time: Time,
}

impl PartialOrd for Command {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for Command {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time).reverse()
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PwmAction {
    On(PwmNumber),
    Off(PwmNumber),
    SetDuty(PwmNumber, u16),
    UnfreezePin(PwmNumber),
}

impl fmt::Display for PwmAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PwmAction::On(x) => { write!(f, "CHANNEL {} ON", x) },
            PwmAction::Off(x) => { write!(f, "CHANNEL {} OFF", x) },
            PwmAction::SetDuty(x, v) => { write!(f, "CHANNEL {} => DUTY {}", x, v) },
            PwmAction::UnfreezePin(x) => { write!(f, "UNFREEZE CHANNEL {}", x) },
        }
    }
}

pub struct Pwm<'a> {
    // channels: [Channel<LowSpeed>; 4],
    command_sender: Sender<'a, CriticalSectionRawMutex, Command, 8>
}

impl Pwm<'_> {
    pub fn new(
        spawner: &Spawner,
        pins: [AnyPin<'static>; NUM_CHANNELS]
    ) -> Self {
        spawner.spawn(pwm_runner(pins).unwrap());

        Pwm {
            command_sender: COMMAND_CHANNEL.sender()
        }
    }

    pub fn send(&self, c: Command) {
        let _ = self.command_sender.try_send(c);
    }
}

#[derive(Copy, Clone, Debug)]
enum PwmPinState {
    Off,
    On,
    Frozen,
}

#[derive(Copy, Clone, Debug)]
struct PwmPinInfo {
    state: PwmPinState,
    duty: u16,
}

impl PwmPinInfo {
    const fn new() -> Self {
        PwmPinInfo {
            state: PwmPinState::Off,
            duty: 0,
        }
    }
}

static PIN_STATES: [RwLock<CriticalSectionRawMutex, PwmPinInfo>; NUM_CHANNELS] = [const { RwLock::new(PwmPinInfo::new()) }; NUM_CHANNELS];
static COMMAND_CHANNEL: EChannel<CriticalSectionRawMutex, Command, 8> = EChannel::new();

const CHANNEL_NUMBERS: [Number; 4] = [Number::Channel3, Number::Channel4, Number::Channel5, Number::Channel6];
#[embassy_executor::task]
async fn pwm_runner(pins: [AnyPin<'static>; NUM_CHANNELS]) {
    let mut command_queue: BinaryHeap<Command> = BinaryHeap::new();
    let now = Instant::now();
    let config = OutputConfig::default();
    let mut channels: Vec<Channel<LowSpeed>> = pins.into_iter().zip(CHANNEL_NUMBERS).map(|(pin, cnum)| Channel::new(cnum, Output::new(pin, Level::Low, config))).collect();

    loop {
        let next_delay = command_queue.peek().map(|e| e.time - now.duration_since_epoch().as_millis());
        let next_action = match next_delay {
            Some(delay) => select(COMMAND_CHANNEL.receive(), ETimer::after(Duration::from_millis(delay))).await,
            None => select(COMMAND_CHANNEL.receive(), future::pending::<()>()).await,
        };

        match next_action {
            Either::First(c) => {
                command_queue.push(c);
            }
            Either::Second(_) => {
                let command = command_queue.pop().unwrap();
                execute_command(&mut channels, command.action).await;
            }
        }
    }
}

async fn execute_command(channels: &mut Vec<Channel<'_, LowSpeed>>, action: PwmAction) {
    match action {
        PwmAction::On(c) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            channels[usize::from(c)].set_duty_cycle(state.duty).unwrap();
            state.state = PwmPinState::On;
        },
        PwmAction::Off(c) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            channels[usize::from(c)].set_duty_cycle(0).unwrap();
            state.state = PwmPinState::On;
        },
        PwmAction::SetDuty(c, duty) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            channels[usize::from(c)].set_duty_cycle(duty).unwrap();
            state.duty = duty;
        },
        PwmAction::UnfreezePin(c) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            channels[usize::from(c)].set_duty_cycle(state.duty).unwrap();
            state.state = PwmPinState::On;
        }
    }
}

