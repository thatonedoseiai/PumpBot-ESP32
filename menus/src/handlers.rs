use crate::components::{ButtonState, OptionSwitchState, OptionSwitchMode, ComponentBehaviour, OptionScrollerState, TextBoxState, ValueSelectorState, ValueSelectorNumType, ValueSelectorMode, ColorSelectorState};
use crate::{ComponentMenu, ComponentMenuInAction, Menu, IOHandles, MenuInternalState, ComponentMenuDefinition};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use global_settings::{lang::{Lang, LanguageString, TEXT_CONNECT, TEXT_DISCONNECT, TEXT_CUSTOM}, PB_GLOBAL_SETTINGS, rgb::RGB, rgb, Theme};
use alloc::string::{ToString, String};
use esp_radio::wifi::{Ssid, WifiError};
use core::str::FromStr;
use embassy_net::IpAddress;
use socket::ServerConnection;
use embedded_graphics::{
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle}
};
use alloc::format;
use esp_radio::wifi::AuthenticationMethod;
use ledc::LedMode;
use embassy_futures::block_on;

#[derive(Debug, Copy, Clone)]
pub enum HandlerError {
    BacklightSetError(esp_hal::ledc::channel::Error),
}
impl core::error::Error for HandlerError { }
impl core::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::BacklightSetError(e) => write!(f, "Backlight failed to set properly: {:?}", e)
        }
    }
}

pub enum MenuInternalStateAction {
    SetLang,
    SetWifi,
}

#[derive(Debug, Clone)]
pub enum HandlerResult {
    None,
    Transition(Menu),
    Back,
    Unfocus,
    ForceRedrawAndUnfocus,
    WifiConnectionFailure(WifiError),
    ShowErrorDialogue(Cow<'static, str>),
}

pub trait Handler<S> {
    async fn handle(&self, state: &mut S, menu_state: &mut MenuInternalState, _: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
}

// GENERIC {{{
pub enum GenericHandler {
    Print(&'static str),
    Signal(HandlerResult),
    SetLanguageAndTransition(Menu),
    ConnectDisconnectWifi,
    ConnectServer,
}

impl Handler<()> for GenericHandler {
    async fn handle(&self, _: &mut (), menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Print(s) => {
                log::info!("{s}");
                Ok(HandlerResult::None)
            },
            Self::Signal(s) => {
                Ok(s.clone())
            },
            Self::SetLanguageAndTransition(m) => {
                if let MenuInternalState::Lang { language } = menu_state {
                    let mut settings = PB_GLOBAL_SETTINGS.write().await;
                    settings.lang = (*language).into();
                    Ok(HandlerResult::Transition(*m))
                } else {
                    panic!("Handler::SetLanguageAndTransition used on bad menu!");
                }
            },
            Self::ConnectDisconnectWifi => {
                if let MenuInternalState::WifiDetails {
                    ap: a,
                    pass: p
                } = menu_state {
                    if !h.wifi.is_connected() {
                        let pw = p.borrow();
                        log::warn!("Connecting to wifi: SSID {}, pass {}", a.ssid.as_str(), &pw);
                        let wifi_res = h.wifi.connect(&a, &pw).await;
                        match wifi_res {
                            Ok(()) => {
                                log::info!("connect returned OK!");
                                Ok(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::ServerDetails)))
                            },
                            Err(e) => {
                                let error_msg = format!("Failed to connect to Wifi!\n\nError:\n{}", e);
                                Ok(HandlerResult::ShowErrorDialogue(Cow::Owned(error_msg)))
                            }
                        }
                    } else {
                        h.wifi.disconnect().await?;
                        Ok(HandlerResult::None)
                    }
                } else {
                    unreachable!();
                }
            },
            Self::ConnectServer => {
                let res = h.server.connect().await;
                match res {
                    Ok(_) => {
                        Ok(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::DisplaySettings)))
                    },
                    Err(_) => {
                        Ok(HandlerResult::ShowErrorDialogue(Cow::Borrowed("Pb failed to connect\nto the server!")))
                    }
                }
            },
        }
    }
}
// }}}
// MENU HANDLER {{{
pub enum MenuHandler {
    Generic(GenericHandler)
}

impl Handler<ComponentMenuInAction> for MenuHandler {
    async fn handle(&self, _: &mut ComponentMenuInAction, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), menu_state, h).await
            }
        }
    }
}
// }}}
// BUTTON HANDLER {{{
pub enum ButtonHandler {
    Generic(GenericHandler),
}

impl Handler<ButtonState> for ButtonHandler {
    async fn handle(&self, _: &mut ButtonState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), menu_state, h).await
            },
        }
    }
}
// }}}
// OPTION SWITCH HANDLER {{{
pub enum OptionSwitchHandler {
    Generic(GenericHandler),
    NextElement,
    PrevElement,
    NextElementUpdateLang,
    PrevElementUpdateLang,
    ToggleFocus,
    PrintSelection,
    ToggleFocusAndSetTheme,
    ToggleFocusAndSetRGB,
}

impl Handler<OptionSwitchState> for OptionSwitchHandler {
    async fn handle(&self, state: &mut OptionSwitchState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), menu_state, h).await
            },
            Self::NextElement => {
                state.selection = (state.selection + 1) % state.definition.options.len();
                state.draw(h).await?;
                Ok(HandlerResult::None)
            },
            Self::PrevElement => {
                let num_options = state.definition.options.len();
                state.selection = (state.selection + num_options - 1) % num_options;
                state.draw(h).await?;
                Ok(HandlerResult::None)
            },
            Self::NextElementUpdateLang => {
                if let MenuInternalState::Lang { language } = menu_state {
                    state.selection = (state.selection + 1) % state.definition.options.len();
                    state.draw(h).await?;
                    *language = state.selection as u8;
                    Ok(HandlerResult::None)
                } else {
                    panic!("OptionSwitchHandler::NextElementUpdateLang used on non-language menu!");
                }
            },
            Self::PrevElementUpdateLang => {
                if let MenuInternalState::Lang { language } = menu_state {
                    let num_options = state.definition.options.len();
                    state.selection = (state.selection + num_options - 1) % num_options;
                    state.draw(h).await?;
                    *language = state.selection as u8;
                    Ok(HandlerResult::None)
                } else {
                    panic!("OptionSwitchHandler::NextElementUpdateLang used on non-language menu!");
                }
            },
            Self::ToggleFocus => {
                let old_state_mode = state.mode;
                state.mode = match state.mode {
                    OptionSwitchMode::Unhighlighted => OptionSwitchMode::Unhighlighted,
                    OptionSwitchMode::Highlighted => OptionSwitchMode::Selected,
                    OptionSwitchMode::Selected => OptionSwitchMode::Highlighted,
                };
                log::info!("toggling option select mode! {:?} -> {:?} and redrawing", old_state_mode, state.mode);
                state.draw(h).await?;
                if state.mode == OptionSwitchMode::Highlighted {
                    Ok(HandlerResult::Unfocus)
                } else {
                    Ok(HandlerResult::None)
                }
            },
            Self::PrintSelection => {
                log::info!("you selected [{}]!", state.definition.options[state.selection][Lang::En]);
                Ok(HandlerResult::None)
            },
            Self::ToggleFocusAndSetTheme => {
                let old_state_mode = state.mode;
                state.mode = match state.mode {
                    OptionSwitchMode::Unhighlighted => OptionSwitchMode::Unhighlighted,
                    OptionSwitchMode::Highlighted => OptionSwitchMode::Selected,
                    OptionSwitchMode::Selected => OptionSwitchMode::Highlighted,
                };
                log::info!("toggling option select mode! {:?} -> {:?} and redrawing", old_state_mode, state.mode);
                state.draw(h).await?;
                if state.mode == OptionSwitchMode::Highlighted {
                    if let MenuInternalState::DisplaySettings { theme_custom_col, .. } = menu_state {
                        let mut settings = PB_GLOBAL_SETTINGS.write().await;
                        settings.theme = match state.selection {
                            0 => Theme::Dark,
                            1 => Theme::Light,
                            2 => Theme::Custom(*theme_custom_col),
                            _ => unreachable!()
                        }
                        // settings.theme = Self::THEMES[self.selection]; // TODO:
                        // todo!();
                    }
                    Ok(HandlerResult::ForceRedrawAndUnfocus)
                } else {
                    Ok(HandlerResult::None)
                }
            },
            Self::ToggleFocusAndSetRGB => {
                let old_state_mode = state.mode;
                state.mode = match state.mode {
                    OptionSwitchMode::Unhighlighted => OptionSwitchMode::Unhighlighted,
                    OptionSwitchMode::Highlighted => OptionSwitchMode::Selected,
                    OptionSwitchMode::Selected => OptionSwitchMode::Highlighted,
                };
                log::info!("toggling option select mode! {:?} -> {:?} and redrawing", old_state_mode, state.mode);
                state.draw(h).await?;
                if let MenuInternalState::RgbMenu { mode, primary_col, secondary_col } = menu_state {
                    if state.mode == OptionSwitchMode::Highlighted {
                        *mode = match state.selection {
                            1 => LedMode::Solid(rgb![0]),
                            2 => LedMode::Fade(rgb![0], rgb![0]),
                            3 => LedMode::Rainbow,
                            _ => LedMode::Off,
                        };
                        let new_mode = match mode {
                            LedMode::Off => LedMode::Off,
                            LedMode::Solid(_) => LedMode::Solid(*primary_col),
                            LedMode::Fade(_, _) => LedMode::Fade(*primary_col, *secondary_col),
                            LedMode::Rainbow => LedMode::Rainbow,
                        };
                        h.leddriver.set_mode(new_mode).await;
                        Ok(HandlerResult::Unfocus)
                    } else {
                        Ok(HandlerResult::None)
                    }
                } else {
                    panic!("bad use of ToggleFocusAndSetRGB option switch action!")
                }
            },
        }
    }
}
// }}}
// OPTION SCROLLER HANDLER {{{
pub enum OptionScrollerHandler {
    Generic(GenericHandler),
    NextOption,
    PrevOption,
    PrintSelection,
    SetMenuState(MenuInternalStateAction),
    TransitionToMenu,
}

impl Handler<OptionScrollerState> for OptionScrollerHandler {
    async fn handle(&self, state: &mut OptionScrollerState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), menu_state, h).await
            },
            Self::NextOption => {
                // log::info!("next selection: {}, PS: {}; [{}]", state.selection, state.page_start, state.definition.options.len());
                let options = state.generated_options(h).await?;
                if options.len() == 0 {
                    return Ok(HandlerResult::None); // TODO: handle this properly!
                }
                if state.selection < options.len() - 1 {
                    state.selection += 1;
                    // if state.selection >= state.page_start + state.definition.num_visible_elements {
                    if state.selection - state.page_start > state.definition.num_visible_elements / 2 &&
                        state.page_start + state.definition.num_visible_elements < options.len() {
                        state.page_start += 1;
                        state.redraw_scrollbar = true;
                    }
                    state.draw(h).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::PrevOption => {
                // log::info!("prev selection: {}, PS: {}; [{}]", state.selection, state.page_start, state.definition.options.len());
                if state.selection > 0 {
                    state.selection -= 1;
                    if state.selection - state.page_start < state.definition.num_visible_elements / 2 &&
                        state.page_start > 0 {
                        state.page_start -= 1;
                        state.redraw_scrollbar = true;
                    }
                    state.draw(h).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::PrintSelection => {
                let options = state.generated_options(h).await?;
                log::info!("scroller menu selection: [{}]", options[state.selection]);
                Ok(HandlerResult::None)
            },
            Self::SetMenuState(a) => {
                match (a, menu_state) {
                    (MenuInternalStateAction::SetLang, MenuInternalState::Lang { language: l, .. }) => {
                        *l = (state.selection % 256) as u8;
                        Ok(HandlerResult::None)
                    },
                    (MenuInternalStateAction::SetWifi, MenuInternalState::Wifi) => {
                        log::info!("wifi selection set to {}!", state.selection);
                        Ok(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::WifiDetails(state.selection))))
                    },
                    _ => { Ok(HandlerResult::None) }
                }
            },
            Self::TransitionToMenu => {
                let (_, menu) = match state.definition.options {
                    OptionsGenerator::Menus(t) => {
                        t[state.selection]
                    }
                    _ => panic!("bad option used with TransitionToMenu handler!"),
                };
                Ok(HandlerResult::Transition(menu))
            },
        }
    }
}
// }}}
// VALUE SELECTOR HANDLER {{{
pub enum ValueSelectorHandler {
    Generic(GenericHandler),
    ToggleFocus,
    Increment(ValueSelectorNumType),
    Decrement(ValueSelectorNumType),
    IncrementBrightness,
    DecrementBrightness,
    IncrementLedBrightness,
    DecrementLedBrightness
}


impl Handler<ValueSelectorState> for ValueSelectorHandler {
    async fn handle(&self, state: &mut ValueSelectorState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => g.handle(&mut (), menu_state, h).await,
            Self::ToggleFocus => {
                let old_state_mode = state.mode;
                state.mode = match state.mode {
                    ValueSelectorMode::Unhighlighted => ValueSelectorMode::Unhighlighted,
                    ValueSelectorMode::Highlighted => ValueSelectorMode::Selected,
                    ValueSelectorMode::Selected => ValueSelectorMode::Highlighted,
                };
                log::info!("toggling value selector mode! {:?} -> {:?} and redrawing", old_state_mode, state.mode);
                state.draw(h).await?;
                if state.mode == ValueSelectorMode::Highlighted {
                    Ok(HandlerResult::Unfocus)
                } else {
                    Ok(HandlerResult::None)
                }
            },
            Self::Increment(u) => {
                if state.selection < state.definition.high_limit {
                    state.selection = state.selection.saturating_add(*u);
                    state.draw(h).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::Decrement(u) => {
                if state.selection > state.definition.low_limit {
                    state.selection = state.selection.saturating_sub(*u);
                    state.draw(h).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::IncrementBrightness => {
                if state.selection < state.definition.high_limit {
                    state.selection = state.selection.saturating_add(1);
                    h.set_brightness_pct((state.selection & 0xff) as u8).map_err(|f| HandlerError::BacklightSetError(f))?;
                    state.draw(h).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::DecrementBrightness => {
                if state.selection > state.definition.low_limit {
                    state.selection = state.selection.saturating_sub(1);
                    h.set_brightness_pct((state.selection & 0xff) as u8).map_err(|f| HandlerError::BacklightSetError(f))?;
                    state.draw(h).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::IncrementLedBrightness => {
                if state.selection < state.definition.high_limit {
                    state.selection = state.selection.saturating_add(1);
                    h.leddriver.set_brightness(state.selection as u8).await;
                    state.draw(h).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::DecrementLedBrightness => {
                if state.selection > state.definition.low_limit {
                    state.selection = state.selection.saturating_sub(1);
                    h.leddriver.set_brightness(state.selection as u8).await;
                    state.draw(h).await?;
                }
                Ok(HandlerResult::None)
            },
        }
    }
}
// }}}
// OPTION SWITCH INITIAL OPTION GENERATOR {{{
pub enum OptionSwitchInitialOptionGenerator {
    Const(usize),
    WifiAuthMethod(&'static [AuthenticationMethod]),
    RGBMode,
}

impl OptionSwitchInitialOptionGenerator {
    pub async fn generate(&self, menu_state: &MenuInternalState, h: &mut IOHandles<'_>) -> usize {
        match self {
            Self::Const(u) => *u,
            Self::WifiAuthMethod(t) => {
                if let MenuInternalState::WifiDetails { ap, .. } = menu_state {
                    t.iter().enumerate().filter(|(i, f)| Some(**f) == ap.auth_method).next().map(|a| a.0).unwrap_or(0)
                } else {
                    panic!("bad use of WifiAuthMethod initial generator!");
                }
            },
            Self::RGBMode => {
                if let MenuInternalState::RgbMenu { mode, .. } = menu_state {
                    match h.leddriver.get_mode().await {
                        LedMode::Off => 0,
                        LedMode::Solid(_) => 1,
                        LedMode::Fade(_, _) => 2,
                        LedMode::Rainbow => 3,
                    }
                } else {
                    panic!("bad use of RGBMode initial generator!");
                }
            },
        }
    }
}
// }}}
// SLIDER BACKGROUND DRAWING {{{
pub enum SliderBackgroundDrawing {
    Fill(RGB),
    GradientY(RGB, RGB),
}

impl SliderBackgroundDrawing {
    pub fn draw(&self, rect: Rectangle, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        match self {
            Self::Fill(c) => {
                rect.into_styled(PrimitiveStyle::with_fill(c.as_rgb565())).draw(&mut h.screen)?;
                Ok(())
            },
            Self::GradientY(a, b) => {
                rect.points().map(|p| {
                    let y_offset = (p.y - rect.top_left.y) as f32;
                    let factor = (y_offset / rect.size.height as f32).clamp(0.0, 1.0);
                    let col = RGB::lerp(a, b, factor);
                    Pixel(p, col.as_rgb565())
                }).draw(&mut h.screen)?;
                Ok(())
            }
        }
    }
}
// }}}
// BUTTON TEXT GENERATOR {{{
pub enum ButtonTextGenerator {
    LangStr(&'static LanguageString),
    WifiConnectDisconnect,
}

impl ButtonTextGenerator {
    pub async fn generate(&self, l: &Lang, h: &mut IOHandles<'_>) -> anyhow::Result<Cow<'static, str>> {
        match self {
            Self::LangStr(s) => Ok(Cow::Borrowed(s[*l])),
            Self::WifiConnectDisconnect => {
                if h.wifi.is_connected() {
                    Ok(Cow::Borrowed(TEXT_DISCONNECT[*l]))
                } else {
                    Ok(Cow::Borrowed(TEXT_CONNECT[*l]))
                }
            }
        }
    }
}
// }}}
// OPTIONS GENERATOR {{{
pub enum OptionsGenerator {
    Const(&'static [&'static str]),
    Menus(&'static [(&'static LanguageString, Menu)]),
    WifiGenerator
}

impl OptionsGenerator {
    pub async fn generate(&self, _: &Lang, h: &mut IOHandles<'_>) -> anyhow::Result<Vec<Cow<'static, str>>> {
        match self {
            Self::Const(s) => Ok(s.iter().map(|f| Cow::Borrowed(*f)).collect()),
            Self::WifiGenerator => {
                let lang = {PB_GLOBAL_SETTINGS.read().await.lang};
                h.wifi.scan().await?;
                let mut list: Vec<Cow<'static, str>> = h.wifi.get_wifis().iter().map(|f| Cow::Owned(f.ssid.as_str().to_string())).collect();
                list.push(Cow::Owned(String::from("<") + TEXT_CUSTOM[lang] + ">"));
                Ok(list)
            },
            Self::Menus(s) => {
                let lang = {&PB_GLOBAL_SETTINGS.read().await.lang};
                Ok(s.iter().map(|(f, _)| Cow::Borrowed(f[*lang])).collect())
            },
        }
    }
}
// }}}
// TEXT GETTER/SETTER (initial owned text for text fields) {{{
pub enum TextGetterSetter {
    Const(&'static str),
    WifiMenuSSIDName,
    WifiMenuPassword,
    ServerIP,
    ServerPort,
}

impl TextGetterSetter {
    pub fn get_owned(&self, menu_def: &ComponentMenuDefinition, state: &MenuInternalState, h: &mut IOHandles<'_>) -> String {
        match (self, menu_def, state) {
            (Self::Const(s), _, _) => {
                String::from(*s)
            },
            (Self::WifiMenuSSIDName, _, MenuInternalState::WifiDetails {ap: a, ..}) => {
                String::from(a.ssid.as_str())
            },
            (Self::WifiMenuPassword, _, _) => {
                String::new()
            },
            (Self::ServerIP, _, _) => {
                ServerConnection::string_from_ip(&h.server.server_ip)
            },
            (Self::ServerPort, _, _) => {
                h.server.server_port.to_string()
            },
            _ => String::from("illegal TextGetterSetter"),
        }
    }
}
// }}}
// TEXT SUBMIT HANDLER {{{
pub enum TextSubmitHandler {
    SetWifiSSID,
    SetWifiPassword,
    SetServerIP,
    SetServerPort,
}

impl Handler<TextBoxState> for TextSubmitHandler {
    async fn handle(&self, state: &mut TextBoxState, menu_state: &mut MenuInternalState, _: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match (self, menu_state) {
            (Self::SetWifiSSID, MenuInternalState::WifiDetails { ap: a, .. }) => {
                // set the wifi details
                a.ssid = Ssid::from(state.current_entry.as_ref().borrow().as_str());
                Ok(HandlerResult::None)
            },
            (Self::SetWifiPassword, MenuInternalState::WifiDetails { pass: p, .. }) => {
                log::info!("SET PASS: {}", state.current_entry.as_ref().borrow());
                p.replace(state.current_entry.as_ref().borrow().to_string());
                Ok(HandlerResult::None)
            },
            (Self::SetServerIP, MenuInternalState::ServerDetails { ip, .. }) => {
                if let Ok(new_ip) = IpAddress::from_str(&state.current_entry.as_ref().borrow()) {
                    *ip = new_ip;
                } else {
                    *state.current_entry.as_ref().borrow_mut() = ServerConnection::string_from_ip(ip);
                }
                Ok(HandlerResult::None)
            },
            (Self::SetServerPort, MenuInternalState::ServerDetails { port, .. }) => {
                if let Ok(new_port) = u16::from_str(&state.current_entry.as_ref().borrow()) {
                    *port = new_port;
                } else {
                    *state.current_entry.as_ref().borrow_mut() = port.to_string();
                }
                Ok(HandlerResult::None)
            },
            _ => {
                log::error!("BAD ACTION");
                Ok(HandlerResult::None)
            }
        }
    }
}
// }}}
// INITIAL VALUE GENERATOR {{{
pub enum InitialValueGenerator {
    Const(ValueSelectorNumType),
    BacklightBrightness,
    LedBrightness,
}

impl InitialValueGenerator {
    pub async fn get(&self, h: &mut IOHandles<'_>) -> ValueSelectorNumType {
        match self {
            Self::Const(s) => *s,
            Self::BacklightBrightness => h.backlight_brightness_pct.into(),
            Self::LedBrightness => h.leddriver.get_brightness().await.into(),
        }
    }
}
// }}}
// SLIDER INITIAL VALUE GETTER {{{
pub enum SliderValueGetter {
    Const(u8),
}

impl SliderValueGetter {
    pub fn get(&self, _: &mut IOHandles<'_>) -> u8 {
        match self {
            Self::Const(v) => *v
        }
    }
}
// }}}
// COLOR GETTER {{{
pub enum ColorGetter {
    Const(RGB),
    ThemeMenuCustomColor,
    LedPrimaryColor,
    LedSecondaryColor,
}

impl ColorGetter {
    pub async fn get(&self, internal_state: &MenuInternalState, h: &mut IOHandles<'_>) -> RGB {
        match self {
            Self::Const(r) => *r,
            Self::ThemeMenuCustomColor => {
                match internal_state {
                    MenuInternalState::DisplaySettings { theme_custom_col, .. } => {
                        *theme_custom_col
                    }
                    _ => panic!("bad ColorGetter::ThemeMenuCustomColor placement!")
                }
            },
            Self::LedPrimaryColor => {
                let mode = h.leddriver.get_mode().await;
                match mode {
                    LedMode::Solid(a) => a,
                    LedMode::Fade(a, _) => a,
                    _ => rgb![128],
                }
            },
            Self::LedSecondaryColor => {
                let mode = h.leddriver.get_mode().await;
                match mode {
                    LedMode::Fade(_, b) => b,
                    _ => rgb![128],
                }
            }
        }
    }
}
// }}}
// COLOR SUBMIT HANDLER {{{
pub enum ColorSubmitHandler {
    Generic(GenericHandler),
    SetThemeMenuColor,
    SetLedPrimaryColor,
    SetLedSecondaryColor,
}

impl Handler<ColorSelectorState> for ColorSubmitHandler {
    async fn handle(&self, state: &mut ColorSelectorState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => g.handle(&mut (), menu_state, h).await,
            Self::SetThemeMenuColor => {
                match menu_state {
                    MenuInternalState::DisplaySettings { theme_custom_col, .. } => {
                        *theme_custom_col = state.current_color;
                        let settings = &mut PB_GLOBAL_SETTINGS.write().await;
                        if let Theme::Custom(_) = settings.theme {
                            settings.theme = Theme::Custom(state.current_color);
                        }
                    }
                    _ => panic!("bad ColorSelectorState::SetThemeMenuHandler handler placement!!"),
                }
                Ok(HandlerResult::None)
            },
            Self::SetLedPrimaryColor => {
                if let MenuInternalState::RgbMenu { mode, primary_col, secondary_col } = menu_state {
                    *primary_col = state.current_color;
                    match mode {
                        LedMode::Off => {},
                        LedMode::Rainbow => {},
                        LedMode::Solid(_) => { h.leddriver.set_mode(LedMode::Solid(state.current_color)).await; },
                        LedMode::Fade(_, _) => { h.leddriver.set_mode(LedMode::Fade(state.current_color, *secondary_col)).await; }
                    }
                    Ok(HandlerResult::None)
                } else {
                    panic!("bad use of SetLedPrimaryColor handler!");
                }
            },
            Self::SetLedSecondaryColor => {
                if let MenuInternalState::RgbMenu { mode, primary_col, secondary_col } = menu_state {
                    *secondary_col = state.current_color;
                    match mode {
                        LedMode::Off => {},
                        LedMode::Rainbow => {},
                        LedMode::Solid(_) => {},
                        LedMode::Fade(_, _) => { h.leddriver.set_mode(LedMode::Fade(*primary_col, state.current_color)).await; }
                    }
                    Ok(HandlerResult::None)
                } else {
                    panic!("bad use of SetLedSecondaryColor handler!");
                }
            }
        }
    }
}
// }}}

// vim:foldmethod=marker
