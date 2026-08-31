//! This module contains the common logic for menus. Each menu will build on top of this module to
//! create its own unique functionality and interact with other menus (e.g. transitions, etc.)

#![no_std]
#![no_main]
#![feature(const_index)]
#![feature(const_trait_impl)]

extern crate alloc;

pub mod menus;
mod screen;
mod static_element;
mod handlers;
mod components;
mod menu_definitions;

use crate::handlers::{
    MenuHandler,
    GenericHandler,
    Handler,
    HandlerResult
};
use crate::components::{
    ComponentDefinition,
    ComponentState,
    RunHandlers,
    ComponentBehaviour,
    InteractionType
};
use crate::menu_definitions::{
    COMPONENT_TESTING,
    LANG,
    WIFI,
    WIFI_DETAILS,
    SERVER_DETAILS,
    DISPLAY_SETTINGS,
    DISPLAY_SETTINGS_SETUP,
    SETTINGS_MENU,
    RGB_MENU,
    WIFI_SETTINGS_MENU,
    WIFI_DETAILS_SETTINGS_MENU,
    SERVER_DETAILS_SETTINGS_MENU,
    PWM_WIZARD,
};
use crate::static_element::StaticElement;
use crate::menus::titlescreen::TitleState;
use crate::menus::setup_method::SetupMethodState;
use crate::menus::home_menu::HomeMenu;
use crate::menus::pwm_wizard::PwmWizardState;
pub use crate::screen::screen::{
    Screen,
    ScreenDrawError
};

use fontfile::{
    pb_font_renderer::PbFontRenderer,
    FontSize,
};
use rotenc::{
    EncoderEvent,
    Direction,
    RotencDriver
};
use button_idf::{
    ButtonEvent,
    ButtonType,
    ButtonEventKind
};
use embassy_futures::select::{
    Either,
    select
};
use wifi::PbWifi;
use embedded_graphics::{
    prelude::*,
    draw_target::DrawTarget,
    text::{
        Alignment,
        Text,
    },
    primitives::{
        Circle,
        PrimitiveStyleBuilder,
        PrimitiveStyle,
        RoundedRectangle,
        Rectangle,
        Line,
    },
    pixelcolor::Rgb565,
};
use global_settings::{
    PB_GLOBAL_SETTINGS,
    PbGlobalSettings,
    rgb,
    rgb::RGB,
    Theme
};
use esp_radio::wifi::ap::AccessPointInfo;
use embassy_net::IpAddress;
use socket::ServerConnection;
use esp_hal::ledc::{
    channel::{
        Channel, 
        ChannelIFace
    }, 
    LowSpeed
};
use ledc::LedMode;
use enum_dispatch::enum_dispatch;
use futures_util::stream::{
    iter,
    StreamExt
};

use core::cell::RefCell;
use core::ops::Deref;
use alloc::{
    vec::Vec, 
    vec, 
    string::String,
    borrow::Cow,
};

#[cfg(feature = "sim")]
mod cond_deps {
    pub mod mock_queue;
    pub mod mock_ledc;
    pub mod mock_pwm;

    pub use mock_pwm::OutputCtl;
    pub use mock_ledc::LedController;
    pub use mock_queue::Queue;
    pub use embedded_graphics_simulator::{SimulatorDisplay, Window, OutputSettingsBuilder, SimulatorEvent};
}
#[cfg(not(feature = "sim"))]
mod cond_deps {
    pub use pwm::{
        Pwm,
        PwmNumber,
    };
    pub use ledc::LedController;
    pub use embassy_sync::{channel::Receiver, blocking_mutex::raw::CriticalSectionRawMutex};
    pub use embassy_executor::Spawner;
}

use crate::cond_deps::*;

enum MenuSignal {
    Back,
    BackN(u8),
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

    fn advance_component(&mut self, n: i16) {
        if !self.component_states.is_empty() {
            self.selected_component = ((self.selected_component as i16 + n).rem_euclid(self.component_states.len() as i16)) as usize;
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

#[derive(Debug)]
enum MenuInternalState {
    ComponentTesting { },
    Lang {
        language: u8,
    },
    Wifi,
    WifiDetails {
        ap: AccessPointInfo,
        pass: RefCell<String>,
    },
    ServerDetails {
        ip: IpAddress,
        port: u16,
    },
    DisplaySettings {
        theme_custom_col: RGB,
    },
    Settings,
    RgbMenu {
        mode: LedMode,
        primary_col: RGB,
        secondary_col: RGB,
    },
    DisplaySettingsSettingsMenu {
        theme_custom_col: RGB,
    },
    WifiSettingsMenu,
    WifiDetailsSettingsMenu {
        ap: AccessPointInfo,
        pass: RefCell<String>,
    },
    ServerDetailsSettingsMenu {
        ip: IpAddress,
        port: u16,
    },
    PwmWizard {
        selected_channel: PwmNumber,
        voltage: u8,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentMenu {
    ComponentTesting,
    Lang,
    Wifi,
    WifiDetails(usize),
    ServerDetails,
    DisplaySettings,
    Settings,
    RgbMenu,
    DisplaySettingsSettingsMenu,
    WifiSettingsMenu,
    WifiDetailsSettingsMenu(usize),
    ServerDetailsSettingsMenu,
    PwmWizard,
}

#[derive(Debug, Clone, Copy)]
pub enum CustomMenu {
    Title,
    SetupMethod,
    HomeMenu,
    PwmWizard,
}

#[enum_dispatch(MenuStateBehaviour)]
#[derive(Debug, Clone, Copy)]
pub enum Menu {
    CustomMenu(CustomMenu),
    ComponentMenu(ComponentMenu),
}

#[enum_dispatch]
trait MenuStateBehaviour {
    async fn run(self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal>;
}

impl ComponentMenu {
    const DIALOGUE_BOX: RoundedRectangle = RoundedRectangle::with_equal_corners(
                        Rectangle::new(Point::new(10, 10), Size::new(108, 140)),
                        Size::new(10, 10),
                    );

    const fn dialogue_box_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_width(3)
            .stroke_color(theme.fg().as_rgb565())
            .fill_color(theme.bg_secondary().as_rgb565())
            .build()
    }

    const fn cross_bg_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(theme.fg().as_rgb565())
            .fill_color(theme.bg_secondary().as_rgb565())
            .build()
    }

    const fn definition(&self) -> &'static ComponentMenuDefinition {
        match self {
            Self::ComponentTesting => &COMPONENT_TESTING,
            Self::Lang => &LANG,
            Self::Wifi => &WIFI,
            Self::WifiDetails(_) => &WIFI_DETAILS,
            Self::ServerDetails => &SERVER_DETAILS,
            Self::DisplaySettings => &DISPLAY_SETTINGS_SETUP,
            Self::Settings => &SETTINGS_MENU,
            Self::RgbMenu => &RGB_MENU,
            Self::DisplaySettingsSettingsMenu => &DISPLAY_SETTINGS,
            Self::WifiSettingsMenu => &WIFI_SETTINGS_MENU,
            Self::WifiDetailsSettingsMenu(_) => &WIFI_DETAILS_SETTINGS_MENU,
            Self::ServerDetailsSettingsMenu => &SERVER_DETAILS_SETTINGS_MENU,
            Self::PwmWizard => &PWM_WIZARD,
        }
    }

    pub fn draw_error_dialogue(message: Cow<'_, str>, theme: &Theme, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        h.font.font.borrow_mut().set_size(FontSize::Sz7)?;
        h.font.bgcol = theme.bg_secondary();
        Circle::new(Point::new(2, 2), 11)
            .into_styled(Self::cross_bg_style(theme))
            .draw(&mut h.screen)?;
        Line::new(Point::new(5, 5), Point::new(10, 10))
            .into_styled(PrimitiveStyle::with_stroke(theme.highlight().as_rgb565(), 2))
            .draw(&mut h.screen)?;
        Line::new(Point::new(5, 10), Point::new(10, 5))
            .into_styled(PrimitiveStyle::with_stroke(theme.highlight().as_rgb565(), 2))
            .draw(&mut h.screen)?;
        Self::DIALOGUE_BOX.into_styled(Self::dialogue_box_style(&theme)).draw(&mut h.screen)?;
        Text::with_alignment(&message, Self::DIALOGUE_BOX.rectangle.top_left + Point::new(5, 10), &h.font, Alignment::Left).draw(&mut h.screen)?;
        Ok(())
    }

    fn initial_state(self, h: &mut IOHandles<'_>, settings: &PbGlobalSettings) -> MenuInternalState {
        match self {
            Self::ComponentTesting => MenuInternalState::ComponentTesting { },
            Self::Lang => MenuInternalState::Lang { language: settings.lang.into() },
            Self::Wifi => MenuInternalState::Wifi,
            Self::WifiDetails(a) => {
                let ap = if a < h.wifi.get_wifis().len() {
                    h.wifi.get_wifis()[a].clone()
                } else {
                    AccessPointInfo::default()
                };
                MenuInternalState::WifiDetails { 
                    ap,
                    pass: RefCell::new(String::new())
                }
            },
            Self::ServerDetails => MenuInternalState::ServerDetails {
                ip: h.server.server_ip,
                port: h.server.server_port,
            },
            Self::DisplaySettings => MenuInternalState::DisplaySettings {
                theme_custom_col: if let Theme::Custom(r) = settings.theme { r } else { rgb![128, 128, 128] },
            },
            Self::Settings => MenuInternalState::Settings,
            Self::RgbMenu => MenuInternalState::RgbMenu {
                mode: LedMode::Off,
                primary_col: rgb![128],
                secondary_col: rgb![128],
            },
            Self::DisplaySettingsSettingsMenu => MenuInternalState::DisplaySettingsSettingsMenu {
                theme_custom_col: if let Theme::Custom(r) = settings.theme { r } else { rgb![128, 128, 128] },
            },
            Self::WifiSettingsMenu => MenuInternalState::WifiSettingsMenu,
            Self::WifiDetailsSettingsMenu(a) => {
                let ap = if a < h.wifi.get_wifis().len() {
                    h.wifi.get_wifis()[a].clone()
                } else {
                    AccessPointInfo::default()
                };
                MenuInternalState::WifiDetailsSettingsMenu { 
                    ap,
                    pass: RefCell::new(String::new())
                }
            },
            Self::ServerDetailsSettingsMenu => MenuInternalState::ServerDetailsSettingsMenu {
                ip: h.server.server_ip,
                port: h.server.server_port,
            },
            Self::PwmWizard => MenuInternalState::PwmWizard {
                selected_channel: PwmNumber::Pwm0,
                voltage: 12, // 12V default
            }
        }
    }
}

impl MenuStateBehaviour for ComponentMenu {
    async fn run(self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let num_components = self.definition().components.len();
        let mut internal_state = self.initial_state(h, &PB_GLOBAL_SETTINGS.read().await.deref());
        let mut component_states = Vec::new();
        for component in self.definition().components.into_iter() {
            component_states.push(component.construct(num_components == 1, self.definition(), &internal_state, h).await);
        }
        let mut cur_menu_state = ComponentMenuInAction {
            layout: self.definition(),
            component_states,
            selected_component: 0,
            mode: if num_components > 1 { ComponentMenuMode::Browse } else { ComponentMenuMode::Edit },
        };
        cur_menu_state.selected().map(|c| c.highlight());

        h.font.bgcol = PB_GLOBAL_SETTINGS.read().await.theme.bg();
        cur_menu_state.draw_all_components(h).await?;
        loop {
            let inp = select(
                h.button.receive(),
                h.rotenc.receive(0),
            ).await;
            let signal: Option<HandlerResult> = match inp {
                Either::Second(k) => {
                    match cur_menu_state.mode {
                        ComponentMenuMode::Browse => {
                            if let Some(m) = cur_menu_state.selected() {
                                m.unhighlight();
                                m.draw(h).await?;
                            }
                            cur_menu_state.advance_component(k);
                            if let Some(m) = cur_menu_state.selected() {
                                m.highlight();
                                m.draw(h).await?;
                            }
                            None
                        },
                        ComponentMenuMode::Edit => 
                            if let Some(f) = cur_menu_state.selected() {
                                if k > 0 {
                                    Some(f.right_handle(&mut internal_state, h, k).await?)
                                } else {
                                    Some(f.left_handle(&mut internal_state, h, k).await?)
                                }
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
                    let bgcol = PB_GLOBAL_SETTINGS.read().await.theme.bg();
                    h.screen.clear(bgcol.as_rgb565())?;
                    h.font.bgcol = bgcol;
                    cur_menu_state.component_states.iter_mut().for_each(|f| { f.reset_draw_flags(); });
                    cur_menu_state.draw_all_components(h).await?;
                    if num_components > 1 {
                        cur_menu_state.mode = ComponentMenuMode::Browse;
                    }
                },
                Some(HandlerResult::Back) => {
                    // println!("GOING BACK TO PREVIOUS MENU");
                    return Ok(MenuSignal::Back);
                },
                Some(HandlerResult::BackN(x)) => {
                    return Ok(MenuSignal::BackN(x));
                }
                Some(HandlerResult::WifiConnectionFailure(e)) => {
                    todo!("handle wifi connection error! {}", e)
                    // return Ok(MenuSignal::None);
                },
                Some(HandlerResult::ShowErrorDialogue(message)) => {
                    let theme = {
                        PB_GLOBAL_SETTINGS.read().await.theme
                    };
                    Self::draw_error_dialogue(message, &theme, h)?;
                    while h.button.receive().await.event != ButtonEventKind::Down {}; // only continue on click
                    let bgcol = PB_GLOBAL_SETTINGS.read().await.theme.bg();
                    h.screen.clear(bgcol.as_rgb565())?;
                    h.font.bgcol = bgcol;
                    cur_menu_state.component_states.iter_mut().for_each(|f| { f.reset_draw_flags(); });
                    cur_menu_state.draw_all_components(h).await?;
                },
                Some(HandlerResult::None) => {},
                None => {},
            }
        }
    }
}

impl MenuStateBehaviour for CustomMenu {
    async fn run(self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        match self {
            Self::Title => TitleState::new().run(h).await,
            Self::SetupMethod => SetupMethodState::new().run(h).await,
            Self::HomeMenu => HomeMenu::new().run(h).await,
            Self::PwmWizard => PwmWizardState::new().run(h).await,
        }
    }
}

/// A collection of the various IOHandles that menus should be allowed to interact with.
pub struct IOHandles<'a> {
    pub screen: Screen<'a>,
    pub leddriver: LedController,
    pub pwm_output: Pwm<'a>,
    pub font: PbFontRenderer,
    pub button: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, 10>,
    pub rotenc: RotencDriver,
    pub wifi: PbWifi<'a>,
    pub server: ServerConnection,
    pub backlight: Channel<'a, LowSpeed>,
    pub backlight_brightness_pct: u8,
}

impl<'a> IOHandles<'a> {
    pub fn new(
        screen: Screen<'a>,
        leddriver: LedController,
        pwm_output: Pwm<'a>,
        font: PbFontRenderer,
        button: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, 10>,
        rotenc: RotencDriver,
        wifi: PbWifi<'a>,
        server: ServerConnection,
        backlight: Channel<'a, LowSpeed>,
        backlight_brightness_pct: u8,
    ) -> IOHandles<'a> {
        Self { screen, leddriver, pwm_output, font, button, rotenc, wifi, server, backlight, backlight_brightness_pct }
    }

    pub fn set_brightness_pct(&mut self, pct: u8) -> Result<(), esp_hal::ledc::channel::Error> {
        self.backlight.set_duty(pct)?;
        self.backlight_brightness_pct = pct;
        Ok(())
    }
}

/// Starts a menuing tree. If the user returns from the menu at the bottom of the tree, the
/// function will return. Parameters:
/// - `start_menu`: the first menu to start displaying
/// - `io_handles`: a collection of IO handles that the menus should be allowed to interact with
/// - `q`: a queue that receives events from the buttons and rotary encoder and sends them for the
/// menus to use to react to button presses and rotenc spins.
#[cfg(not(feature = "sim"))]
pub async fn run_menu_loop(_: Spawner, start_menu: Menu, io_handles: &mut IOHandles<'_>) -> anyhow::Result<()> {
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
            MenuSignal::BackN(k) => {
                menu_stack.truncate(menu_stack.len() - k as usize + 1);
                let maybe_menu = menu_stack.pop();
                if let Some(m) = maybe_menu {
                    menu = m;
                }
            }
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
