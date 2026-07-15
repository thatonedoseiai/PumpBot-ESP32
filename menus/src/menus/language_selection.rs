use crate::{MenuSignal, MenuBehaviour, IOHandles, Event, MenuSelection};
use rotenc::Direction;
use button_idf::ButtonType;
use fontfile::FontSize;
use global_settings::lang::{Lang, TEXT_LANGUAGE_NAME, TEXT_LANGUAGE, TEXT_CHOOSE_LANG, TEXT_NEXT, TEXT_BACK};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{Rectangle, PrimitiveStyleBuilder, PrimitiveStyle, Line}
};
use embassy_futures::select::{select, Either};
use embassy_executor::Spawner;

pub struct LanguageState {
    cur_lang: Lang,
    undraw_language: Rectangle
}

impl LanguageState {
    pub fn new() -> Self {
        LanguageState {
            cur_lang: Lang::En,
            undraw_language: Rectangle::zero(),
        }
    }
}

impl MenuBehaviour for LanguageState {
    async fn init(&mut self, spawner: Spawner, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        io_handles.font.font.borrow_mut().set_size(FontSize::Sz12)?;

        Text::with_alignment(TEXT_CHOOSE_LANG[Lang::En], Point::new(64, 20), io_handles.font.clone(), Alignment::Center).draw(&mut io_handles.screen)?;
        Text::with_alignment(TEXT_LANGUAGE[Lang::En], Point::new(8, 50), io_handles.font.clone(), Alignment::Left).draw(&mut io_handles.screen)?;
        Text::with_alignment(TEXT_BACK[Lang::En], Point::new(8, 150), io_handles.font.clone(), Alignment::Left).draw(&mut io_handles.screen)?;
        Text::with_alignment(TEXT_NEXT[Lang::En], Point::new(120, 150), io_handles.font.clone(), Alignment::Right).draw(&mut io_handles.screen)?;

        Ok(MenuSignal::None)
    }

    async fn update(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let mut draw = false;
        loop {
            if draw {
                let language_name = Text::with_alignment(TEXT_LANGUAGE_NAME[self.cur_lang], Point::new(120, 50), io_handles.font.clone(), Alignment::Right);
                self.undraw_language = language_name.bounding_box();
                language_name.draw(&mut io_handles.screen)?;
                draw = false;
            }
            let result = select(
                io_handles.rotenc.receive(),
                io_handles.button.receive(),
            ).await;
            match result {
                Either::First(r) => {
                    match r.dir {
                        Direction::Clockwise => {
                            self.cur_lang = next_lang(self.cur_lang);
                            draw = true;
                        },
                        Direction::Anticlockwise => {
                            self.cur_lang = prev_lang(self.cur_lang);
                            draw = true;
                        },
                        _ => {}
                    };
                },
                Either::Second(b) => {
                    match b.button_type {
                        ButtonType::Right => return Ok(MenuSignal::Return),
                        ButtonType::Left => return Ok(MenuSignal::Transition(MenuSelection::TitleMenu)),
                        _ => {}
                    };
                }
            }
        }
    }
}

/// A helper function that determines the order of the language swaps. Russian is currently unused
/// because it was causing problems due to drawing offscreen.
const fn next_lang(cur: Lang) -> Lang {
    match cur {
        Lang::En => Lang::Jp,
        Lang::Jp => Lang::Fr,
        Lang::Fr => Lang::Es,
        Lang::Es => Lang::Pt,
        Lang::Pt => Lang::Zh,
        Lang::Zh => Lang::Cn,
        Lang::Cn => Lang::Ru,
        Lang::Ru => Lang::De,
        Lang::De => Lang::En,
    }
}

/// A helper function that determines the order of the language swaps. Russian is currently unused
/// because it was causing problems due to drawing offscreen.
const fn prev_lang(cur: Lang) -> Lang {
    match cur {
        Lang::Jp => Lang::En,
        Lang::Fr => Lang::Jp,
        Lang::Es => Lang::Fr,
        Lang::Pt => Lang::Es,
        Lang::Zh => Lang::Pt,
        Lang::Cn => Lang::Zh,
        Lang::Ru => Lang::Cn,
        Lang::De => Lang::Ru,
        Lang::En => Lang::De,
    }
}