use crate::handlers::{HandlerResult, ButtonHandler, Handler, OptionSwitchHandler};
use crate::IOHandles;
use crate::screen::screen::{Screen, ScreenDrawError};
use fontfile::{PbFont, pb_font_renderer::PbFontRenderer, FontSize, FontFileError};
use global_settings::{rgb::RGB, rgb, PB_GLOBAL_SETTINGS, Theme};
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
use core::fmt;

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
    async fn draw(&mut self, s: &mut Screen<'_>, f: &mut PbFontRenderer) -> anyhow::Result<()>;
    fn highlight(&mut self) -> &mut Self;
    fn unhighlight(&mut self) -> &mut Self;
}

impl ComponentBehaviour for ComponentState {
    async fn draw(&mut self, s: &mut Screen<'_>, f: &mut PbFontRenderer) -> anyhow::Result<()> {
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

impl ButtonState {
    const RADIUS: u32 = 5;
    const BORDER_SIZE: u32 = 10;

    fn graphic_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_width(3)
            .stroke_color(theme.fg().as_rgb565())
            .fill_color(theme.bg().as_rgb565())
            .build()
    }

    const fn highlighted_graphic_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_width(3)
            .stroke_color(theme.highlight().as_rgb565())
            .fill_color(theme.bg().as_rgb565())
            .build()
    }
}

impl ComponentBehaviour for ButtonState {
    async fn draw(&mut self, s: &mut Screen<'_>, f: &mut PbFontRenderer) -> anyhow::Result<()> {
        // println!("DRAWING COMPONENT [{}]", self.id);
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        f.fgcol = theme.fg();
        f.bgcol = theme.bg();
        f.font.borrow_mut().set_size(self.definition.font_size).unwrap();
        let button_text = Text::with_alignment(self.definition.text, self.definition.pos, &*f, Alignment::Left);
        let text_bb = button_text.bounding_box();
        let backing_rectangle = RoundedRectangle::with_equal_corners(
            // Rectangle::new(self.definition.pos, bounding_box_size + Size::new(BUTTON_BORDER_SIZE, BUTTON_BORDER_SIZE)),
            text_bb.resized(text_bb.size + Size::new(Self::BORDER_SIZE, Self::BORDER_SIZE), AnchorPoint::Center),
            Size::new(Self::RADIUS, Self::RADIUS),
        );
        backing_rectangle.into_styled(
            if self.highlighted {
                Self::highlighted_graphic_style(theme)
            } else {
                Self::graphic_style(theme)
            }
        ).draw(s)?;
        button_text.draw(s)?;
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
    pub click: ButtonHandler,
    pub left: ButtonHandler,
    pub right: ButtonHandler,
    pub font_size: FontSize,
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
    pub font_size: FontSize,
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

impl OptionSwitchState {
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

    const fn cursor_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(theme.highlight().as_rgb565())
            .build()
    }

    const fn undraw_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg().as_rgb565())
            .build()
    }
}

impl ComponentBehaviour for OptionSwitchState {
    async fn draw(&mut self, s: &mut Screen<'_>, f: &mut PbFontRenderer) -> anyhow::Result<()> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        f.bgcol = theme.bg();
        f.font.borrow_mut().set_size(self.definition.font_size)?;
        if let Some(boxes) = self.text_undraw {
            boxes.into_styled(Self::undraw_style(theme)).draw(s)?;
            self.text_undraw = None;
        }
        if let Some(c_undraws) = self.cursor_undraws {
            c_undraws.iter().try_for_each(|k| k.into_styled(Self::undraw_style(theme)).draw(s))?;
            self.cursor_undraws = None;
        }
        match self.mode {
            OptionSwitchMode::Unhighlighted => {
                f.fgcol = theme.fg();
                Text::with_alignment(self.definition.options[self.selection], self.definition.pos, &*f, Alignment::Left).draw(s)?;
            },
            OptionSwitchMode::Highlighted => {
                f.fgcol = theme.highlight();
                Text::with_alignment(self.definition.options[self.selection], self.definition.pos, &*f, Alignment::Left).draw(s)?;
            },
            OptionSwitchMode::Selected => {
                f.fgcol = theme.highlight();
                let text = Text::with_alignment(self.definition.options[self.selection], self.definition.pos, &*f, Alignment::Left);
                let text_bb = text.bounding_box();
                let left_coord = text_bb.top_left + Size::new(0, text_bb.size.height / 2);
                let right_coord = left_coord + Size::new(text_bb.size.width, 0);
                let left_cursor = Self::OPTION_SWITCH_LEFT_CURSOR.translate(left_coord);
                let right_cursor = Self::OPTION_SWITCH_RIGHT_CURSOR.translate(right_coord);
                left_cursor.into_styled(Self::cursor_style(theme)).draw(s)?;
                right_cursor.into_styled(Self::cursor_style(theme)).draw(s)?;
                text.draw(s)?;
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