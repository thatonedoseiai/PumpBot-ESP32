use button_idf::ButtonType;
use embassy_futures::select::{Either, select};
use rotenc::Direction;
use crate::{
    components,
    MenuSignal,
    IOHandles,
    MenuBehaviour,
    components::button_component::{ButtonComponent, ButtonComponentError},
};
use embassy_executor::Spawner;
use embedded_graphics::{
    prelude::DrawTarget,
    pixelcolor::Rgb565,
};
use core::fmt;
use core::borrow::BorrowMut;
use fontfile::pb_font_renderer::PbFontRenderer;
use core::pin::Pin;
use alloc::boxed::Box;
use alloc::vec::Vec;

type PinnedFuture = Pin<Box<dyn Future<Output = Result<ComponentSignal, ComponentMenuError>>>>;

components![
    ButtonComponent -> ButtonComponentError,
];

pub enum ComponentSignal {
    MenuSignal(MenuSignal),
    Focus,
    Unfocus,
    None
}

pub trait MenuComponent {
    type Error;

    fn draw<D: DrawTarget<Color = Rgb565>>(&self, f: PbFontRenderer, d: &mut D) -> impl core::future::Future<Output = Result<(), <D as DrawTarget>::Error>>;
    fn position(&self) -> impl core::future::Future<Output = (u8, u8)>;
    fn highlight(&self) -> impl core::future::Future<Output = &Self>;
    fn unhighlight(&self) -> impl core::future::Future<Output = &Self>;
    fn click(&self, io_handles: &mut IOHandles<'_>) -> impl core::future::Future<Output = Result<ComponentSignal, Self::Error>>;
    fn right(&self, io_handles: &mut IOHandles<'_>) -> impl core::future::Future<Output = &Self>;
    fn left(&self, io_handles: &mut IOHandles<'_>) -> impl core::future::Future<Output = &Self>;
    fn receive_signal(&self, sig: ComponentSignal) -> impl core::future::Future<Output = ()>;
}

#[derive(Debug)]
pub enum ComponentMenuError {
    Error
}

impl fmt::Display for ComponentMenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentMenuError::Error => write!(f, "Error!")
        }
    }
}
impl core::error::Error for ComponentMenuError {}

pub trait ComponentCombinedMenuButtonBehaviours {
    async fn left_behaviour(h: &mut IOHandles) -> Result<ComponentSignal, ComponentMenuError>;
    async fn right_behaviour(h: &mut IOHandles) -> Result<ComponentSignal, ComponentMenuError>;
}

pub struct ComponentCombinedMenu<B: ComponentCombinedMenuButtonBehaviours> {
    components: Vec<Components>,
    _marker: core::marker::PhantomData<B>
}

impl<B: ComponentCombinedMenuButtonBehaviours> ComponentCombinedMenu<B> {
    pub const fn new(components: Vec<Components>) -> Self {
        ComponentCombinedMenu {
            components,
            _marker: core::marker::PhantomData
        }
    }
}

enum MenuMode {
    Highlight,
    Focus,
}

impl<B: ComponentCombinedMenuButtonBehaviours> MenuBehaviour for ComponentCombinedMenu<B> {
    async fn init(&mut self, spawner: Spawner, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        // self.components.iter().try_for_each(|f| f.draw(io_handles.screen.borrow_mut()))?;
        Ok(MenuSignal::None)
    }

    async fn update(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let mut focused = 0;
        let mut mode = if self.components.len() > 1 {
            self.components[focused].highlight().await;
            MenuMode::Highlight
        } else {
            MenuMode::Focus
        };
        for f in self.components.iter() {
            f.draw(io_handles.font.clone(), io_handles.screen.borrow_mut()).await?;
        }

        loop {
            let res = select(
                io_handles.rotenc.receive(),
                io_handles.button.receive(),
            ).await;
            match res {
                Either::First(r) => {
                    match r.dir {
                        Direction::Clockwise => {
                            match mode {
                                MenuMode::Highlight => {
                                    self.components[focused].unhighlight().await.draw(io_handles.font.clone(), io_handles.screen.borrow_mut()).await?;
                                    focused = (focused + 1) % self.components.len();
                                    self.components[focused].highlight().await.draw(io_handles.font.clone(), io_handles.screen.borrow_mut()).await?;
                                }
                                MenuMode::Focus => {
                                    self.components[focused].right(io_handles).await;
                                }
                            }
                        },
                        Direction::Anticlockwise => {
                            match mode {
                                MenuMode::Highlight => {
                                    self.components[focused].unhighlight().await.draw(io_handles.font.clone(), io_handles.screen.borrow_mut()).await?;
                                    focused = (focused + self.components.len() - 1) % self.components.len();
                                    self.components[focused].highlight().await.draw(io_handles.font.clone(), io_handles.screen.borrow_mut()).await?;
                                }
                                MenuMode::Focus => {
                                    self.components[focused].left(io_handles).await;
                                }
                            }
                        },
                        _ => {}
                    }
                },
                Either::Second(b) => {
                    let signal = match b.button_type {
                        ButtonType::Right => {
                            // right button behaviour
                            B::right_behaviour(io_handles).await?
                        },
                        ButtonType::Left => {
                            // left button behaviour
                            B::left_behaviour(io_handles).await?
                        },
                        ButtonType::Rotenc => {
                            self.components[focused].click(io_handles).await?
                            // rotenc
                        }
                    };
                    match signal {
                        ComponentSignal::Focus => { mode = MenuMode::Focus; },
                        ComponentSignal::Unfocus => { mode = MenuMode::Highlight; },
                        ComponentSignal::MenuSignal(m) => { return Ok(m); },
                        ComponentSignal::None => {},
                    };
                }
            }
        }
    }
}