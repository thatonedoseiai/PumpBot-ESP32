use crate::components::{ButtonState, OptionSwitchState, OptionSwitchMode, ComponentBehaviour, OptionScrollerState};
use crate::{ComponentMenuInAction, Menu, IOHandles};

#[derive(Debug, Clone, Copy)]
pub enum HandlerResult {
    None,
    Transition(Menu),
    Back,
    Unfocus,
}

pub trait Handler<S> {
    async fn handle(&self, state: &mut S, _: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
}

pub enum GenericHandler {
    Print(&'static str),
    Signal(HandlerResult),
}

impl Handler<()> for GenericHandler {
    async fn handle(&self, state: &mut (), h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
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
    async fn handle(&self, state: &mut ComponentMenuInAction, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), h).await
            }
        }
    }
}

pub enum ButtonHandler {
    Generic(GenericHandler),
}

impl Handler<ButtonState> for ButtonHandler {
    async fn handle(&self, state: &mut ButtonState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), h).await
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
    async fn handle(&self, state: &mut OptionSwitchState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), h).await
            },
            Self::NextElement => {
                state.selection = (state.selection + 1) % state.definition.options.len();
                state.draw(&mut h.screen, &mut h.font).await?;
                Ok(HandlerResult::None)
            },
            Self::PrevElement => {
                let num_options = state.definition.options.len();
                state.selection = (state.selection + num_options - 1) % num_options;
                state.draw(&mut h.screen, &mut h.font).await?;
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
                state.draw(&mut h.screen, &mut h.font).await?;
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
}

impl Handler<OptionScrollerState> for OptionScrollerHandler {
    async fn handle(&self, state: &mut OptionScrollerState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), h).await
            },
            Self::NextOption => {
                log::info!("next selection: {}, PS: {}; [{}]", state.selection, state.page_start, state.definition.options.len());
                if state.selection < state.definition.options.len() - 1 {
                    state.selection += 1;
                    if state.selection >= state.page_start + state.definition.num_visible_elements {
                        state.page_start += 1;
                        state.redraw_scrollbar = true;
                    }
                    state.draw(&mut h.screen, &mut h.font).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::PrevOption => {
                log::info!("prev selection: {}, PS: {}; [{}]", state.selection, state.page_start, state.definition.options.len());
                if state.selection > 0 {
                    state.selection -= 1;
                    if state.selection < state.page_start {
                        state.page_start -= 1;
                        state.redraw_scrollbar = true;
                    }
                    state.draw(&mut h.screen, &mut h.font).await?;
                }
                Ok(HandlerResult::None)
            },
            Self::PrintSelection => {
                log::info!("scroller menu selection: [{}]", state.definition.options[state.selection]);
                Ok(HandlerResult::None)
            }
        }
    }
}
