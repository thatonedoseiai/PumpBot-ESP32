use crate::components::{ButtonState, OptionSwitchState, OptionSwitchMode, ComponentBehaviour, OptionScrollerState, TextBoxState};
use crate::{ComponentMenu, ComponentMenuInAction, Menu, IOHandles, MenuInternalState, ComponentMenuDefinition};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use global_settings::{lang::Lang, PB_GLOBAL_SETTINGS};
use alloc::string::{ToString, String};
use esp_radio::wifi::{Ssid, WifiError};
use core::str::FromStr;
use embassy_net::IpAddress;
use socket::ServerConnection;

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
}

pub trait Handler<S> {
    async fn handle(&self, state: &mut S, menu_state: &mut MenuInternalState, _: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
}

// GENERIC {{{
pub enum GenericHandler {
    Print(&'static str),
    Signal(HandlerResult),
}

impl Handler<()> for GenericHandler {
    async fn handle(&self, state: &mut (), menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Print(s) => {
                // println!("{s}");
                log::info!("{s}");
                Ok(HandlerResult::None)
            },
            Self::Signal(s) => {
                // println!("sending {:?}", s);
                Ok(s.clone())
            }
        }
    }
}
// }}}
// MENU HANDLER {{{
pub enum MenuHandler {
    Generic(GenericHandler)
}

impl Handler<ComponentMenuInAction> for MenuHandler {
    async fn handle(&self, state: &mut ComponentMenuInAction, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
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
    ConnectWifi,
    ConnectServer,
}

impl Handler<ButtonState> for ButtonHandler {
    async fn handle(&self, state: &mut ButtonState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), menu_state, h).await
            },
            Self::ConnectWifi => {
                if let MenuInternalState::WifiDetails {
                    ap: a,
                    pass: p
                } = menu_state {
                    let pw = p.borrow();
                    log::warn!("Connecting to wifi: SSID {}, pass {}", a.ssid.as_str(), &pw);
                    let wifi_res = h.wifi.connect(&a, &pw).await;
                    match wifi_res {
                        Ok(()) => {
                            log::info!("connect returned OK!");
                            Ok(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::ServerDetails)))
                        },
                        Err(e) => Ok(HandlerResult::WifiConnectionFailure(e))
                    }
                } else {
                    unreachable!();
                }
                // Ok(HandlerResult::None)
                // todo!("ConnectWifi button handler not implemented yet!")
            },
            Self::ConnectServer => {
                let res = h.server.connect().await;
                match res {
                    Ok(h) => {
                        log::info!("pb connected to server!");
                        Ok(HandlerResult::None)
                    },
                    Err(e) => {
                        log::info!("pb failed to connect!");
                        Ok(HandlerResult::None)
                    }
                }
            }
        }
    }
}
// }}}
// OPTION SWITCH HANDLER {{{
pub enum OptionSwitchHandler {
    Generic(GenericHandler),
    NextElement,
    PrevElement,
    ToggleFocus,
    PrintSelection,
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
            }
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
    SetMenuState(MenuInternalStateAction)
}

impl Handler<OptionScrollerState> for OptionScrollerHandler {
    async fn handle(&self, state: &mut OptionScrollerState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), menu_state, h).await
            },
            Self::NextOption => {
                // log::info!("next selection: {}, PS: {}; [{}]", state.selection, state.page_start, state.definition.options.len());
                let lang = &PB_GLOBAL_SETTINGS.read().await.lang;
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
            }
        }
    }
}
// }}}
// OPTIONS GENERATOR {{{
pub enum OptionsGenerator {
    Const(&'static [&'static str]),
    WifiGenerator
}

impl OptionsGenerator {
    pub async fn generate(&self, _: &Lang, h: &mut IOHandles<'_>) -> anyhow::Result<Vec<Cow<'static, str>>> {
        match self {
            Self::Const(s) => Ok(s.iter().map(|f| Cow::Borrowed(*f)).collect()),
            Self::WifiGenerator => {
                h.wifi.scan().await?;
                Ok(h.wifi.get_wifis().iter().map(|f| Cow::Owned(f.ssid.as_str().to_string())).collect())
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
    async fn handle(&self, state: &mut TextBoxState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
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

// vim:foldmethod=marker
