//! This module contains the common logic for menus. Each menu will build on top of this module to
//! create its own unique functionality and interact with other menus (e.g. transitions, etc.)

#![no_std]
#![no_main]

extern crate alloc;

pub mod menus;
// pub mod components;
mod event;
mod screen;
mod menu_define;

pub use crate::event::event::Event;
use crate::menus::titlescreen::TitleState;
// use crate::menus::language_selection::LanguageState;
// use crate::menus::setup_method::SetupMethodState;
// use crate::menus::test_component_menu::TestComponentMenu;
// use crate::menus_define;
// use ilidriver::ILIDriver;
// use alloc::sync::Arc;
use fontfile::{PbFont, pb_font_renderer::PbFontRenderer};
pub use crate::screen::screen::{Screen, ScreenDrawError};
use log::warn;
use core::fmt;
use rotenc::EncoderEvent;
use button_idf::{ButtonEvent, ButtonType};
use rotenc::Direction;
use embassy_futures::select::{Either, select};
use wifi::PbWifi;
use alloc::{vec::Vec, vec};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::draw_target::DrawTarget;
use log::info;
use alloc::borrow::Cow;

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

#[derive(Debug, Clone, Copy)]
enum ComponentSignal {
    Next,
    Prev,
    Transition(&'static Layout),
    None,
}

#[derive(Debug)]
enum Handler {
    Print(Cow<'static, str>),
    Signal(ComponentSignal),
}

impl Handler {
    fn dispatch(&self, io_handles: &mut IOHandles<'_>) -> ComponentSignal {
        match self {
            Handler::Print(s) => {
                info!("{}", s);
                ComponentSignal::None
            },
            Handler::Signal(s) => {
                info!("SIGNAL (TODO: SHOW WHAT SIGNAL IS COMING OUT)");
                *s
            }
        }
    }
}



/// Defines the visual construction of a menu
#[derive(Debug)]
pub struct Layout {
    menu_type: Menu,
    components: &'static [Component],
    left_behaviour: Handler,
    right_behaviour: Handler,
}

impl From<&'static Layout> for MenuState {
    fn from(val: &'static Layout) -> MenuState {
        match val.menu_type {
            Menu::TitleMenu => MenuState::TitleMenu (
                TitleState::new()
            ),
            Menu::Unimplemented => MenuState::Unimplemented(
                ComponentMenu { 
                    layout: val, 
                    state: EmptyState { } 
                }
            ),
            // Menu::Lang => MenuState::Lang {
            //     layout: val, state: TitleInternalState { }
            // },
        }
    }
}

#[derive(Debug)]
pub struct Component {
    onclick: Handler,
    rotenc_right: Handler,
    rotenc_left: Handler,
    component_spec: ComponentType,
}

#[derive(Debug)]
pub enum ComponentType {
    Button,
}

pub enum ComponentState {
    Button { 
        highlighted: bool,
    },
}

pub struct ComponentWithState {
    state: ComponentState,
    component: &'static Component
}

impl From<&'static Component> for ComponentWithState {
    fn from(val: &'static Component) -> ComponentWithState {
        ComponentWithState {
            state: match val.component_spec {
                ComponentType::Button => ComponentState::Button {
                    highlighted: false
                }
            },
            component: val,
        }
    }
}

pub trait ComponentBehaviour {
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, f: &mut PbFontRenderer, d: &mut D) -> Result<(), <D as DrawTarget>::Error>;
}

impl ComponentBehaviour for ComponentWithState {
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, f: &mut PbFontRenderer, d: &mut D) -> Result<(), <D as DrawTarget>::Error> {
        match self.component.component_spec {
            ComponentType::Button => { info!("draw button!") }
        };
        Ok(())
    }
}

/// Signals for menu controller actions, such as to stop menuing, transition to a different menu,
/// or continue displaying the same menu.
pub enum MenuSignal {
    None,
    Return,
    Back,
    Transition(&'static Layout)
}

#[derive(Debug, PartialEq)]
pub enum Menu {
    TitleMenu,
    // LanguageMenu,
    // SetupMethodMenu,
    Unimplemented
}

struct ComponentMenu {
    layout: &'static Layout,
    state: EmptyState,
}

enum MenuState {
    TitleMenu(TitleState),
    // LanguageMenu { layout: &'static Layout, state: EmptyState },
    // SetupMethodMenu { layout: &'static Layout, state: EmptyState },
    Unimplemented(ComponentMenu),
}

pub trait MenuBehaviour {
    async fn run(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal>;
}

impl MenuBehaviour for MenuState {
    async fn run(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        match self {
            MenuState::TitleMenu(t) => t.run(io_handles).await,
            MenuState::Unimplemented(t) => t.run(io_handles).await,
        }
    }
}

impl MenuBehaviour for ComponentMenu {
    async fn run(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let mut components = Vec::new();
        for i in self.layout.components.into_iter() {
            let with_state: ComponentWithState = i.into();
            with_state.draw(&mut io_handles.font.clone(), &mut io_handles.screen)?;
            components.push(with_state);
        }

        let mut selected_component = 0;
        loop {
            let result = select(
                io_handles.rotenc.receive(),
                io_handles.button.receive(),
            ).await;
            let signal = match result {
                Either::First(r) => {
                    match r.dir {
                        Direction::Clockwise => self.layout.components[selected_component].rotenc_right.dispatch(io_handles),
                        Direction::Anticlockwise => self.layout.components[selected_component].rotenc_left.dispatch(io_handles),
                        _ => { ComponentSignal::None },
                    }
                },
                Either::Second(b) => {
                    match b.button_type {
                        ButtonType::Right => self.layout.right_behaviour.dispatch(io_handles),
                        ButtonType::Left => self.layout.left_behaviour.dispatch(io_handles),
                        ButtonType::Rotenc => self.layout.components[selected_component].onclick.dispatch(io_handles),
                    }
                }
            };
            match signal {
                ComponentSignal::None => {},
                ComponentSignal::Next => {
                    selected_component = (selected_component + 1) % self.layout.components.len();
                    info!("selected component: {}", selected_component);
                },
                ComponentSignal::Prev => {
                    selected_component = (selected_component + self.layout.components.len() - 1) % self.layout.components.len();
                    info!("selected component: {}", selected_component);
                }
                ComponentSignal::Transition(t) => {
                    info!("transitioning to {:?}", t);
                    return Ok(MenuSignal::Transition(t));
                }
            }
        }
    }
}

struct EmptyState { }

// menus_define![
//     TitleMenu, Title(TitleState);
//     LanguageMenu, Lang(LanguageState);
//     SetupMethodMenu, SetupMethod(SetupMethodState);
//     ComponentMenuTest, TESTCOMPONENTMENU(TestComponentMenu)
// ];

/// A collection of the various IOHandles that menus should be allowed to interact with.
pub struct IOHandles<'a> {
    pub screen: Screen<'a>,
    pub leddriver: LedController,
    pub pwm_output: Pwm<'a>,
    pub font: PbFontRenderer,
    pub button: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, 10>,
    pub rotenc: Receiver<'static, CriticalSectionRawMutex, EncoderEvent, 10>,
    pub wifi: PbWifi<'a>,
}

impl<'a> IOHandles<'a> {
    pub fn new(
        screen: Screen<'a>,
        leddriver: LedController,
        pwm_output: Pwm<'a>,
        font: PbFontRenderer,
        button: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, 10>,
        rotenc: Receiver<'static, CriticalSectionRawMutex, EncoderEvent, 10>,
        wifi: PbWifi<'a>,
    ) -> IOHandles<'a> {
        Self { screen, leddriver, pwm_output, font, button, rotenc, wifi }
    }
}

/// Starts a menuing tree. If the user returns from the menu at the bottom of the tree, the
/// function will return. Parameters:
/// - `start_menu`: the first menu to start displaying
/// - `io_handles`: a collection of IO handles that the menus should be allowed to interact with
/// - `q`: a queue that receives events from the buttons and rotary encoder and sends them for the
/// menus to use to react to button presses and rotenc spins.
#[cfg(not(feature = "sim"))]
pub async fn run_menu_loop(spawner: Spawner, start_menu: &'static Layout, io_handles: &mut IOHandles<'_>) -> anyhow::Result<()> {
    log::info!("beginning menu loop!");

    let mut layout = MenuSignal::Transition(start_menu);
    loop {
        match layout {
            MenuSignal::Transition(l) => { 
                if l.menu_type == Menu::Unimplemented {
                    warn!("transition to unimplemented menu! Returning now.");
                    return Err(MenuError::UnimplementedMenu)?;
                }
                io_handles.screen.clear(Rgb565::BLACK)?;
                let mut m: MenuState = l.into();
                layout = m.run(io_handles).await?;
                // cur_menu = m.into();
                // cur_menu.init(spawner, io_handles).await?;
            },
            MenuSignal::Return => { return Ok(()); },
            _ => {}
        }

        // PROFILER.lock().unwrap().dump();
        // SpanGuard::dump();
    }
}

pub const TITLESCREEN: Layout = Layout {
    menu_type: Menu::TitleMenu,
    components: &[],
    left_behaviour: Handler::Print(Cow::Borrowed("left button!")),
    right_behaviour: Handler::Print(Cow::Borrowed("right button!")),
};

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
