use crate::handlers::{HandlerResult, ButtonHandler, Handler, OptionSwitchHandler, OptionScrollerHandler};
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
    OptionScroller(&'static OptionScrollerDefinition),
}

pub enum ComponentState {
    Button(ButtonState),
    OptionSwitch(OptionSwitchState),
    OptionScroller(OptionScrollerState),
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
            Self::OptionScroller(b) => b.draw(s, f).await,
        }
    }

    fn highlight(&mut self) -> &mut Self {
        match self {
            Self::Button(b) => { b.highlight(); },
            Self::OptionSwitch(b) => { b.highlight(); },
            Self::OptionScroller(b) => { b.highlight(); },
        }
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        match self {
            Self::Button(b) => { b.unhighlight(); },
            Self::OptionSwitch(b) => { b.unhighlight(); },
            Self::OptionScroller(b) => { b.unhighlight(); },
        }
        self
    }
}

impl RunHandlers for ComponentState {
    async fn left_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.left_handle(h).await,
            Self::OptionSwitch(b) => b.left_handle(h).await,
            Self::OptionScroller(b) => b.left_handle(h).await,
        }
    }

    async fn right_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.right_handle(h).await,
            Self::OptionSwitch(b) => b.right_handle(h).await,
            Self::OptionScroller(b) => b.right_handle(h).await,
        }
    }

    async fn click_handle(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.click_handle(h).await,
            Self::OptionSwitch(b) => b.click_handle(h).await,
            Self::OptionScroller(b) => b.click_handle(h).await,
        }
    }
}

impl ComponentState {
    pub const fn definition(&self) -> ComponentDefinition {
        match self {
            Self::Button(b) => ComponentDefinition::Button(b.definition),
            Self::OptionSwitch(b) => ComponentDefinition::OptionSwitch(b.definition),
            Self::OptionScroller(b) => ComponentDefinition::OptionScroller(b.definition),
        }
    }
}

impl ComponentDefinition {
    pub fn construct(&self, start_focused: bool) -> ComponentState {
        match self {
            Self::Button(definition) => ComponentState::Button(ButtonState { definition, highlighted: false }),
            Self::OptionSwitch(definition) => ComponentState::OptionSwitch(OptionSwitchState { 
                definition, 
                mode: if start_focused { OptionSwitchMode::Selected } else { OptionSwitchMode::Unhighlighted },
                selection: 0, 
                text_undraw: None,
                cursor_undraws: None,
            }),
            Self::OptionScroller(definition) => ComponentState::OptionScroller(OptionScrollerState {
                definition,
                selection: 0,
                page_start: 0,
                redraw_scrollbar: true,
            }),
        }
    }

    pub const fn interaction_type(&self) -> InteractionType {
        match self {
            Self::Button(_) => InteractionType::NonEditable,
            Self::OptionSwitch(_) => InteractionType::Editable,
            Self::OptionScroller(_) => InteractionType::Editable,
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
// SCROLLING MENU THING {{{
pub struct OptionScrollerState {
    pub definition: &'static OptionScrollerDefinition,
    pub selection: usize,
    pub page_start: usize,
    pub redraw_scrollbar: bool,
}

pub struct OptionScrollerDefinition {
    pub pos: Point,
    pub click: OptionScrollerHandler,
    pub left: OptionScrollerHandler,
    pub right: OptionScrollerHandler,
    pub num_visible_elements: usize,
    pub width: u32,
    pub font_size: FontSize,
    pub options: &'static [&'static str],
}

impl RunHandlers for OptionScrollerState {
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

impl OptionScrollerState {
    const OPTION_HEIGHT: i32 = 20;
    const TEXT_OFFSET: i32 = 5;
    const SCROLLBAR_WIDTH: u32 = 5;

    const fn even_bg_rect_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg_secondary().as_rgb565())
            .build()
    }

    const fn odd_bg_rect_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg().as_rgb565())
            .build()
    }

    const fn pill_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.fg().as_rgb565())
            .build()
    }
}

impl ComponentBehaviour for OptionScrollerState {
    async fn draw(&mut self, s: &mut Screen<'_>, f: &mut PbFontRenderer) -> anyhow::Result<()> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        f.bgcol = theme.bg();
        f.font.borrow_mut().set_size(self.definition.font_size)?;
        for i in 0..self.definition.num_visible_elements {
            let option_index = i + self.page_start;
            let rect_pos = self.definition.pos + Point::new(0, i32::try_from(i)? * Self::OPTION_HEIGHT);
            let rect = Rectangle::new(rect_pos, Size::new(self.definition.width, Self::OPTION_HEIGHT as u32));
            rect.into_styled(
            if option_index % 2 == 0 {
                Self::even_bg_rect_style(&theme)
            } else {
                Self::odd_bg_rect_style(&theme)
            }
            ).draw(s)?;

            f.fgcol = if option_index == self.selection {
                theme.highlight()
            } else {
                theme.fg()
            };

            f.bgcol = if option_index % 2 == 0 {
                theme.bg_secondary()
            } else {
                theme.bg()
            };
            let option_text = Text::with_alignment(self.definition.options[option_index], rect_pos + Point::new(0, Self::OPTION_HEIGHT - Self::TEXT_OFFSET), &*f, Alignment::Left);
            option_text.draw(s)?;
        }

        if self.redraw_scrollbar {
            let bg = Rectangle::new(Point::new((s.size().width - Self::SCROLLBAR_WIDTH).try_into()?, 0), Size::new(Self::SCROLLBAR_WIDTH, s.size().height));
            let scrollbar_unit_length = (s.size().height as usize) / self.definition.options.len();
            let pill_top_left = Point::new(
                (s.size().width - Self::SCROLLBAR_WIDTH).try_into()?, 
                (scrollbar_unit_length * self.page_start).try_into()?);
            let pill = RoundedRectangle::with_equal_corners(
                Rectangle::new(pill_top_left, 
                    Size::new(Self::SCROLLBAR_WIDTH, (scrollbar_unit_length * self.definition.num_visible_elements).try_into()?)),
                Size::new(Self::SCROLLBAR_WIDTH / 2, Self::SCROLLBAR_WIDTH / 2),
            );
            bg.into_styled(Self::even_bg_rect_style(&theme)).draw(s)?;
            pill.into_styled(Self::pill_style(&theme)).draw(s)?;
            self.redraw_scrollbar = false;
        }
        Ok(())
    }

    fn highlight(&mut self) -> &mut Self {
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        self
    }
}
// }}}

// vim:foldmethod=marker