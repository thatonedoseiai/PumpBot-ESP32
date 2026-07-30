use crate::handlers::{HandlerResult, ButtonHandler, Handler};
use crate::IOHandles;
use crate::screen::screen::{Screen, ScreenDrawError};
use fontfile::{PbFont, pb_font_renderer::PbFontRenderer};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{Rectangle, PrimitiveStyleBuilder, PrimitiveStyle, RoundedRectangle}
};

#[derive(PartialEq)]
pub enum InteractionType {
    Skip,
    NonEditable,
    Editable,
}

pub enum ComponentDefinition {
    Button(&'static ButtonDefinition)
}

pub enum ComponentState {
    Button(ButtonState)
}

pub trait RunHandlers {
    fn left_handle(&mut self, h: &mut IOHandles) -> HandlerResult;
    fn right_handle(&mut self, h: &mut IOHandles) -> HandlerResult;
    fn click_handle(&mut self, h: &mut IOHandles) -> HandlerResult;
}

pub trait ComponentBehaviour {
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, s: &mut D, f: PbFontRenderer) -> Result<(), <D as DrawTarget>::Error>;
    fn highlight(&mut self) -> &mut Self;
    fn unhighlight(&mut self) -> &mut Self;
}

pub struct ButtonState {
    definition: &'static ButtonDefinition,
    highlighted: bool,
    id: u8,
}

impl ComponentBehaviour for ComponentState {
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, s: &mut D, f: PbFontRenderer) -> Result<(), <D as DrawTarget>::Error> {
        match self {
            Self::Button(b) => b.draw(s, f),
        }
    }

    fn highlight(&mut self) -> &mut Self {
        match self {
            Self::Button(b) => b.highlight(),
        };
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        match self {
            Self::Button(b) => b.unhighlight(),
        };
        self
    }
}

const BUTTON_RADIUS: u32 = 10;

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

impl ComponentBehaviour for ButtonState {
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, s: &mut D, f: PbFontRenderer) -> Result<(), <D as DrawTarget>::Error> {
        // println!("DRAWING COMPONENT [{}]", self.id);
        let backing_rectangle = RoundedRectangle::with_equal_corners(
            Rectangle::new(self.definition.pos, self.definition.size),
            Size::new(BUTTON_RADIUS, BUTTON_RADIUS),
        );
        backing_rectangle.into_styled(
            if self.highlighted {
                BUTTON_HIGHLIGHTED_GRAPHIC_STYLE
            } else {
                BUTTON_GRAPHIC_STYLE
            }
        ).draw(s)?;
        Text::with_alignment(self.definition.text, self.definition.pos, f, Alignment::Left).draw(s)?;
        Ok(())
    }

    fn highlight(&mut self) -> &mut Self {
        // println!("HIGHLIGHTING COMPONENT [{}]", self.id);
        self.highlighted = true;
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        // println!("UNHIGHLIGHTING COMPONENT [{}]", self.id);
        self.highlighted = false;
        self
    }
}

pub struct ButtonDefinition {
    pub pos: Point,
    pub size: Size,
    pub click: ButtonHandler,
    pub left: ButtonHandler,
    pub right: ButtonHandler,
    pub text: &'static str,
}

impl RunHandlers for ButtonState {
    fn left_handle(&mut self, h: &mut IOHandles) -> HandlerResult {
        self.definition.left.handle(self, h)
    }

    fn right_handle(&mut self, h: &mut IOHandles) -> HandlerResult {
        self.definition.right.handle(self, h)
    }

    fn click_handle(&mut self, h: &mut IOHandles) -> HandlerResult {
        self.definition.click.handle(self, h)
    }
}

impl RunHandlers for ComponentState {
    fn left_handle(&mut self, h: &mut IOHandles) -> HandlerResult {
        match self {
            Self::Button(b) => b.left_handle(h),
        }
    }

    fn right_handle(&mut self, h: &mut IOHandles) -> HandlerResult {
        match self {
            Self::Button(b) => b.right_handle(h),
        }
    }

    fn click_handle(&mut self, h: &mut IOHandles) -> HandlerResult {
        match self {
            Self::Button(b) => b.click_handle(h),
        }
    }
}

impl ComponentState {
    pub const fn definition(&self) -> ComponentDefinition {
        match self {
            Self::Button(b) => ComponentDefinition::Button(b.definition)
        }
    }
}

impl ComponentDefinition {
    pub fn construct(&self) -> ComponentState {
        static mut ID: u8 = 0;
        unsafe { ID = ID + 1 };
        match self {
            Self::Button(definition) => ComponentState::Button(ButtonState { definition, highlighted: false, id: unsafe {ID} }),
        }
    }

    pub const fn interaction_type(&self) -> InteractionType {
        match self {
            Self::Button(_) => InteractionType::NonEditable,
        }
    }
}