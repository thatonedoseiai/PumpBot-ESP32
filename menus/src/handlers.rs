use crate::components::{ButtonState, OptionSwitchState, OptionSwitchMode, ComponentBehaviour, OptionScrollerState};
use crate::{ComponentMenuInAction, Menu, IOHandles, MenuInternalState};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use global_settings::{lang::Lang, PB_GLOBAL_SETTINGS};
use alloc::string::ToString;

pub enum MenuInternalStateAction {
    SetLang,
}

#[derive(Debug, Clone, Copy)]
pub enum HandlerResult {
    None,
    Transition(Menu),
    Back,
    Unfocus,
}

pub trait Handler<S> {
    async fn handle(&self, state: &mut S, menu_state: &mut MenuInternalState, _: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
}

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
                Ok(*s)
            }
        }
    }
}

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

pub enum ButtonHandler {
    Generic(GenericHandler),
}

impl Handler<ButtonState> for ButtonHandler {
    async fn handle(&self, state: &mut ButtonState, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), menu_state, h).await
            }
        }
    }
}

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
                log::info!("you selected [{}]!", state.definition.options[state.selection]);
                Ok(HandlerResult::None)
            }
        }
    }
}

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
                let options = state.generated_options.get_or_try_init(async {
                    state.definition.options.generate(&PB_GLOBAL_SETTINGS.read().await.lang, h).await
                }).await?;
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
                    // if state.selection < state.page_start {
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
                let options = state.generated_options.get_or_try_init(async {
                    state.definition.options.generate(&PB_GLOBAL_SETTINGS.read().await.lang, h).await
                }).await?;
                log::info!("scroller menu selection: [{}]", options[state.selection]);
                Ok(HandlerResult::None)
            },
            Self::SetMenuState(a) => {
                match (a, menu_state) {
                    (MenuInternalStateAction::SetLang, MenuInternalState::Lang { language: l, .. }) => {
                        *l = (state.selection % 256) as u8;
                        Ok(HandlerResult::None)
                    },
                    _ => { Ok(HandlerResult::None) }
                }
            }
        }
    }
}

pub enum OptionsGenerator {
    Const(&'static [&'static str]),
    WifiGenerator
}

impl OptionsGenerator {
    pub async fn generate(&self, _: &Lang, h: &mut IOHandles<'_>) -> anyhow::Result<Vec<Cow<'static, str>>> {
        match self {
            Self::Const(s) => Ok(s.iter().map(|f| Cow::Borrowed(*f)).collect()),
            Self::WifiGenerator => {
                Ok(h.wifi.get_wifis().await?.into_iter().map(|f| Cow::Owned(f.ssid.as_str().to_string())).collect())
            },
        }
    }
}
