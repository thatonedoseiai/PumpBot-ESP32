use crate::{ButtonState, ComponentMenuInAction, Menu, IOHandles};

#[derive(Debug, Clone, Copy)]
pub enum HandlerResult {
    None,
    Transition(Menu),
    Back,
    Unfocus,
}

pub trait Handler<S> {
    fn handle(&self, state: &mut S, _: &mut IOHandles) -> HandlerResult;
}

pub enum GenericHandler {
    Print(&'static str),
    Signal(HandlerResult),
}

impl Handler<()> for GenericHandler {
    fn handle(&self, state: &mut (), h: &mut IOHandles) -> HandlerResult {
        match self {
            Self::Print(s) => {
                // println!("{s}");
                log::info!("{s}");
                HandlerResult::None
            },
            Self::Signal(s) => {
                // println!("sending {:?}", s);
                *s
            }
        }
    }
}

pub enum MenuHandler {
    Generic(GenericHandler)
}

impl Handler<ComponentMenuInAction> for MenuHandler {
    fn handle(&self, state: &mut ComponentMenuInAction, h: &mut IOHandles) -> HandlerResult {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), h)
            }
        }
    }
}

pub enum ButtonHandler {
    Generic(GenericHandler),
}

impl Handler<ButtonState> for ButtonHandler {
    fn handle(&self, state: &mut ButtonState, h: &mut IOHandles) -> HandlerResult {
        match self {
            Self::Generic(g) => {
                g.handle(&mut (), h)
            }
        }
    }
}
