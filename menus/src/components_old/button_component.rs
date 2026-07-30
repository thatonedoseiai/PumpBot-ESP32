use crate::menus::component_combined_menu::{MenuComponent, ComponentSignal};
use crate::IOHandles;
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{Rectangle, PrimitiveStyleBuilder, PrimitiveStyle, RoundedRectangle}
};
use core::error::Error;
use core::fmt;
use fontfile::pb_font_renderer::PbFontRenderer;
use core::pin::Pin;
use alloc::boxed::Box;
use core::cell::RefCell;

const BUTTON_RADIUS: u32 = 10;

type PinnedFuture = Pin<Box<dyn Future<Output = Result<ComponentSignal, ButtonComponentError>>>>;

const BUTTON_GRAPHIC_STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .stroke_width(5)
    .stroke_color(Rgb565::RED)
    .fill_color(Rgb565::GREEN)
    .build();

const BUTTON_HIGHLIGHTED_GRAPHIC_STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .stroke_width(5)
    .stroke_color(Rgb565::GREEN)
    .fill_color(Rgb565::RED)
    .build();


pub struct ButtonComponent {
    graphics: RoundedRectangle,
    button_style: RefCell<&'static PrimitiveStyle<Rgb565>>,
    text: &'static str,
    onclick: fn(&mut IOHandles<'_>) -> PinnedFuture,
}

impl ButtonComponent {
    pub const fn new(top_left: Point, size: Size, text: &'static str, onclick: fn(&mut IOHandles<'_>) -> PinnedFuture) -> Self
    {
        let graphics = RoundedRectangle::with_equal_corners(
            Rectangle::new(top_left, size),
            Size::new(BUTTON_RADIUS, BUTTON_RADIUS),
        );
        // let text = Text::with_alignment(words, top_left, font, Alignment::Left);
        let button_style = RefCell::new(&BUTTON_GRAPHIC_STYLE);
        ButtonComponent {
            graphics,
            button_style,
            text,
            onclick
        }
    }
}

impl MenuComponent for ButtonComponent {
    type Error = ButtonComponentError;

    async fn draw<D: DrawTarget<Color = Rgb565>>(&self, f: PbFontRenderer, d: &mut D) -> Result<(), <D as DrawTarget>::Error> {
        // todo!()
        self.graphics.into_styled(self.button_style.borrow().clone()).draw(d)?;
        Text::with_alignment(self.text, self.graphics.rectangle.top_left, f, Alignment::Left).draw(d)?;
        Ok(())
    }

    async fn position(&self) -> (u8, u8) {
        todo!()
    }

    async fn highlight(&self) -> &Self {
        // todo!()
        self.button_style.replace(&BUTTON_HIGHLIGHTED_GRAPHIC_STYLE);
        &self
    }

    async fn unhighlight(&self) -> &Self {
        // todo!()
        self.button_style.replace(&BUTTON_GRAPHIC_STYLE);
        &self
    }

    async fn click(&self, h: &mut IOHandles<'_>) -> Result<ComponentSignal, Self::Error> {
        // todo!()
        (self.onclick)(h).await
    }

    async fn right(&self, h: &mut IOHandles<'_>) -> &Self {
        // todo!()
        // DO NOTHING
        &self
    }

    async fn left(&self, h: &mut IOHandles<'_>) -> &Self {
        // todo!()
        // DO NOTHING
        &self
    }

    async fn receive_signal(&self, c: ComponentSignal) -> () {
        // todo!()
        () // DO NOTHING
    }
}

#[derive(Debug)]
pub struct ButtonComponentError {

}

impl Error for ButtonComponentError { }

impl fmt::Display for ButtonComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Button Error")
    }
}
