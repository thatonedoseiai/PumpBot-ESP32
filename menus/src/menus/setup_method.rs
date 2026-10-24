use crate::{MenuSignal, IOHandles, Menu, ComponentMenu};
use rotenc::Direction;
use button_idf::{ButtonType, ButtonEventKind};
use fontfile::FontSize;
use global_settings::{
    lang::{
        Lang,
        LanguageString,
        TEXT_SETUP_PB,
        TEXT_WIFI_SETUP,
        TEXT_STANDALONE_SETUP,
        TEXT_TOOLTIP_WIFI_SETUP,
        TEXT_TOOLTIP_STANDALONE_SETUP,
        TEXT_NEXT,
        TEXT_BACK,
    }, 
    Theme,
    PB_GLOBAL_SETTINGS
};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{Rectangle, PrimitiveStyleBuilder, PrimitiveStyle, Line, Triangle, Styled}
};
use embassy_futures::select::{select, Either};
use core::borrow::BorrowMut;
use core::cell::RefCell;
use alloc::borrow::Cow;


const STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::WHITE)
                .stroke_width(1)
                .fill_color(Rgb565::CSS_CADET_BLUE)
                .build();
const CURSOR_WIDTH: i32 = 4;


pub fn cursor_right(rightmost_point: Point) -> Styled<Triangle, PrimitiveStyle<Rgb565>> {
    Triangle::new(Point::new(0, 0), Point::new(-CURSOR_WIDTH, -CURSOR_WIDTH), Point::new(-CURSOR_WIDTH, CURSOR_WIDTH))
            .translate(rightmost_point)
            .into_styled(STYLE)
}

pub fn cursor_left(leftmost_point: Point) -> Styled<Triangle, PrimitiveStyle<Rgb565>> {
    Triangle::new(Point::new(0, 0), Point::new(CURSOR_WIDTH, -CURSOR_WIDTH), Point::new(CURSOR_WIDTH, CURSOR_WIDTH))
            .translate(leftmost_point)
            .into_styled(STYLE)
}

pub fn static_draw_two_cursors<S: DrawTarget<Color = Rgb565>>(center: Point, width: i32, s: &mut S) -> Result<(Rectangle, Rectangle), <S as DrawTarget>::Error> {
    let left_cursor = cursor_right(center - Point::new(width, 0));
    let right_cursor = cursor_left(center + Point::new(width, 0));
    left_cursor.draw(s)?;
    right_cursor.draw(s)?;
    Ok((left_cursor.bounding_box(), right_cursor.bounding_box()))
}

pub struct SetupMethodState {
    undraw_tooltip: [Rectangle; 2]
}

const TOOLTIPS_TOP: [&'static LanguageString; 2] = [&TEXT_TOOLTIP_WIFI_SETUP, &TEXT_TOOLTIP_STANDALONE_SETUP];
impl SetupMethodState {
    pub fn new() -> Self {
        SetupMethodState {
            undraw_tooltip: [Rectangle::zero(); 2]
        }
    }

    fn draw_tooltip(&mut self, current_tooltip: u8, lang: Lang, io_handles: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let tooltip_top = Text::with_alignment(TOOLTIPS_TOP[current_tooltip as usize][lang], Point::new(64, 90), &io_handles.font, Alignment::Center);
        self.undraw_tooltip[0] = tooltip_top.bounding_box();
        tooltip_top.draw(io_handles.screen.borrow_mut())?;
        Ok(())
    }
}

const CURSOR_YS: [i32; 2] = [47, 57];
const CURSOR_WIDTHS: [i32; 2] = [34, 44];
impl SetupMethodState {
    fn draw_menu_base(&mut self, current_selection: u8, lang: Lang, theme: &Theme, io_handles: &mut IOHandles<'_>) -> anyhow::Result<()> {
        io_handles.font.bgcol = theme.bg();
        Text::with_alignment(TEXT_SETUP_PB[lang], Point::new(64, 20), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
        io_handles.font.fgcol = if current_selection == 0 { theme.highlight() } else { theme.fg() };
        Text::with_alignment(TEXT_WIFI_SETUP[lang], Point::new(64, 50), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
        io_handles.font.fgcol = if current_selection == 1 { theme.highlight() } else { theme.fg() };
        Text::with_alignment(TEXT_STANDALONE_SETUP[lang], Point::new(64, 65), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
        io_handles.font.fgcol = theme.fg();
        Text::with_alignment(TEXT_BACK[lang], Point::new(8, 150), &io_handles.font, Alignment::Left).draw(io_handles.screen.borrow_mut())?;
        Text::with_alignment(TEXT_NEXT[lang], Point::new(120, 150), &io_handles.font, Alignment::Right).draw(io_handles.screen.borrow_mut())?;
        Line::new(Point::new(0, 40), Point::new(128, 40))
            .into_styled(PrimitiveStyle::with_stroke(theme.fg().into(), 1))
            .draw(io_handles.screen.borrow_mut())?;
        Line::new(Point::new(0, 54), Point::new(128, 54))
            .into_styled(PrimitiveStyle::with_stroke(theme.fg().into(), 1))
            .draw(io_handles.screen.borrow_mut())?;
        Line::new(Point::new(0, 68), Point::new(128, 68))
            .into_styled(PrimitiveStyle::with_stroke(theme.fg().into(), 1))
            .draw(io_handles.screen.borrow_mut())?;

        self.draw_tooltip(current_selection, lang, io_handles)?;
        Ok(())
    }

    pub(crate) async fn run(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let (theme, lang) = {
            let settings = &PB_GLOBAL_SETTINGS.read().await;
            (settings.theme, settings.lang)
        };
        RefCell::borrow_mut(&io_handles.font.font).set_size(FontSize::Sz7)?;
        io_handles.font.fgcol = theme.fg();

        let black = PrimitiveStyleBuilder::new()
                .fill_color(theme.bg().as_rgb565())
                .build();

        let mut current_selection: u8 = 0;
        self.draw_menu_base(current_selection, lang, &theme, io_handles)?;

        loop {
            let result = select(
                io_handles.rotenc.receive(0),
                io_handles.button.receive(),
            ).await;
            match result {
                Either::First(r) => {
                    current_selection = ((current_selection as i16 + r).rem_euclid(2)) as u8;
                    for bb in self.undraw_tooltip.iter() {
                        bb.into_styled(black).draw(io_handles.screen.borrow_mut())?;
                    }
                    self.draw_tooltip(current_selection, lang, io_handles)?;
                    io_handles.font.fgcol = if current_selection == 0 { theme.highlight() } else { theme.fg() };
                    Text::with_alignment(TEXT_WIFI_SETUP[lang], Point::new(64, 50), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
                    io_handles.font.fgcol = if current_selection == 1 { theme.highlight() } else { theme.fg() };
                    Text::with_alignment(TEXT_STANDALONE_SETUP[lang], Point::new(64, 65), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
                    io_handles.font.fgcol = theme.fg();
                },
                Either::Second(b) => {
                    match (&b.button_type, &b.event) {
                        (ButtonType::Right, ButtonEventKind::Down) |
                        (ButtonType::Rotenc, ButtonEventKind::Down) => {
                            if current_selection == 0 {
                                ComponentMenu::draw_error_dialogue(Cow::Borrowed("no.\n\nThis option is not yet\nimplemented!"), &theme, io_handles)?;
                                while io_handles.button.receive().await.event != ButtonEventKind::Down {}
                                io_handles.screen.clear(theme.bg().as_rgb565())?;
                                self.draw_menu_base(current_selection, lang, &theme, io_handles)?;
                            } else {
                                return Ok(MenuSignal::Transition(Menu::ComponentMenu(ComponentMenu::Wifi)))
                            }
                        },
                        (ButtonType::Left, ButtonEventKind::Down) => return Ok(MenuSignal::Back),
                        _ => {}
                    }
                }
            }
            // draw the cursors
        }
    }
}