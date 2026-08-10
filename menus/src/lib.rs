//! This module contains the common logic for menus. Each menu will build on top of this module to
//! create its own unique functionality and interact with other menus (e.g. transitions, etc.)

#![no_std]
#![no_main]

extern crate alloc;

pub mod menus;
// pub mod components;
// mod event;
mod screen;
mod static_element;
// mod menu_define;
mod handlers;
mod components;
mod menu_definitions;

use crate::handlers::{ButtonHandler, MenuHandler, GenericHandler, Handler, HandlerResult};
use crate::components::{ButtonState, ComponentDefinition, ComponentState, RunHandlers, ButtonDefinition, ComponentBehaviour, InteractionType};
use crate::menu_definitions::{COMPONENT_TESTING, LANG, WIFI, WIFI_DETAILS};
use crate::static_element::StaticElement;

// pub use crate::event::event::Event;
use crate::menus::titlescreen::TitleState;
// use crate::menus::language_selection::LanguageState;
use crate::menus::setup_method::SetupMethodState;
// use crate::menus::test_component_menu::TestComponentMenu;
// use crate::menus_define;
// use ilidriver::ILIDriver;
// use alloc::sync::Arc;
use fontfile::{PbFont, pb_font_renderer::PbFontRenderer};
pub use crate::screen::screen::{Screen, ScreenDrawError};
use log::warn;
use core::fmt;
use rotenc::EncoderEvent;
use button_idf::{ButtonEvent, ButtonType, ButtonEventKind};
use rotenc::Direction;
use embassy_futures::select::{Either, select};
use wifi::PbWifi;
use alloc::{vec::Vec, vec};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::draw_target::DrawTarget;
use log::info;
use alloc::borrow::Cow;
use global_settings::PB_GLOBAL_SETTINGS;
use core::cell::{RefCell, Cell};
use esp_radio::wifi::ap::AccessPointInfo;
use alloc::string::String;

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

// #[derive(Debug)]
// pub enum MenuError {
//     UnimplementedMenu,
// }

// impl fmt::Display for MenuError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             MenuError::UnimplementedMenu => write!(f, "UnimplementedMenu")
//         }
//     }
// }

// impl core::error::Error for MenuError { }

enum MenuSignal {
    Back,
    Transition(Menu),
    Return,
}

struct ComponentMenuDefinition {
    components: &'static [ComponentDefinition],
    static_elements: &'static [StaticElement],
    left_btn: MenuHandler,
    right_btn: MenuHandler,
}

#[derive(PartialEq, Clone, Copy)]
enum ComponentMenuMode {
    Browse,
    Edit,
}

pub struct ComponentMenuInAction {
    layout: &'static ComponentMenuDefinition,
    component_states: Vec<ComponentState>,
    selected_component: usize,
    mode: ComponentMenuMode
}

impl ComponentMenuInAction {
    fn next_component(&mut self) {
        if !self.component_states.is_empty() {
            self.selected_component = (self.selected_component + 1) % self.component_states.len();
        }
    }

    fn prev_component(&mut self) {
        if !self.component_states.is_empty() {
            self.selected_component = (self.selected_component + self.component_states.len() - 1) % self.component_states.len();
        }
    }

    fn selected(&mut self) -> Option<&mut ComponentState> {
        if self.component_states.is_empty() {
            None
        } else {
            Some(&mut self.component_states[self.selected_component])
        }
    }

    async fn draw_all_components(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        for c in self.layout.static_elements {
            c.draw(&mut h.font, &mut h.screen).await?;
        }
        for m in self.component_states.iter_mut() {
            m.draw(h).await?;
        }
        Ok(())
    }
}

enum MenuInternalState {
    ComponentTesting { },
    Lang {
        language: u8,
    },
    Wifi,
    WifiDetails {
        ap: AccessPointInfo,
        pass: RefCell<String>,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentMenu {
    ComponentTesting,
    Lang(u8),
    Wifi,
    WifiDetails(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum CustomMenu {
    Title,
    SetupMethod,
}

#[derive(Debug, Clone, Copy)]
pub enum Menu {
    CustomMenu(CustomMenu),
    ComponentMenu(ComponentMenu),
}

trait MenuStateBehaviour {
    async fn run(self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal>;
}

impl ComponentMenu {
    const fn definition(&self) -> &'static ComponentMenuDefinition {
        match self {
            Self::ComponentTesting => &COMPONENT_TESTING,
            Self::Lang(_) => &LANG,
            Self::Wifi => &WIFI,
            Self::WifiDetails(_) => &WIFI_DETAILS,
        }
    }

    fn initial_state(self, h: &mut IOHandles<'_>) -> MenuInternalState {
        match self {
            Self::ComponentTesting => MenuInternalState::ComponentTesting { },
            Self::Lang(s) => MenuInternalState::Lang { language: s },
            Self::Wifi => MenuInternalState::Wifi,
            Self::WifiDetails(a) => MenuInternalState::WifiDetails { ap: h.wifi.get_wifis()[a].clone(), pass: RefCell::new(String::new()) },
        }
    }
}

impl MenuStateBehaviour for Menu {
    async fn run(self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        match self {
            Self::CustomMenu(m) => m.run(h).await,
            Self::ComponentMenu(m) => m.run(h).await,
        }
    }
}

impl MenuStateBehaviour for ComponentMenu {
    async fn run(self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let num_components = self.definition().components.len();
        let mut internal_state = self.initial_state(h);
        let mut cur_menu_state = ComponentMenuInAction {
            layout: self.definition(),
            component_states: self.definition()
                                      .components
                                      .into_iter()
                                      .map(|f| f.construct(num_components == 1, self.definition(), &internal_state, h))
                                      .collect(),
            selected_component: 0,
            mode: if num_components > 1 { ComponentMenuMode::Browse } else { ComponentMenuMode::Edit },
        };
        cur_menu_state.selected().map(|c| c.highlight());

        cur_menu_state.draw_all_components(h).await?;
        loop {
            let inp = select(
                h.button.receive(),
                h.rotenc.receive(),
            ).await;
            let signal: Option<HandlerResult> = match inp {
                Either::Second(EncoderEvent {dir: Direction::Clockwise, ..}) => {
                    match cur_menu_state.mode {
                        ComponentMenuMode::Browse => {
                            if let Some(m) = cur_menu_state.selected() {
                                m.unhighlight().draw(h).await?;
                            }
                            cur_menu_state.next_component();
                            if let Some(m) = cur_menu_state.selected() {
                                m.highlight().draw(h).await?;
                            }
                            None
                        },
                        ComponentMenuMode::Edit => 
                            if let Some(f) = cur_menu_state.selected() {
                                Some(f.right_handle(&mut internal_state, h).await?)
                            } else {
                                None
                            }
                    }
                },
                Either::Second(EncoderEvent {dir: Direction::Anticlockwise, ..}) => {
                    match cur_menu_state.mode {
                        ComponentMenuMode::Browse => {
                            if let Some(m) = cur_menu_state.selected() {
                                m.unhighlight().draw(h).await?;
                            }
                            cur_menu_state.prev_component();
                            if let Some(m) = cur_menu_state.selected() {
                                m.highlight().draw(h).await?;
                            }
                            None
                        },
                        ComponentMenuMode::Edit =>
                            if let Some(f) = cur_menu_state.selected() {
                                Some(f.left_handle(&mut internal_state, h).await?)
                            } else {
                                None
                            }
                    }
                },
                Either::First(ButtonEvent {
                    button_type: ButtonType::Left,
                    event: ButtonEventKind::Down
                }) => Some(cur_menu_state.layout.left_btn.handle(&mut cur_menu_state, &mut internal_state, h).await?),
                Either::First(ButtonEvent {
                    button_type: ButtonType::Right,
                    event: ButtonEventKind::Down
                }) => Some(cur_menu_state.layout.right_btn.handle(&mut cur_menu_state, &mut internal_state, h).await?),
                Either::First(ButtonEvent {
                    button_type: ButtonType::Rotenc,
                    event: ButtonEventKind::Down
                }) => {
                    let cur_mode = cur_menu_state.mode;
                    match (cur_mode, cur_menu_state.selected().map(|f| f.definition().interaction_type())) {
                        (ComponentMenuMode::Browse, Some(InteractionType::Editable)) => {
                            cur_menu_state.mode = ComponentMenuMode::Edit;
                            if let Some(f) = cur_menu_state.selected() {
                                Some(f.click_handle(&mut internal_state, h).await?)
                            } else {
                                None
                            }
                        },
                        (ComponentMenuMode::Browse, Some(InteractionType::NonEditable)) => 
                            if let Some(f) = cur_menu_state.selected() {
                                Some(f.click_handle(&mut internal_state, h).await?)
                            } else {
                                None
                            },
                        (ComponentMenuMode::Edit, _) => 
                            if let Some(f) = cur_menu_state.selected() {
                                Some(f.click_handle(&mut internal_state, h).await?)
                            } else {
                                None
                            },
                        _ => None
                    }
                },
                _ => None,
            };
            // println!("selecting {}", cur_menu_state.selected_component);
            match signal {
                Some(HandlerResult::Transition(m)) => {
                    // println!("TRANSITIONING TO {:?}", m);
                    return Ok(MenuSignal::Transition(m));
                },
                Some(HandlerResult::Unfocus) => {
                    if num_components > 1 {
                        cur_menu_state.mode = ComponentMenuMode::Browse;
                    }
                },
                Some(HandlerResult::ForceRedrawAndUnfocus) => {
                    cur_menu_state.draw_all_components(h).await?;
                    if num_components > 1 {
                        cur_menu_state.mode = ComponentMenuMode::Browse;
                    }
                },
                Some(HandlerResult::Back) => {
                    // println!("GOING BACK TO PREVIOUS MENU");
                    return Ok(MenuSignal::Back);
                },
                Some(HandlerResult::None) => {},
                None => {},
            }
        }
    }
}

impl MenuStateBehaviour for CustomMenu {
    async fn run(self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        // Do custom things here
        // Ok(MenuSignal::Transition(Menu::CustomMenu(CustomMenu::CustomTitle)))
        match self {
            Self::Title => TitleState::new().run(h).await,
            Self::SetupMethod => SetupMethodState::new().run(h).await,
        }
    }
}

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
pub async fn run_menu_loop(spawner: Spawner, start_menu: Menu, io_handles: &mut IOHandles<'_>) -> anyhow::Result<()> {
    log::info!("beginning menu loop!");

    let mut menu_stack = vec![];
    let mut menu = start_menu;
    loop {
        io_handles.screen.clear(PB_GLOBAL_SETTINGS.read().await.theme.bg().into())?;
        let result = menu.run(io_handles).await?;
        match result {
            MenuSignal::Transition(m) => {
                menu_stack.push(menu);
                menu = m; 
            },
            MenuSignal::Back => {
                let maybe_menu = menu_stack.pop();
                if let Some(m) = maybe_menu {
                    menu = m;
                }
            },
            MenuSignal::Return => {
                return Ok(());
            }
        }
        // PROFILER.lock().unwrap().dump();
        // SpanGuard::dump();
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
