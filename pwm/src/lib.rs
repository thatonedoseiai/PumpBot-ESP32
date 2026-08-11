#![no_std]
#![no_main]

extern crate alloc;

use esp_hal::gpio::{AnyPin, Output, Level, OutputConfig};
use esp_hal::ledc::{channel::{Channel, Number}, LowSpeed};
use esp_hal::time::Instant;
use embassy_futures::{select::{select3, Either3}, block_on};
use embassy_sync::channel::{Channel as EChannel, Sender};
use embassy_sync::rwlock::RwLock;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Timer as ETimer, Duration};
use embassy_executor::Spawner;
use embedded_hal::pwm::SetDutyCycle;
use core::cmp::{Ord, Ordering, PartialOrd};
use core::fmt;
use alloc::collections::binary_heap::BinaryHeap;
use alloc::vec::Vec;
use futures::future::OptionFuture;

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
    Toggle(PwmNumber),
    FreezePin(PwmNumber, Time),
}

impl fmt::Display for PwmAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PwmAction::On(x) => { write!(f, "CHANNEL {} ON", x) },
            PwmAction::Off(x) => { write!(f, "CHANNEL {} OFF", x) },
            PwmAction::SetDuty(x, v) => { write!(f, "CHANNEL {} => DUTY {}", x, v) },
            PwmAction::UnfreezePin(x) => { write!(f, "UNFREEZE CHANNEL {}", x) },
            PwmAction::Toggle(x) => { write!(f, "TOGGLE CHANNEL {}", x) },
            PwmAction::FreezePin(x, t) => { write!(f, "FREEZE CHANNEL {} FOR {} MS", x, t) }
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

    pub async fn send(&self, c: Command) {
        self.command_sender.send(c).await;
    }
}

#[derive(Copy, Clone, Debug)]
enum PwmPinState {
    Off,
    On,
}

#[derive(Copy, Clone, Debug)]
struct PwmPinInfo {
    state: PwmPinState,
    duty: u16,
    freeze_timer: Time,
    frozen: bool,
}

impl PwmPinInfo {
    const fn new() -> Self {
        PwmPinInfo {
            state: PwmPinState::Off,
            duty: 0,
            freeze_timer: 0,
            frozen: false,
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
        // let mut state = PIN_STATES[usize::from(c)].write().await;
        let next_delay = command_queue.peek().map(|e| e.time - now.duration_since_epoch().as_millis());
        let delay_event: OptionFuture<_> = match next_delay {
            Some(d) => Some(ETimer::after(Duration::from_millis(d))),
            None => None,
        }.into();
        let next_unfreeze = PIN_STATES.iter().map(|f| block_on(f.read()).freeze_timer).min().unwrap();
        let unfreeze_event: OptionFuture<_> = if next_unfreeze > 0 {
            Some(ETimer::after(Duration::from_millis(next_unfreeze)))
        } else {
            None
        }.into();
        let next_action = select3(COMMAND_CHANNEL.receive(), delay_event, unfreeze_event).await;

        match next_action {
            Either3::First(c) => {
                command_queue.push(c);
            },
            Either3::Second(_) => {
                let command = command_queue.pop().unwrap();
                execute_command(&mut channels, command.action).await;
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
            state.state = PwmPinState::Off;
        },
        PwmAction::SetDuty(c, duty) => {
            let mut state = PIN_STATES[usize::from(c)].write().await;
            channels[usize::from(c)].set_duty_cycle(duty).unwrap();
            state.duty = duty;
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

