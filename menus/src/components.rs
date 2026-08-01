use crate::handlers::{HandlerResult, ButtonHandler, Handler, OptionSwitchHandler};
use crate::IOHandles;
use crate::screen::screen::{Screen, ScreenDrawError};
use fontfile::{PbFont, pb_font_renderer::PbFontRenderer};
use global_settings::{rgb::RGB, rgb, PB_GLOBAL_SETTINGS};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{
        Rectangle, 
        PrimitiveStyleBuilder, 
        PrimitiveStyle, 
        RoundedRectangle,
        Triangle
    },
    geometry::AnchorPoint,
};

#[derive(PartialEq)]
pub enum InteractionType {
    Skip,
    NonEditable,
    Editable,
}

pub enum ComponentDefinition {
    Button(&'static ButtonDefinition),
    OptionSwitch(&'static OptionSwitchDefinition),
}

pub enum ComponentState {
    Button(ButtonState),
    OptionSwitch(OptionSwitchState),
}

pub trait RunHandlers {
    async fn left_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
    async fn right_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
    async fn click_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
}

pub trait ComponentBehaviour {
    async fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, s: &mut D, f: PbFontRenderer) -> Result<(), <D as DrawTarget>::Error>;
    fn highlight(&mut self) -> &mut Self;
    fn unhighlight(&mut self) -> &mut Self;
}

impl ComponentBehaviour for ComponentState {
    async fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, s: &mut D, mut f: PbFontRenderer) -> Result<(), <D as DrawTarget>::Error> {
        match self {
            Self::Button(b) => b.draw(s, f).await,
            Self::OptionSwitch(b) => b.draw(s, f).await,
        }
    }

    fn highlight(&mut self) -> &mut Self {
        match self {
            Self::Button(b) => { b.highlight(); },
            Self::OptionSwitch(b) => { b.highlight(); },
        }
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        match self {
            Self::Button(b) => { b.unhighlight(); },
            Self::OptionSwitch(b) => { b.unhighlight(); },
        }
        self
    }
}

impl RunHandlers for ComponentState {
    async fn left_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.left_handle(h).await,
            Self::OptionSwitch(b) => b.left_handle(h).await,
        }
    }

    async fn right_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.right_handle(h).await,
            Self::OptionSwitch(b) => b.right_handle(h).await,
        }
    }

    async fn click_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.click_handle(h).await,
            Self::OptionSwitch(b) => b.click_handle(h).await,
        }
    }
}

impl ComponentState {
    pub const fn definition(&self) -> ComponentDefinition {
        match self {
            Self::Button(b) => ComponentDefinition::Button(b.definition),
            Self::OptionSwitch(b) => ComponentDefinition::OptionSwitch(b.definition),
        }
    }
}

impl ComponentDefinition {
    pub fn construct(&self) -> ComponentState {
        match self {
            Self::Button(definition) => ComponentState::Button(ButtonState { definition, highlighted: false }),
            Self::OptionSwitch(definition) => ComponentState::OptionSwitch(OptionSwitchState { 
                definition, 
                mode: OptionSwitchMode::Unhighlighted, 
                selection: 0, 
                text_undraw: None,
                cursor_undraws: None,
            }),
        }
    }

    pub const fn interaction_type(&self) -> InteractionType {
        match self {
            Self::Button(_) => InteractionType::NonEditable,
            Self::OptionSwitch(_) => InteractionType::Editable,
        }
    }
}


// BUTTON {{{
pub struct ButtonState {
    definition: &'static ButtonDefinition,
    highlighted: bool,
}

const BUTTON_RADIUS: u32 = 10;
const BUTTON_BORDER_SIZE: u32 = 5;

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
    async fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, s: &mut D, mut f: PbFontRenderer) -> Result<(), <D as DrawTarget>::Error> {
        // println!("DRAWING COMPONENT [{}]", self.id);
        let button_text = Text::with_alignment(self.definition.text, self.definition.pos, f, Alignment::Left);
        let text_bb = button_text.bounding_box();
        let backing_rectangle = RoundedRectangle::with_equal_corners(
            // Rectangle::new(self.definition.pos, bounding_box_size + Size::new(BUTTON_BORDER_SIZE, BUTTON_BORDER_SIZE)),
            text_bb.resized(text_bb.size + Size::new(BUTTON_BORDER_SIZE, BUTTON_BORDER_SIZE), AnchorPoint::Center),
            Size::new(BUTTON_RADIUS, BUTTON_RADIUS),
        );
        backing_rectangle.into_styled(
            if self.highlighted {
                BUTTON_HIGHLIGHTED_GRAPHIC_STYLE
            } else {
                BUTTON_GRAPHIC_STYLE
            }
        ).draw(s)?;
        button_text.draw(s)?;
        // todo!("make button width calculated based on text width");
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
    // pub size: Size,
    pub click: ButtonHandler,
    pub left: ButtonHandler,
    pub right: ButtonHandler,
    pub text: &'static str,
}

impl RunHandlers for ButtonState {
    async fn left_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.left.handle(self, h).await
    }

    async fn right_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.right.handle(self, h).await
    }

    async fn click_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.click.handle(self, h).await
    }
}
// }}}
// OPTION SWITCH {{{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OptionSwitchMode {
    Unhighlighted,
    Highlighted,
    Selected
}

pub struct OptionSwitchState {
    pub definition: &'static OptionSwitchDefinition,
    pub mode: OptionSwitchMode,
    pub selection: usize,
    text_undraw: Option<Rectangle>,
    cursor_undraws: Option<[Rectangle; 2]>,
}

pub struct OptionSwitchDefinition {
    pub pos: Point,
    pub click: OptionSwitchHandler,
    pub left: OptionSwitchHandler,
    pub right: OptionSwitchHandler,
    pub options: &'static [&'static str],
}

impl RunHandlers for OptionSwitchState {
    async fn left_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.left.handle(self, h).await
    }

    async fn right_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.right.handle(self, h).await
    }

    async fn click_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.click.handle(self, h).await
    }
}

const OPTION_SWITCH_CURSOR_STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .stroke_width(1)
    .stroke_color(Rgb565::RED)
    .build();

const OPTION_SWITCH_HIGHLIGHTED_TEXT_COLOR: RGB = rgb![255, 0, 0];
const OPTION_SWITCH_DEFAULT_TEXT_COLOR: RGB = rgb![255, 255, 255];

const OPTION_SWITCH_LEFT_CURSOR: Triangle = Triangle::new(
                    Point::new(-10, 5),
                    Point::new(-5, 10),
                    Point::new(-5, 0),
                );

const OPTION_SWITCH_RIGHT_CURSOR: Triangle = Triangle::new(
                    Point::new(10, 5),
                    Point::new(5, 10),
                    Point::new(5, 0),
                );

const OPTION_SWITCH_UNDRAW_STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .fill_color(Rgb565::BLUE)
    .build();
 

impl ComponentBehaviour for OptionSwitchState {
    async fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, s: &mut D, mut f: PbFontRenderer) -> Result<(), <D as DrawTarget>::Error> {
        if let Some(boxes) = self.text_undraw {
            boxes.into_styled(OPTION_SWITCH_UNDRAW_STYLE).draw(s)?;
            self.text_undraw = None;
        }
        if let Some(c_undraws) = self.cursor_undraws {
            c_undraws.iter().try_for_each(|k| k.into_styled(OPTION_SWITCH_UNDRAW_STYLE).draw(s))?;
            self.cursor_undraws = None;
        }
        match self.mode {
            OptionSwitchMode::Unhighlighted => {
                let old_fgcol = f.fgcol;
                f.fgcol = OPTION_SWITCH_DEFAULT_TEXT_COLOR;
                Text::with_alignment(self.definition.options[self.selection], self.definition.pos, f.clone(), Alignment::Left).draw(s)?;
                f.fgcol = old_fgcol;
            },
            OptionSwitchMode::Highlighted => {
                let old_fgcol = f.fgcol;
                f.fgcol = OPTION_SWITCH_HIGHLIGHTED_TEXT_COLOR;
                Text::with_alignment(self.definition.options[self.selection], self.definition.pos, f.clone(), Alignment::Left).draw(s)?;
                f.fgcol = old_fgcol;
            },
            OptionSwitchMode::Selected => {
                let old_fgcol = f.fgcol;
                f.fgcol = OPTION_SWITCH_HIGHLIGHTED_TEXT_COLOR;
                let text = Text::with_alignment(self.definition.options[self.selection], self.definition.pos, f.clone(), Alignment::Left);
                let text_bb = text.bounding_box();
                let left_coord = text_bb.top_left + Size::new(0, text_bb.size.height / 2);
                let right_coord = left_coord + Size::new(text_bb.size.width, 0);
                let left_cursor = OPTION_SWITCH_LEFT_CURSOR.translate(left_coord);
                let right_cursor = OPTION_SWITCH_RIGHT_CURSOR.translate(right_coord);
                left_cursor.into_styled(OPTION_SWITCH_CURSOR_STYLE).draw(s)?;
                right_cursor.into_styled(OPTION_SWITCH_CURSOR_STYLE).draw(s)?;
                text.draw(s)?;
                f.fgcol = old_fgcol;
                self.text_undraw = Some(text_bb);
                self.cursor_undraws = Some([
                    left_cursor.bounding_box(),
                    right_cursor.bounding_box(),
                ]);
            },
        }
        Ok(())
    }

    fn highlight(&mut self) -> &mut Self {
        if self.mode == OptionSwitchMode::Unhighlighted {
            self.mode = OptionSwitchMode::Highlighted;
        }
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        if self.mode == OptionSwitchMode::Highlighted {
            self.mode = OptionSwitchMode::Unhighlighted;
        }
        self
    }
}
// }}}

// vim:foldmethod=marker