//! This module contains the common logic for menus. Each menu will build on top of this module to
//! create its own unique functionality and interact with other menus (e.g. transitions, etc.)

pub mod menus;
pub mod mock_queue;
pub mod mock_ledc;
pub mod mock_pwm;

mod event;
mod screen;

pub use crate::event::event::Event;
use crate::menus::titlescreen::{TitleState};
// use ilidriver::ILIDriver;
use std::sync::Arc;
use fontfile::{PbFont, pb_font_renderer::PbFontRenderer};
pub use crate::screen::screen::{Screen, ScreenDrawError};
use log::warn;
use std::fmt;

use profiler::SpanGuard;

#[cfg(target_os = "espidf")]
use pwm::OutputCtl;
#[cfg(not(target_os = "espidf"))]
use mock_pwm::OutputCtl;

#[cfg(target_os = "espidf")]
use ledc::LedController;
#[cfg(not(target_os = "espidf"))]
use mock_ledc::LedController;

#[cfg(target_os = "espidf")]
use esp_idf_hal::task::queue::Queue;
#[cfg(not(target_os = "espidf"))]
use mock_queue::Queue;

#[cfg(not(target_os = "espidf"))]
use embedded_graphics_simulator::{SimulatorDisplay, Window, OutputSettingsBuilder, SimulatorEvent};

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

impl std::error::Error for MenuError { }

/// Signals for menu controller actions, such as to stop menuing, transition to a different menu,
/// or continue displaying the same menu.
pub enum MenuSignal {
    None,
    Return,
    Transition(MenuSelection)
}

/// The list of all currently implemented menus.
#[derive(PartialEq, Debug)]
pub enum MenuSelection {
    TitleMenu,
    Unimplemented
}

/// A wrapper around the current states of every menu.
enum MenuStates {
    Title(TitleState),
}

impl fmt::Display for MenuStates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuStates::Title(_) => write!(f, "Title"),
        }
    }
}

/// A collection of the various IOHandles that menus should be allowed to interact with.
pub struct IOHandles<'a> {
    pub screen: Screen<'a>,
    pub leddriver: LedController,
    pub pwm_output: OutputCtl<'a>,
    pub font: PbFontRenderer, 
}

impl<'a> IOHandles<'a> {
    pub fn new( screen: Screen<'a>, leddriver: LedController, pwm_output: OutputCtl<'a>, font: PbFontRenderer) -> IOHandles<'a> {
        Self { screen, leddriver, pwm_output, font }
    }
}

/// A trait that defines the behaviours that menus are required to implement.
pub trait MenuBehaviour: Sized {
    // type Args: Into<Self> + From<MenuSelection>;
    fn init(&mut self, io_handles: &mut IOHandles) -> anyhow::Result<MenuSignal>;
    fn update(&mut self, io_handles: &mut IOHandles, events: &mut Vec<Event>) -> anyhow::Result<MenuSignal>;
}

impl From<MenuSelection> for MenuStates {
    fn from(val: MenuSelection) -> MenuStates {
        match val {
            MenuSelection::TitleMenu => MenuStates::Title(TitleState::new()),
            MenuSelection::Unimplemented => unreachable!(),
        }
    }
}

impl MenuBehaviour for MenuStates {
    // Args = MenuSelection;
    fn init(&mut self, io_handles: &mut IOHandles) -> anyhow::Result<MenuSignal> {
        match self {
            MenuStates::Title(t) => t.init(io_handles),
        }
    }

    fn update(&mut self, io_handles: &mut IOHandles, events: &mut Vec<Event>) -> anyhow::Result<MenuSignal> {
        match self {
            MenuStates::Title(t) => t.update(io_handles, events),
        }
    }
}

/// Starts a menuing tree. If the user returns from the menu at the bottom of the tree, the
/// function will return. Parameters:
/// - `start_menu`: the first menu to start displaying
/// - `io_handles`: a collection of IO handles that the menus should be allowed to interact with
/// - `q`: a queue that receives events from the buttons and rotary encoder and sends them for the
/// menus to use to react to button presses and rotenc spins.
#[cfg(target_os = "espidf")]
pub fn run_menu_loop(start_menu: MenuSelection, io_handles: &mut IOHandles, q: Arc<Queue<Event>>) -> anyhow::Result<()> {

    let mut cur_menu: MenuStates = start_menu.into();
    let mut events = vec![];
    cur_menu.init(io_handles)?;

    log::info!("beginning loop!");

    loop {
        // q.recv(10);
        if let Some((ev, _)) = q.recv_front(1) { // PROBLEM HERE???
            events.push(ev);
        }
        let response = cur_menu.update(io_handles, &mut events)?;
            match response {
                MenuSignal::Transition(m) => { 
                    if m == MenuSelection::Unimplemented {
                        warn!("transition to unimplemented menu from {}! Returning now.", cur_menu);
                        return Err(MenuError::UnimplementedMenu)?;
                    }
                    cur_menu = m.into();
                    cur_menu.init(io_handles)?;
                },
                MenuSignal::Return => { return Ok(()); },
                _ => {}
            }
        events.clear();

        // PROFILER.lock().unwrap().dump();
        SpanGuard::dump();
    }
}

#[cfg(not(target_os = "espidf"))]
pub fn run_menu_loop(start_menu: MenuSelection, io_handles: &mut IOHandles, q: Arc<Queue<Event>>, mut window: Window) -> anyhow::Result<()> {

    let mut cur_menu: MenuStates = start_menu.into();
    let mut events = vec![];
    cur_menu.init(io_handles)?;

    loop {
        // q.recv(10);
        if let Some((ev, _)) = q.recv_front(10) {
            events.push(ev);
        }
        let response = cur_menu.update(io_handles, &mut events)?;
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