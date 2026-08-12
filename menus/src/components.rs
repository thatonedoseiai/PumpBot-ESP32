use crate::handlers::{HandlerResult, ButtonHandler, Handler, OptionSwitchHandler, OptionScrollerHandler, OptionsGenerator, TextGetterSetter, TextSubmitHandler, ValueSelectorHandler, InitialValueGenerator, ColorSelectorHandler, ColorGetter};
use crate::{IOHandles, MenuInternalState, ComponentMenuDefinition};
use crate::screen::screen::{Screen, ScreenDrawError};
use fontfile::{PbFont, pb_font_renderer::PbFontRenderer, FontSize, FontFileError};
use global_settings::{lang::*, rgb::RGB, rgb, PB_GLOBAL_SETTINGS, Theme};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{
        Rectangle, 
        PrimitiveStyleBuilder, 
        PrimitiveStyle, 
        RoundedRectangle,
        Triangle,
        Circle,
        Line,
    },
    geometry::AnchorPoint,
};
use core::fmt;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cell::{RefCell, Ref, Cell};
use alloc::rc::Rc;
use alloc::string::{String, ToString};

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
    TextBox(&'static TextBoxDefinition),
    ValueSelector(&'static ValueSelectorDefinition),
    ColorSelector(&'static ColorSelectorDefinition),
}

pub enum ComponentState {
    Button(ButtonState),
    OptionSwitch(OptionSwitchState),
    OptionScroller(OptionScrollerState),
    TextBox(TextBoxState),
    ValueSelector(ValueSelectorState),
    ColorSelector(ColorSelectorState),
}

pub trait RunHandlers {
    async fn left_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
    async fn right_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
    async fn click_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult>;
}

pub trait ComponentBehaviour {
    async fn draw(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()>;
    fn highlight(&mut self) -> &mut Self;
    fn unhighlight(&mut self) -> &mut Self;
}

impl ComponentBehaviour for ComponentState {
    async fn draw(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        match self {
            Self::Button(b) => b.draw(h).await,
            Self::OptionSwitch(b) => b.draw(h).await,
            Self::OptionScroller(b) => b.draw(h).await,
            Self::TextBox(b) => b.draw(h).await,
            Self::ValueSelector(b) => b.draw(h).await,
            Self::ColorSelector(b) => b.draw(h).await,
        }
    }

    fn highlight(&mut self) -> &mut Self {
        match self {
            Self::Button(b) => { b.highlight(); },
            Self::OptionSwitch(b) => { b.highlight(); },
            Self::OptionScroller(b) => { b.highlight(); },
            Self::TextBox(b) => { b.highlight(); },
            Self::ValueSelector(b) => { b.highlight(); },
            Self::ColorSelector(b) => { b.highlight(); },
        }
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        match self {
            Self::Button(b) => { b.unhighlight(); },
            Self::OptionSwitch(b) => { b.unhighlight(); },
            Self::OptionScroller(b) => { b.unhighlight(); },
            Self::TextBox(b) => { b.unhighlight(); },
            Self::ValueSelector(b) => { b.unhighlight(); }
            Self::ColorSelector(b) => { b.unhighlight(); }
        }
        self
    }
}

impl RunHandlers for ComponentState {
    async fn left_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.left_handle(menu_state, h).await,
            Self::OptionSwitch(b) => b.left_handle(menu_state, h).await,
            Self::OptionScroller(b) => b.left_handle(menu_state, h).await,
            Self::TextBox(b) => b.left_handle(menu_state, h).await,
            Self::ValueSelector(b) => b.left_handle(menu_state, h).await,
            Self::ColorSelector(b) => b.left_handle(menu_state, h).await,
        }
    }

    async fn right_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.right_handle(menu_state, h).await,
            Self::OptionSwitch(b) => b.right_handle(menu_state, h).await,
            Self::OptionScroller(b) => b.right_handle(menu_state, h).await,
            Self::TextBox(b) => b.right_handle(menu_state, h).await,
            Self::ValueSelector(b) => b.right_handle(menu_state, h).await,
            Self::ColorSelector(b) => b.right_handle(menu_state, h).await,
        }
    }

    async fn click_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        match self {
            Self::Button(b) => b.click_handle(menu_state, h).await,
            Self::OptionSwitch(b) => b.click_handle(menu_state, h).await,
            Self::OptionScroller(b) => b.click_handle(menu_state, h).await,
            Self::TextBox(b) => b.click_handle(menu_state, h).await,
            Self::ValueSelector(b) => b.click_handle(menu_state, h).await,
            Self::ColorSelector(b) => b.click_handle(menu_state, h).await,
        }
    }
}

impl ComponentState {
    pub const fn definition(&self) -> ComponentDefinition {
        match self {
            Self::Button(b) => ComponentDefinition::Button(b.definition),
            Self::OptionSwitch(b) => ComponentDefinition::OptionSwitch(b.definition),
            Self::OptionScroller(b) => ComponentDefinition::OptionScroller(b.definition),
            Self::TextBox(b) => ComponentDefinition::TextBox(b.definition),
            Self::ValueSelector(b) => ComponentDefinition::ValueSelector(b.definition),
            Self::ColorSelector(b) => ComponentDefinition::ColorSelector(b.definition),
        }
    }
}

impl ComponentDefinition {
    pub fn construct(&self, start_focused: bool, menu_definition: &ComponentMenuDefinition, internal_state: &MenuInternalState, h: &mut IOHandles<'_>) -> ComponentState {
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
                generated_options: RefCell::new(None),
            }),
            Self::TextBox(definition) => ComponentState::TextBox(TextBoxState {
                definition,
                highlighted: false,
                current_entry: Rc::new(RefCell::new(definition.initial_text.get_owned(menu_definition, internal_state, h))),
                sub_menu_open: false,
                text_selector_state: TextSelectorState {
                    selection: 0,
                    layer: TextSelectorLayer::Lowercase,
                    undraws: RefCell::new([Rectangle::zero(); 2 * TextSelectorState::NUM_PREVIEW_EACH_SIDE + 1]),
                },
                text_preview_state: RefCell::new(TextPreviewState {
                    undraws: Vec::new(),
                    cursor_pos: TextPreviewState::START_CURSOR_POS,
                    previous_cursor_positions: Vec::new(),
                    cursor_undraw: Rectangle::zero(),
                }),
            }),
            Self::ValueSelector(definition) => ComponentState::ValueSelector(ValueSelectorState {
                definition,
                selection: definition.initial_value.get(h),
                undraw: Rectangle::zero(),
                mode: if start_focused { ValueSelectorMode::Selected } else { ValueSelectorMode::Unhighlighted },
            }),
            Self::ColorSelector(definition) => ComponentState::ColorSelector(ColorSelectorState {
                definition,
                current_color: definition.initial_color.get(h),
                mode: if start_focused { ColorSelectorMode::Selected } else { ColorSelectorMode::Unhighlighted },
                selection_undraw: Rectangle::zero(),
                slider_state: ColorSliderState {
                    mode: SliderSetMode::ChangeChannel,
                    channel: ColorSelectorChannel::Red,
                },
            }),
        }
    }

    pub const fn interaction_type(&self) -> InteractionType {
        match self {
            Self::Button(_) => InteractionType::NonEditable,
            Self::OptionSwitch(_) => InteractionType::Editable,
            Self::OptionScroller(_) => InteractionType::Editable,
            Self::TextBox(_) => InteractionType::Editable,
            Self::ValueSelector(_) => InteractionType::Editable,
            Self::ColorSelector(_) => InteractionType::Editable,
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
    async fn draw(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        // println!("DRAWING COMPONENT [{}]", self.id);
        let s = &mut h.screen;
        let f = &mut h.font;
        let settings = &PB_GLOBAL_SETTINGS.read().await;
        let theme = &settings.theme;
        f.fgcol = theme.fg();
        f.bgcol = theme.bg();
        f.font.borrow_mut().set_size(self.definition.font_size).unwrap();
        let button_text = Text::with_alignment(self.definition.text[settings.lang], self.definition.pos, &h.font, Alignment::Left);
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
    pub text: &'static LanguageString,
}

impl RunHandlers for ButtonState {
    async fn left_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.left.handle(self, menu_state, h).await
    }

    async fn right_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.right.handle(self, menu_state, h).await
    }

    async fn click_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.click.handle(self, menu_state, h).await
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
    pub options: &'static [LanguageString],
}

impl RunHandlers for OptionSwitchState {
    async fn left_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.left.handle(self, menu_state, h).await
    }

    async fn right_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.right.handle(self, menu_state, h).await
    }

    async fn click_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.click.handle(self, menu_state, h).await
    }
}

impl OptionSwitchState {
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
    async fn draw(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let s = &mut h.screen;
        let f = &mut h.font;
        let settings = &PB_GLOBAL_SETTINGS.read().await;
        let theme = &settings.theme;
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
                Text::with_alignment(self.definition.options[self.selection][settings.lang], self.definition.pos, &h.font, Alignment::Left).draw(s)?;
            },
            OptionSwitchMode::Highlighted => {
                f.fgcol = theme.highlight();
                Text::with_alignment(self.definition.options[self.selection][settings.lang], self.definition.pos, &h.font, Alignment::Left).draw(s)?;
            },
            OptionSwitchMode::Selected => {
                f.fgcol = theme.highlight();
                let text = Text::with_alignment(self.definition.options[self.selection][settings.lang], self.definition.pos, &h.font, Alignment::Left);
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
// OPTION SCROLLER {{{
pub struct OptionScrollerState {
    pub definition: &'static OptionScrollerDefinition,
    pub selection: usize,
    pub page_start: usize,
    pub redraw_scrollbar: bool,
    pub generated_options: RefCell<Option<Rc<Vec<Cow<'static, str>>>>>,
}

pub struct OptionScrollerDefinition {
    pub pos: Point,
    pub click: OptionScrollerHandler,
    pub left: OptionScrollerHandler,
    pub right: OptionScrollerHandler,
    pub num_visible_elements: usize,
    pub width: u32,
    pub font_size: FontSize,
    pub options: OptionsGenerator,
}

impl RunHandlers for OptionScrollerState {
    async fn left_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.left.handle(self, menu_state, h).await
    }

    async fn right_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.right.handle(self, menu_state, h).await
    }

    async fn click_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.click.handle(self, menu_state, h).await
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

    pub async fn generated_options(&self, h: &mut IOHandles<'_>) -> anyhow::Result<Rc<Vec<Cow<'static, str>>>> {
        let is_uninitialized = {
            self.generated_options.borrow().is_none()
        };
        if is_uninitialized {
            let res = self.definition.options.generate(&PB_GLOBAL_SETTINGS.read().await.lang, h).await?;
            self.generated_options.replace(Some(Rc::new(res)));
        }

        Ok(self.generated_options.borrow().clone().expect("Menu contents should have been generated!"))
    }
}

impl ComponentBehaviour for OptionScrollerState {
    async fn draw(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let options = self.generated_options(h).await?;
        let f = &mut h.font;
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        f.bgcol = theme.bg();
        f.font.borrow_mut().set_size(self.definition.font_size)?;
        for i in 0..(core::cmp::min(self.definition.num_visible_elements, options.len())) {
            let option_index = i + self.page_start;
            let rect_pos = self.definition.pos + Point::new(0, i32::try_from(i)? * Self::OPTION_HEIGHT);
            let rect = Rectangle::new(rect_pos, Size::new(self.definition.width, Self::OPTION_HEIGHT as u32));
            rect.into_styled(
            if option_index % 2 == 0 {
                Self::even_bg_rect_style(&theme)
            } else {
                Self::odd_bg_rect_style(&theme)
            }
            ).draw(&mut h.screen)?;

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
            let option_text = Text::with_alignment(&options[option_index], rect_pos + Point::new(0, Self::OPTION_HEIGHT - Self::TEXT_OFFSET), &*f, Alignment::Left);
            option_text.draw(&mut h.screen)?;
        }

        if self.redraw_scrollbar && (self.definition.num_visible_elements < options.len()) {
            let bg = Rectangle::new(Point::new((h.screen.size().width - Self::SCROLLBAR_WIDTH).try_into()?, 0), Size::new(Self::SCROLLBAR_WIDTH, h.screen.size().height));
            let scrollbar_unit_length = (h.screen.size().height as usize) / options.len();
            let pill_top_left = Point::new(
                (h.screen.size().width - Self::SCROLLBAR_WIDTH).try_into()?, 
                (scrollbar_unit_length * self.page_start).try_into()?);
            let pill = RoundedRectangle::with_equal_corners(
                Rectangle::new(pill_top_left, 
                    Size::new(Self::SCROLLBAR_WIDTH, (scrollbar_unit_length * self.definition.num_visible_elements).try_into()?)),
                Size::new(Self::SCROLLBAR_WIDTH / 2, Self::SCROLLBAR_WIDTH / 2),
            );
            bg.into_styled(Self::even_bg_rect_style(&theme)).draw(&mut h.screen)?;
            pill.into_styled(Self::pill_style(&theme)).draw(&mut h.screen)?;
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
// TEXT BOX {{{
#[derive(Clone, Copy)]
enum TextSelectorLayer {
    Lowercase,
    Uppercase,
    Symbol,
}

impl TextSelectorLayer {
    const LOWERCASE_REEL: &'static [TextSelectorOption] = &[
        TextSelectorOption { symbol: 'a', action: Action::Type('a') },
        TextSelectorOption { symbol: 'b', action: Action::Type('b') },
        TextSelectorOption { symbol: 'c', action: Action::Type('c') },
        TextSelectorOption { symbol: 'd', action: Action::Type('d') },
        TextSelectorOption { symbol: 'e', action: Action::Type('e') },
        TextSelectorOption { symbol: 'f', action: Action::Type('f') },
        TextSelectorOption { symbol: 'g', action: Action::Type('g') },
        TextSelectorOption { symbol: 'h', action: Action::Type('h') },
        TextSelectorOption { symbol: 'i', action: Action::Type('i') },
        TextSelectorOption { symbol: 'j', action: Action::Type('j') },
        TextSelectorOption { symbol: 'k', action: Action::Type('k') },
        TextSelectorOption { symbol: 'l', action: Action::Type('l') },
        TextSelectorOption { symbol: 'm', action: Action::Type('m') },
        TextSelectorOption { symbol: 'n', action: Action::Type('n') },
        TextSelectorOption { symbol: 'o', action: Action::Type('o') },
        TextSelectorOption { symbol: 'p', action: Action::Type('p') },
        TextSelectorOption { symbol: 'q', action: Action::Type('q') },
        TextSelectorOption { symbol: 'r', action: Action::Type('r') },
        TextSelectorOption { symbol: 's', action: Action::Type('s') },
        TextSelectorOption { symbol: 't', action: Action::Type('t') },
        TextSelectorOption { symbol: 'u', action: Action::Type('u') },
        TextSelectorOption { symbol: 'v', action: Action::Type('v') },
        TextSelectorOption { symbol: 'w', action: Action::Type('w') },
        TextSelectorOption { symbol: 'x', action: Action::Type('x') },
        TextSelectorOption { symbol: 'y', action: Action::Type('y') },
        TextSelectorOption { symbol: 'z', action: Action::Type('z') },
        TextSelectorOption { symbol: '_', action: Action::Type(' ') },
        TextSelectorOption { symbol: '✓', action: Action::Confirm },
        TextSelectorOption { symbol: 'A', action: Action::ToLayer(TextSelectorLayer::Uppercase) },
        TextSelectorOption { symbol: '@', action: Action::ToLayer(TextSelectorLayer::Symbol) },
        TextSelectorOption { symbol: '⌫', action: Action::Backspace },
        // TextSelectorOption { symbol: 'L', action: Action::Left },
        // TextSelectorOption { symbol: 'R', action: Action::Right },
    ];
    const UPPERCASE_REEL: &'static [TextSelectorOption] = &[
        TextSelectorOption { symbol: 'A', action: Action::Type('A') },
        TextSelectorOption { symbol: 'B', action: Action::Type('B') },
        TextSelectorOption { symbol: 'C', action: Action::Type('C') },
        TextSelectorOption { symbol: 'D', action: Action::Type('D') },
        TextSelectorOption { symbol: 'E', action: Action::Type('E') },
        TextSelectorOption { symbol: 'F', action: Action::Type('F') },
        TextSelectorOption { symbol: 'G', action: Action::Type('G') },
        TextSelectorOption { symbol: 'H', action: Action::Type('H') },
        TextSelectorOption { symbol: 'I', action: Action::Type('I') },
        TextSelectorOption { symbol: 'J', action: Action::Type('J') },
        TextSelectorOption { symbol: 'K', action: Action::Type('K') },
        TextSelectorOption { symbol: 'L', action: Action::Type('L') },
        TextSelectorOption { symbol: 'M', action: Action::Type('M') },
        TextSelectorOption { symbol: 'N', action: Action::Type('N') },
        TextSelectorOption { symbol: 'O', action: Action::Type('O') },
        TextSelectorOption { symbol: 'P', action: Action::Type('P') },
        TextSelectorOption { symbol: 'Q', action: Action::Type('Q') },
        TextSelectorOption { symbol: 'R', action: Action::Type('R') },
        TextSelectorOption { symbol: 'S', action: Action::Type('S') },
        TextSelectorOption { symbol: 'T', action: Action::Type('T') },
        TextSelectorOption { symbol: 'U', action: Action::Type('U') },
        TextSelectorOption { symbol: 'V', action: Action::Type('V') },
        TextSelectorOption { symbol: 'W', action: Action::Type('W') },
        TextSelectorOption { symbol: 'X', action: Action::Type('X') },
        TextSelectorOption { symbol: 'Y', action: Action::Type('Y') },
        TextSelectorOption { symbol: 'Z', action: Action::Type('Z') },
        TextSelectorOption { symbol: '_', action: Action::Type(' ') },
        TextSelectorOption { symbol: 'a', action: Action::ToLayer(TextSelectorLayer::Lowercase) },
        TextSelectorOption { symbol: '@', action: Action::ToLayer(TextSelectorLayer::Symbol) },
        TextSelectorOption { symbol: '✓', action: Action::Confirm },
        TextSelectorOption { symbol: '⌫', action: Action::Backspace },
    ];
    const SYMBOL_REEL: &'static [TextSelectorOption] = &[
        TextSelectorOption { symbol: '?', action: Action::Type('?') },
        TextSelectorOption { symbol: '!', action: Action::Type('!') },
        TextSelectorOption { symbol: '"', action: Action::Type('"') },
        TextSelectorOption { symbol: '#', action: Action::Type('#') },
        TextSelectorOption { symbol: '$', action: Action::Type('$') },
        TextSelectorOption { symbol: '%', action: Action::Type('%') },
        TextSelectorOption { symbol: '&', action: Action::Type('&') },
        TextSelectorOption { symbol: '\'',action: Action::Type('\'') },
        TextSelectorOption { symbol: '(', action: Action::Type('(') },
        TextSelectorOption { symbol: ')', action: Action::Type(')') },
        TextSelectorOption { symbol: '*', action: Action::Type('*') },
        TextSelectorOption { symbol: '+', action: Action::Type('+') },
        TextSelectorOption { symbol: ',', action: Action::Type(',') },
        TextSelectorOption { symbol: '-', action: Action::Type('-') },
        TextSelectorOption { symbol: '.', action: Action::Type('.') },
        TextSelectorOption { symbol: '/', action: Action::Type('/') },
        TextSelectorOption { symbol: '0', action: Action::Type('0') },
        TextSelectorOption { symbol: '1', action: Action::Type('1') },
        TextSelectorOption { symbol: '2', action: Action::Type('2') },
        TextSelectorOption { symbol: '3', action: Action::Type('3') },
        TextSelectorOption { symbol: '4', action: Action::Type('4') },
        TextSelectorOption { symbol: '5', action: Action::Type('5') },
        TextSelectorOption { symbol: '6', action: Action::Type('6') },
        TextSelectorOption { symbol: '7', action: Action::Type('7') },
        TextSelectorOption { symbol: '8', action: Action::Type('8') },
        TextSelectorOption { symbol: '9', action: Action::Type('9') },
        TextSelectorOption { symbol: ':', action: Action::Type(':') },
        TextSelectorOption { symbol: ';', action: Action::Type(';') },
        TextSelectorOption { symbol: '<', action: Action::Type('<') },
        TextSelectorOption { symbol: '=', action: Action::Type('=') },
        TextSelectorOption { symbol: '>', action: Action::Type('>') },
        TextSelectorOption { symbol: 'a', action: Action::ToLayer(TextSelectorLayer::Lowercase) },
        TextSelectorOption { symbol: 'A', action: Action::ToLayer(TextSelectorLayer::Uppercase) },
        TextSelectorOption { symbol: '✓', action: Action::Confirm },
        TextSelectorOption { symbol: '⌫', action: Action::Backspace },
    ];

    pub const fn as_reel(&self) -> &'static [TextSelectorOption] {
        match self {
            Self::Lowercase => &Self::LOWERCASE_REEL,
            Self::Uppercase => &Self::UPPERCASE_REEL,
            Self::Symbol => &Self::SYMBOL_REEL,
        }
    }
}

enum Action {
    Left,
    Right,
    Confirm,
    Backspace,
    ToLayer(TextSelectorLayer),
    Type(char),
}

struct TextSelectorOption {
    symbol: char,
    action: Action,
}

struct TextSelectorState {
    selection: usize,
    layer: TextSelectorLayer,
    undraws: RefCell<[Rectangle; 2 * Self::NUM_PREVIEW_EACH_SIDE + 1]>,
}

impl TextSelectorState {
    const NUM_PREVIEW_EACH_SIDE: usize = 3;
    const OPTION_SPACING: usize = 15;
    const TEXT_REEL_POSITION: Point = Point::new(64, 140);

    const fn undraw_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg().as_rgb565())
            .build()
    }

    pub const fn prev(&mut self) {
        let num_elems = self.layer.as_reel().len();
        self.selection = (self.selection + num_elems - 1) % num_elems;
    }

    pub const fn next(&mut self) {
        self.selection = (self.selection + 1) % self.layer.as_reel().len();
    }

    pub const fn selection(&self) -> &Action {
        &self.layer.as_reel()[self.selection].action
    }

    pub async fn draw(&self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let num_elems = self.layer.as_reel().len();
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        let beginning_index = (self.selection + num_elems - Self::NUM_PREVIEW_EACH_SIDE) % num_elems;
        let mut tmp = [0;4];
        h.font.bgcol = theme.bg();
        log::info!("drawing text reel!");

        let position = Self::TEXT_REEL_POSITION - Point::new((Self::OPTION_SPACING * Self::NUM_PREVIEW_EACH_SIDE).try_into()?, 0);

        for i in 0..(2 * Self::NUM_PREVIEW_EACH_SIDE + 1) {
            let index = (beginning_index + i) % num_elems;
            h.font.fgcol = if index == self.selection {
                theme.fg()
            } else {
                theme.fg()
            };
            h.font.font.borrow_mut().set_size(
                if index == self.selection {
                    FontSize::Sz12
                } else {
                    FontSize::Sz7
                }
            )?;

            let text_reel_symbol = Text::with_alignment(
                self.layer.as_reel()[index].symbol.encode_utf8(&mut tmp),
                position + Point::new((i * Self::OPTION_SPACING).try_into()?, 0),
                &h.font,
                Alignment::Center
            );
            self.undraws.borrow()[i].into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
            self.undraws.borrow_mut()[i] = text_reel_symbol.bounding_box();
            text_reel_symbol.draw(&mut h.screen)?;
        }
        Ok(())
    }
}

struct TextPreviewState {
    undraws: Vec<Rectangle>,
    cursor_pos: Point,
    previous_cursor_positions: Vec<Point>,
    cursor_undraw: Rectangle,
}

impl TextPreviewState {
    const START_CURSOR_POS: Point = Point::new(10, 20);
    const NEWLINE_HEIGHT: i32 = 15;
    const TEXT_PREVIEW_FONT_SIZE: FontSize = FontSize::Sz12;
    const CURSOR_OBJECT: Line = Line::new(Point::new(0, 0), Point::new(0, 18));
    const CURSOR_VERTICAL_OFFSET: i32 = -15;

    const fn undraw_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg().as_rgb565())
            .build()
    }

    const fn cursor_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_color(theme.highlight().as_rgb565())
            .stroke_width(2)
            .build()
    }

    fn undraw_cursor(&mut self, s: &mut Screen, theme: &Theme) -> Result<(), ScreenDrawError> {
        self.cursor_undraw.into_styled(Self::undraw_style(theme)).draw(s)
    }

    fn redraw_cursor(&mut self, s: &mut Screen, theme: &Theme) -> Result<(), ScreenDrawError> {
        let cursor = Self::CURSOR_OBJECT
            .translate(self.cursor_pos + Point::new(0, Self::CURSOR_VERTICAL_OFFSET))
            .into_styled(Self::cursor_style(theme));
        self.cursor_undraw = cursor.bounding_box();
        cursor.draw(s)?;
        Ok(())
    }

    async fn draw(&mut self, text: &str, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        log::info!("drawing text preview with string {}", text);

        let mut tmp = [0;4];
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        h.font.font.borrow_mut().set_size(Self::TEXT_PREVIEW_FONT_SIZE)?;
        h.font.fgcol = theme.fg();
        h.font.bgcol = theme.bg();

        self.undraw_cursor(&mut h.screen, &theme)?;

        for c in text.chars() {
            let mut char_as_text = Text::with_alignment(c.encode_utf8(&mut tmp), self.cursor_pos, &h.font, Alignment::Left);
            let char_bb = char_as_text.bounding_box();
            if char_bb.size.width.checked_add_signed(char_bb.top_left.x) > Some(h.screen.size().width) {
                char_as_text.translate_mut(Point::new(Self::START_CURSOR_POS.x - char_bb.top_left.x, Self::NEWLINE_HEIGHT));
            }
            self.previous_cursor_positions.push(self.cursor_pos);
            self.undraws.push(char_bb);
            self.cursor_pos = char_as_text.draw(&mut h.screen)?;
        }

        self.redraw_cursor(&mut h.screen, &theme)?;

        Ok(())
    }

    async fn backspace(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;

        self.undraw_cursor(&mut h.screen, &theme)?;

        if let Some(rect) = self.undraws.pop() {
            rect.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
            if let Some(prev_pos) = self.previous_cursor_positions.pop() {
                self.cursor_pos = prev_pos;
            }
        }

        self.redraw_cursor(&mut h.screen, &theme)?;

        Ok(())
    }
}

pub struct TextBoxState {
    pub definition: &'static TextBoxDefinition,
    pub highlighted: bool,
    pub current_entry: Rc<RefCell<String>>,
    pub sub_menu_open: bool,
    text_selector_state: TextSelectorState,
    text_preview_state: RefCell<TextPreviewState>,
}

pub struct TextBoxDefinition {
    pub pos: Point,
    pub max_length: usize,
    pub width: u32,
    pub preview_chars: usize,
    pub font_size: FontSize,
    pub empty_text: &'static str,
    pub initial_text: TextGetterSetter,
    pub on_submit: TextSubmitHandler,
}

impl TextBoxState {
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

    fn get_preview(&self) -> String {
        let text = self.current_entry.as_ref().borrow();
        if text.is_empty() {
            self.definition.empty_text.to_string()
        } else if text.len() < self.definition.preview_chars {
            text.clone()
        } else {
            text[0..self.definition.preview_chars].to_string() + "..."
        }
    }
}

impl RunHandlers for TextBoxState {
    async fn left_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        if self.sub_menu_open {
            self.text_selector_state.prev();
            self.text_selector_state.draw(h).await?;
        }
        Ok(HandlerResult::None)
    }

    async fn right_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        if self.sub_menu_open {
            self.text_selector_state.next();
            self.text_selector_state.draw(h).await?;
        }
        Ok(HandlerResult::None)
    }

    async fn click_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        log::info!("self.sub_menu_open: {}", self.sub_menu_open);
        if !self.sub_menu_open {
            let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
            h.font.fgcol = theme.fg();
            h.font.bgcol = theme.bg();

            // let preview_text = Text::with_alignment(&self.current_entry.as_ref().borrow(), self.definition.pos, &h.font, Alignment::Left);
            h.screen.clear(theme.bg().as_rgb565())?;
            self.text_preview_state.borrow_mut().cursor_pos = TextPreviewState::START_CURSOR_POS;

            self.text_preview_state.borrow_mut().draw(&self.current_entry.as_ref().borrow(), h).await?;
            self.text_selector_state.draw(h).await?;
            self.sub_menu_open = true;
            Ok(HandlerResult::None)
        } else {
            match self.text_selector_state.selection() {
                Action::Left => {
                    unimplemented!()
                },
                Action::Right => {
                    unimplemented!()
                },
                Action::Confirm => {
                    let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
                    h.screen.clear(theme.bg().as_rgb565())?;
                    self.sub_menu_open = false;
                    self.definition.on_submit.handle(self, menu_state, h).await?;
                    Ok(HandlerResult::ForceRedrawAndUnfocus)
                },
                Action::Backspace => {
                    if self.current_entry.as_ref().borrow_mut().pop().is_some() {
                        self.text_preview_state.borrow_mut().backspace(h).await?;
                    }
                    Ok(HandlerResult::None)
                },
                Action::Type(c) => {
                    if self.current_entry.as_ref().borrow().len() < self.definition.max_length {
                        self.current_entry.as_ref().borrow_mut().push(*c);
                        let mut buf = [0;4];
                        self.text_preview_state.borrow_mut().draw(c.encode_utf8(&mut buf), h).await?;
                    }
                    Ok(HandlerResult::None)
                },
                Action::ToLayer(l) => {
                    self.text_selector_state.layer = *l;
                    self.text_selector_state.draw(h).await?;
                    Ok(HandlerResult::None)
                }
            }
        }
    }
}

impl ComponentBehaviour for TextBoxState {
    async fn draw(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        h.font.fgcol = if self.current_entry.as_ref().borrow().is_empty() { theme.bg_secondary() } else { theme.fg() };
        h.font.bgcol = theme.bg();
        h.font.font.borrow_mut().set_size(self.definition.font_size)?;

        let text = self.get_preview();
        let entry_text = Text::with_alignment(&text, self.definition.pos, &h.font, Alignment::Left);
        let text_bb = entry_text.bounding_box();

        let backing_box = RoundedRectangle::with_equal_corners(
            Rectangle::new(
                text_bb.top_left - Point::new(Self::BORDER_SIZE as i32 / 2, Self::BORDER_SIZE as i32 / 2),
                Size::new(self.definition.width, text_bb.size.height + Self::BORDER_SIZE)
            ),
            Size::new(Self::RADIUS, Self::RADIUS),
        );

        backing_box.into_styled(
            if self.highlighted {
                Self::highlighted_graphic_style(theme)
            } else {
                Self::graphic_style(theme)
            }
        ).draw(&mut h.screen)?;

        entry_text.draw(&mut h.screen)?;
        // log::info!("drawing text box with entry: {} at {}, fgcol {:?} bgcol {:?}", &self.current_entry.as_ref().borrow(), self.definition.pos, h.font.fgcol, h.font.bgcol);
        Ok(())
    }

    fn highlight(&mut self) -> &mut Self {
        self.highlighted = true;
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        self.highlighted = false;
        self
    }
}

// }}}
// VALUE SELECTOR {{{
pub type ValueSelectorNumType = u32;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueSelectorMode {
    Highlighted,
    Unhighlighted,
    Selected,
}

pub struct ValueSelectorState {
    pub definition: &'static ValueSelectorDefinition,
    pub selection: ValueSelectorNumType,
    pub undraw: Rectangle,
    pub mode: ValueSelectorMode,
}

pub struct ValueSelectorDefinition {
    pub pos: Point,
    pub click: ValueSelectorHandler,
    pub left: ValueSelectorHandler,
    pub right: ValueSelectorHandler,
    pub suffix: &'static str,
    pub font_size: FontSize,
    pub low_limit: ValueSelectorNumType,
    pub high_limit: ValueSelectorNumType,
    pub initial_value: InitialValueGenerator,
}

impl RunHandlers for ValueSelectorState {
    async fn left_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.left.handle(self, menu_state, h).await
    }

    async fn right_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.right.handle(self, menu_state, h).await
    }

    async fn click_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        self.definition.click.handle(self, menu_state, h).await
    }
}

impl ValueSelectorState {
    const fn undraw_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg().as_rgb565())
            .build()
    }
}

impl ComponentBehaviour for ValueSelectorState {
    async fn draw(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        h.font.fgcol = if self.mode == ValueSelectorMode::Highlighted {
            theme.highlight()
        } else {
            theme.fg()
        };
        h.font.bgcol = theme.bg();
        h.font.font.borrow_mut().set_size(self.definition.font_size)?;
        let preview = self.selection.to_string() + self.definition.suffix;
        let item_text = Text::with_alignment(&preview, self.definition.pos, &h.font, Alignment::Left);
        self.undraw.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
        self.undraw = item_text.bounding_box();
        item_text.draw(&mut h.screen)?;
        Ok(())
    }

    fn highlight(&mut self) -> &mut Self {
        self.mode = ValueSelectorMode::Highlighted;
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        self.mode = ValueSelectorMode::Unhighlighted;
        self
    }
}
// }}}
// COLOR SELECTOR {{{
enum SliderSetMode {
    ChangeChannel,
    ModifyChannel,
}

enum ColorSelectorMode {
    Highlighted,
    Unhighlighted,
    Selected,
}

struct ColorSliderState {
    mode: SliderSetMode,
    channel: ColorSelectorChannel,
}

#[derive(Eq, PartialEq)]
enum ColorSelectorChannel {
    Red,
    Green,
    Blue,
    Done,
}

impl ColorSelectorChannel {
    const fn slider_x_pos(&self) -> i32 {
        match self {
            Self::Red => 19,
            Self::Green => 49,
            Self::Blue => 79,
            Self::Done => unreachable!(),
        }
    }
}

pub struct ColorSelectorState {
    pub definition: &'static ColorSelectorDefinition,
    pub current_color: RGB,
    pub mode: ColorSelectorMode,
    selection_undraw: Rectangle,
    slider_state: ColorSliderState,
}

pub struct ColorSelectorDefinition {
    pub preview_rect: Rectangle,
    pub left: ColorSelectorHandler,
    pub right: ColorSelectorHandler,
    // pub click: ColorSelectorHandler,
    pub initial_color: ColorGetter,
}

impl RunHandlers for ColorSelectorState {
    async fn left_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        match self.slider_state.mode {
            SliderSetMode::ChangeChannel => {
                self.slider_state.channel = match self.slider_state.channel {
                    ColorSelectorChannel::Red => ColorSelectorChannel::Done,
                    ColorSelectorChannel::Green => ColorSelectorChannel::Red,
                    ColorSelectorChannel::Blue => ColorSelectorChannel::Green,
                    ColorSelectorChannel::Done => ColorSelectorChannel::Blue,
                }
            }
            SliderSetMode::ModifyChannel => {
                match self.slider_state.channel {
                    ColorSelectorChannel::Red => {
                        self.current_color.r = self.current_color.r.saturating_sub(1);
                        self.draw_slider(Point::new(ColorSelectorChannel::Red.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.r, h)?;
                    },
                    ColorSelectorChannel::Green => {
                        self.current_color.g = self.current_color.g.saturating_sub(1);
                        self.draw_slider(Point::new(ColorSelectorChannel::Green.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.g, h)?;
                    },
                    ColorSelectorChannel::Blue => {
                        self.current_color.b = self.current_color.b.saturating_sub(1);
                        self.draw_slider(Point::new(ColorSelectorChannel::Blue.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.b, h)?;
                    },
                    ColorSelectorChannel::Done => unreachable!(),
                }
                Self::EDITOR_PREVIEW_RECT.into_styled(self.preview_style()).draw(&mut h.screen)?;
            }
        }
        Ok(HandlerResult::None)
        // self.definition.right.handle(self, menu_state, h).await
    }

    async fn right_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        match self.slider_state.mode {
            SliderSetMode::ChangeChannel => {
                self.slider_state.channel = match self.slider_state.channel {
                    ColorSelectorChannel::Red => ColorSelectorChannel::Green,
                    ColorSelectorChannel::Green => ColorSelectorChannel::Blue,
                    ColorSelectorChannel::Blue => ColorSelectorChannel::Done,
                    ColorSelectorChannel::Done => ColorSelectorChannel::Red,
                }
            }
            SliderSetMode::ModifyChannel => {
                match self.slider_state.channel {
                    ColorSelectorChannel::Red => {
                        self.current_color.r = self.current_color.r.saturating_add(1);
                        self.draw_slider(Point::new(ColorSelectorChannel::Red.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.r, h)?;
                    },
                    ColorSelectorChannel::Green => {
                        self.current_color.g = self.current_color.g.saturating_add(1);
                        self.draw_slider(Point::new(ColorSelectorChannel::Green.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.g, h)?;
                    },
                    ColorSelectorChannel::Blue => {
                        self.current_color.b = self.current_color.b.saturating_add(1);
                        self.draw_slider(Point::new(ColorSelectorChannel::Blue.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.b, h)?;
                    },
                    ColorSelectorChannel::Done => unreachable!(),
                }
                Self::EDITOR_PREVIEW_RECT.into_styled(self.preview_style()).draw(&mut h.screen)?;
            }
        }
        Ok(HandlerResult::None)
    }

    async fn click_handle(&mut self, menu_state: &mut MenuInternalState, h: &mut IOHandles<'_>) -> anyhow::Result<HandlerResult> {
        // self.definition.click.handle(self, menu_state, h).await
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        match self.mode {
            ColorSelectorMode::Unhighlighted => unreachable!(),
            ColorSelectorMode::Highlighted => {
                self.mode = ColorSelectorMode::Selected;
                h.screen.clear(theme.bg().as_rgb565())?;
                self.draw(h).await?;
                Ok(HandlerResult::None)
            },
            ColorSelectorMode::Selected => {
                match self.slider_state.mode {
                    SliderSetMode::ChangeChannel => {
                        if self.slider_state.channel == ColorSelectorChannel::Done {
                            self.mode = ColorSelectorMode::Highlighted;
                            h.screen.clear(theme.bg().as_rgb565())?;
                            Ok(HandlerResult::ForceRedrawAndUnfocus)
                        } else {
                            self.slider_state.mode = SliderSetMode::ModifyChannel;
                            Ok(HandlerResult::None)
                        }
                    }
                    SliderSetMode::ModifyChannel => {
                        self.slider_state.mode = SliderSetMode::ChangeChannel;
                        Ok(HandlerResult::None)
                    }
                }
            }
        }
    }
}

impl ColorSelectorState {
    const BORDER_SIZE: u32 = 10;
    const RADIUS: u32 = 5;
    const EDITOR_PREVIEW_RECT: Rectangle = Rectangle::new(Point::new(0, 100), Size::new(128, 50));
    const SLIDERS_TOP: i32 = 20;
    const SLIDERS_HEIGHT: u32 = 80;
    const SLIDERS_WIDTH: u32 = 30;
    const CURSOR_RADIUS: i32 = 5;
    const DONE_TEXT_POS: Point = Point::new(64, 20);
    // const RECT_R: Rectangle = Rectangle::new(Point::new(19, Self::SLIDERS_TOP), Size::new(30, Self::SLIDERS_HEIGHT));
    // const RECT_G: Rectangle = Rectangle::new(Point::new(49, Self::SLIDERS_TOP), Size::new(30, Self::SLIDERS_HEIGHT));
    // const RECT_B: Rectangle = Rectangle::new(Point::new(79, Self::SLIDERS_TOP), Size::new(30, Self::SLIDERS_HEIGHT));
    const CURSOR: Circle = Circle::new(Point::new(0, 0), Self::CURSOR_RADIUS as u32 * 2);
    const CURSOR_STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
        .stroke_width(2)
        .stroke_color(Rgb565::WHITE)
        .build();

    fn draw_preview(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        self.definition.preview_rect.into_styled(self.preview_style()).draw(&mut h.screen)?;
        Ok(())
    }

    fn draw_slider(&mut self, pos: Point, style: PrimitiveStyle<Rgb565>, val: u8, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        Rectangle::new(pos, Size::new(Self::SLIDERS_WIDTH, Self::SLIDERS_HEIGHT)).into_styled(style).draw(&mut h.screen)?;
        Self::CURSOR.translate(Point::new(pos.x + (Self::SLIDERS_WIDTH as i32 / 2) - Self::CURSOR_RADIUS, Self::SLIDERS_TOP + ((Self::SLIDERS_HEIGHT - Self::CURSOR.diameter) * val as u32) as i32 / 255)).into_styled(Self::CURSOR_STYLE).draw(&mut h.screen)?;
        Ok(())
    }

    fn draw_done_button(&mut self, lang: Lang, theme: &Theme, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        h.font.fgcol = if self.slider_state.channel == ColorSelectorChannel::Done { theme.highlight() } else { theme.fg() };
        h.font.bgcol = theme.bg();
        Text::with_alignment(TEXT_OK[lang], Self::DONE_TEXT_POS, &h.font, Alignment::Center).draw(&mut h.screen)?;
        Ok(())
    }

    const fn undraw_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg().as_rgb565())
            .build()
    }

    const fn preview_style(&self) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(self.current_color.as_rgb565())
            .build()
    }

    const fn selection_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_width(3)
            .stroke_color(theme.highlight().as_rgb565())
            .build()
    }
}

impl ComponentBehaviour for ColorSelectorState {
    async fn draw(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let settings = &PB_GLOBAL_SETTINGS.read().await;
        let theme = &settings.theme;

        match self.mode {
            ColorSelectorMode::Highlighted => {
                let selection_rect = RoundedRectangle::with_equal_corners(
                    self.definition.preview_rect.resized(self.definition.preview_rect.size + Size::new(Self::BORDER_SIZE, Self::BORDER_SIZE), AnchorPoint::Center),
                    Size::new(Self::RADIUS, Self::RADIUS),
                ).into_styled(Self::selection_style(theme));

                self.selection_undraw = selection_rect.bounding_box();

                selection_rect.draw(&mut h.screen)?;

                self.draw_preview(h)?;
            },
            ColorSelectorMode::Unhighlighted => {
                self.selection_undraw.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
                self.selection_undraw = Rectangle::zero();
                self.draw_preview(h)?;
            },
            ColorSelectorMode::Selected => {
                Self::EDITOR_PREVIEW_RECT.into_styled(self.preview_style()).draw(&mut h.screen)?;
                self.draw_slider(Point::new(ColorSelectorChannel::Red.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.r, h)?;
                self.draw_slider(Point::new(ColorSelectorChannel::Green.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.g, h)?;
                self.draw_slider(Point::new(ColorSelectorChannel::Blue.slider_x_pos(), Self::SLIDERS_TOP), Self::selection_style(theme), self.current_color.b, h)?;
                self.draw_done_button(settings.lang, theme, h)?;
            },
        }
        Ok(())
    }

    fn highlight(&mut self) -> &mut Self {
        self.mode = ColorSelectorMode::Highlighted;
        self
    }

    fn unhighlight(&mut self) -> &mut Self {
        self.mode = ColorSelectorMode::Unhighlighted;
        self
    }
}
// }}}

// vim:foldmethod=marker