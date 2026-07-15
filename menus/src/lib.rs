//! This module contains the common logic for menus. Each menu will build on top of this module to
//! create its own unique functionality and interact with other menus (e.g. transitions, etc.)

#![no_std]
#![no_main]

extern crate alloc;

pub mod menus;
mod event;
mod screen;
mod menu_define;

pub use crate::event::event::Event;
use crate::menus::titlescreen::{TitleState};
use crate::menus::language_selection::LanguageState;
// use crate::menus_define;
// use ilidriver::ILIDriver;
// use alloc::sync::Arc;
use fontfile::{PbFont, pb_font_renderer::PbFontRenderer};
pub use crate::screen::screen::{Screen, ScreenDrawError};
use log::warn;
use core::fmt;
use rotenc::EncoderEvent;
use button_idf::ButtonEvent;
use alloc::{vec::Vec, vec};

use profiler::SpanGuard;

#[cfg(feature = "sim")]
mod cond_deps {
    pub mod mock_queue;
    pub mod mock_ledc;
    pub mod mock_pwm;

    pub use mock_pwm::OutputCtl;
    pub use mock_ledc::LedController;
    // pub use esp_idf_hal::task::queue::Queue;
    pub use mock_queue::Queue;
    pub use embedded_graphics_simulator::{SimulatorDisplay, Window, OutputSettingsBuilder, SimulatorEvent};
}
#[cfg(not(feature = "sim"))]
mod cond_deps {
    pub use pwm::Pwm;
    pub use ledc::LedController;
    pub use embassy_sync::{channel::Receiver, blocking_mutex::raw::CriticalSectionRawMutex};
    pub use embassy_executor::Spawner;
    pub use embassy_time::{Timer, Duration};
}

use crate::cond_deps::*;

#[derive(Debug)]
pub enum MenuError {
    UnimplementedMenu,
}

impl fmt::Display for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuError::UnimplementedMenu => write!(f, "UnimplementedMenu")
        }
    }
}

impl core::error::Error for MenuError { }

/// Signals for menu controller actions, such as to stop menuing, transition to a different menu,
/// or continue displaying the same menu.
pub enum MenuSignal {
    None,
    Return,
    Back,
    Transition(MenuSelection)
}

menus_define![
    TitleMenu, Title(TitleState);
    LanguageMenu, Lang(LanguageState)
];

/// A collection of the various IOHandles that menus should be allowed to interact with.
pub struct IOHandles<'a> {
    pub screen: Screen<'a>,
    pub leddriver: LedController,
    pub pwm_output: Pwm<'a>,
    pub font: PbFontRenderer,
    pub button: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, 10>,
    pub rotenc: Receiver<'static, CriticalSectionRawMutex, EncoderEvent, 10>,
}

impl<'a> IOHandles<'a> {
    pub fn new(
        screen: Screen<'a>,
        leddriver: LedController,
        pwm_output: Pwm<'a>,
        font: PbFontRenderer,
        button: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, 10>,
        rotenc: Receiver<'static, CriticalSectionRawMutex, EncoderEvent, 10>
    ) -> IOHandles<'a> {
        Self { screen, leddriver, pwm_output, font, button, rotenc }
    }
}

/// A trait that defines the behaviours that menus are required to implement.
pub trait MenuBehaviour: Sized {
    // type Args: Into<Self> + From<MenuSelection>;
    fn init(&mut self, spawner: Spawner, io_handles: &mut IOHandles) -> impl core::future::Future<Output = anyhow::Result<MenuSignal>>;
    fn update(&mut self, io_handles: &mut IOHandles) -> impl core::future::Future<Output = anyhow::Result<MenuSignal>>;
}

/// Starts a menuing tree. If the user returns from the menu at the bottom of the tree, the
/// function will return. Parameters:
/// - `start_menu`: the first menu to start displaying
/// - `io_handles`: a collection of IO handles that the menus should be allowed to interact with
/// - `q`: a queue that receives events from the buttons and rotary encoder and sends them for the
/// menus to use to react to button presses and rotenc spins.
#[cfg(not(feature = "sim"))]
pub async fn run_menu_loop(spawner: Spawner, start_menu: MenuSelection, io_handles: &mut IOHandles<'_>) -> anyhow::Result<()> {

    let mut cur_menu: MenuStates = start_menu.into();
    // let mut events = vec![];
    cur_menu.init(spawner, io_handles).await?;

    log::info!("beginning menu loop!");

    loop {
        // q.recv(10);
        // if let Some((ev, _)) = q.recv_front(1) { // PROBLEM HERE???
        //     events.push(ev);
        // }
        // if let Ok(button_event) = button_events.try_receive() {
        //     events.push(button_event.into());
        // }
        // if let Ok(rotenc_event) = rotenc_events.try_receive() {
        //     events.push(rotenc_event.into())
        // }
        let response = cur_menu.update(io_handles).await?;
        match response {
            MenuSignal::Transition(m) => { 
                if m == MenuSelection::Unimplemented {
                    warn!("transition to unimplemented menu from {}! Returning now.", cur_menu);
                    return Err(MenuError::UnimplementedMenu)?;
                }
                cur_menu = m.into();
                cur_menu.init(spawner, io_handles).await?;
            },
            MenuSignal::Return => { return Ok(()); },
            _ => {}
        }
        // events.clear();

        Timer::after(Duration::from_millis(10)).await; // do the other tasks

        // PROFILER.lock().unwrap().dump();
        SpanGuard::dump();
    }
}

#[cfg(feature = "sim")]
pub async fn run_menu_loop(start_menu: MenuSelection, io_handles: &mut IOHandles, q: Arc<Queue<Event>>, mut window: Window) -> anyhow::Result<()> {

    let mut cur_menu: MenuStates = start_menu.into();
    let mut events = vec![];
    cur_menu.init(io_handles)?;

    loop {
        // q.recv(10);
        if let Some((ev, _)) = q.recv_front(10) {
            events.push(ev);
        }
        let response = cur_menu.update(io_handles, &mut events).await?;
        match response {
            MenuSignal::Transition(m) => { 
                cur_menu = m.into();
                cur_menu.init(io_handles)?;
            },
            MenuSignal::Return => { return Ok(()); },
            _ => {}
        }
        window.update(&io_handles.screen.disp);
        for event in window.events() {
            if event == SimulatorEvent::Quit {
                return Ok(());
            }
        }
        events.clear();
    }
}
