use crate::{MenuSignal, IOHandles};
use rotenc::Direction;
use button_idf::{ButtonType, ButtonEventKind};
use fontfile::FontSize;
use global_settings::{
    lang::{
        Lang,
        LanguageString,
        TEXT_SETUP_PB,
        TEXT_SETUP_PB_A,
        TEXT_WIFI_SETUP,
        TEXT_STANDALONE_SETUP,
        TEXT_TOOLTIP_WIFI_SETUP,
        TEXT_TOOLTIP_WIFI_SETUP_A,
        TEXT_TOOLTIP_STANDALONE_SETUP,
        TEXT_TOOLTIP_STANDALONE_SETUP_A,
        TEXT_NEXT,
        TEXT_BACK,
    }, 
    PB_GLOBAL_SETTINGS};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{Rectangle, PrimitiveStyleBuilder, PrimitiveStyle, Line, Triangle, Styled}
};
use embassy_futures::select::{select, Either};
use embassy_executor::Spawner;
use core::borrow::BorrowMut;
use core::cell::RefCell;


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
const TOOLTIPS_BOTTOM: [&'static LanguageString; 2] = [&TEXT_TOOLTIP_WIFI_SETUP_A, &TEXT_TOOLTIP_STANDALONE_SETUP_A];
impl SetupMethodState {
    pub fn new() -> Self {
        SetupMethodState {
            undraw_tooltip: [Rectangle::zero(); 2]
        }
    }

    fn draw_tooltip(&mut self, current_tooltip: u8, io_handles: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let tooltip_top = Text::with_alignment(TOOLTIPS_TOP[current_tooltip as usize][Lang::En], Point::new(64, 80), &io_handles.font, Alignment::Center);
        self.undraw_tooltip[0] = tooltip_top.bounding_box();
        tooltip_top.draw(io_handles.screen.borrow_mut())?;
        let tooltip_bottom = Text::with_alignment(TOOLTIPS_BOTTOM[current_tooltip as usize][Lang::En], Point::new(64, 90), &io_handles.font, Alignment::Center);
        self.undraw_tooltip[1] = tooltip_bottom.bounding_box();
        tooltip_bottom.draw(io_handles.screen.borrow_mut())?;
        Ok(())
    }
}

const CURSOR_YS: [i32; 2] = [47, 57];
const CURSOR_WIDTHS: [i32; 2] = [34, 44];
impl SetupMethodState {
    pub async fn run(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        RefCell::borrow_mut(&io_handles.font.font).set_size(FontSize::Sz7)?;

        RefCell::borrow_mut(&io_handles.font.font).set_size(FontSize::Sz7)?;
        Text::with_alignment(TEXT_SETUP_PB[Lang::En], Point::new(64, 20), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
        Text::with_alignment(TEXT_SETUP_PB_A[Lang::En], Point::new(64, 30), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
        Text::with_alignment(TEXT_WIFI_SETUP[Lang::En], Point::new(64, 50), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
        Text::with_alignment(TEXT_STANDALONE_SETUP[Lang::En], Point::new(64, 60), &io_handles.font, Alignment::Center).draw(io_handles.screen.borrow_mut())?;
        Text::with_alignment(TEXT_BACK[Lang::En], Point::new(8, 150), &io_handles.font, Alignment::Left).draw(io_handles.screen.borrow_mut())?;
        Text::with_alignment(TEXT_NEXT[Lang::En], Point::new(120, 150), &io_handles.font, Alignment::Right).draw(io_handles.screen.borrow_mut())?;
        Line::new(Point::new(0, 42), Point::new(128, 42))
            .into_styled(PrimitiveStyle::with_stroke(theme.fg().into(), 1))
            .draw(io_handles.screen.borrow_mut())?;
        Line::new(Point::new(0, 52), Point::new(128, 52))
            .into_styled(PrimitiveStyle::with_stroke(theme.fg().into(), 1))
            .draw(io_handles.screen.borrow_mut())?;
        Line::new(Point::new(0, 62), Point::new(128, 62))
            .into_styled(PrimitiveStyle::with_stroke(theme.fg().into(), 1))
            .draw(io_handles.screen.borrow_mut())?;

        let black = PrimitiveStyleBuilder::new()
                .fill_color(theme.bg().as_rgb565())
                .build();
        let mut current_selection: u8 = 0;
        self.draw_tooltip(current_selection, io_handles)?;
        let (mut lc_bb, mut rc_bb) = static_draw_two_cursors(Point::new(64, 47), 34, &mut io_handles.screen)?;

        loop {
            let result = select(
                io_handles.rotenc.receive(),
                io_handles.button.receive(),
            ).await;
            match result {
                Either::First(r) => {
                    if r.dir != Direction::None {
                        current_selection = (current_selection + 1) % 2;
                        for bb in [lc_bb, rc_bb, self.undraw_tooltip[0], self.undraw_tooltip[1]].iter() {
                            bb.into_styled(black).draw(io_handles.screen.borrow_mut())?;
                        }
                        log::info!("rotary encoder! {}", r);
                        self.draw_tooltip(current_selection, io_handles)?;
                        (lc_bb, rc_bb) = static_draw_two_cursors(Point::new(64, CURSOR_YS[current_selection as usize]), CURSOR_WIDTHS[current_selection as usize], &mut io_handles.screen)?;
                    }
                },
                Either::Second(b) => {
                    match (&b.button_type, &b.event) {
                        (ButtonType::Right, ButtonEventKind::Down) => return Ok(MenuSignal::Return),
                        (ButtonType::Left, ButtonEventKind::Down) => return Ok(MenuSignal::Back),
                        _ => {}
                    }
                }
            }
            // draw the cursors
        }
    }
}