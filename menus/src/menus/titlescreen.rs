//! The definitions of the functionality for the title screen.
//! This menu will display the welcome text to pumpbot, while cycling the welcome text language at a
//! constant rate.

use crate::{MenuSignal, IOHandles, Menu, ComponentMenu};
use global_settings::lang::{Lang, TEXT_WELCOME, TEXT_PRESSENC, TEXT_WELCOME_A};
// use ilidriver::ILIDriver;
use fontfile::FontSize;
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    primitives::{Rectangle, PrimitiveStyleBuilder}
};
use profiler::{timed};
use button_idf::ButtonType;
use embassy_futures::select::{select, Either};
use embassy_time::{Timer, Duration};
use core::borrow::BorrowMut;
use core::cell::RefCell;
use global_settings::PB_GLOBAL_SETTINGS;

/// represents the internal state of the title screen - what language it's on and how long it has
/// until it swaps to a different language.
#[derive(Debug, Clone, Copy)]
pub struct TitleMenu;

pub struct TitleState {
    cur_lang: Lang,
    undraw_bbs: [Rectangle;3],
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
        Lang::Cn => Lang::De, // Lang::Ru,
        Lang::Ru => Lang::De,
        Lang::De => Lang::En,
    }
}

impl TitleState {
    pub fn new() -> Self {
        TitleState {
            cur_lang: Lang::En,
            undraw_bbs: [Rectangle::zero(); 3],
        }
    }
}

impl TitleState {
    pub(crate) async fn run(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        // info!("update loop iteration {}", self.counter);
        // self.counter = self.counter.wrapping_add(1);
        // if self.counter == 0 {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        io_handles.font.fgcol = theme.fg();
        io_handles.font.bgcol = theme.bg();
        loop {
            timed!("Set font size", {
                RefCell::borrow_mut(&io_handles.font.font).set_size(FontSize::Sz12)?;
            });
            // io_handles.screen.clear(Rgb565::BLACK);
            let black = timed!("Create black style", {
                PrimitiveStyleBuilder::new()
                        .fill_color(theme.bg().as_rgb565())
                        .build()
            });

            timed!("Draw all screen clearing rects", {
                for rect in self.undraw_bbs {
                    rect.into_styled(black).draw(io_handles.screen.borrow_mut())?;
                }
            });

            let top_text = timed!("Create WELCOME", {
                Text::with_alignment(TEXT_WELCOME[self.cur_lang], Point::new(64, 30), &io_handles.font, Alignment::Center)
            });
            self.undraw_bbs[0] = timed!("push box WELCOME", {
                top_text.bounding_box()
            });
            timed!("Draw WELCOME", {
                top_text.draw(io_handles.screen.borrow_mut())?
            });
            let mid_text = timed!("Create WELCOME 2", {
                Text::with_alignment(TEXT_WELCOME_A[self.cur_lang], Point::new(64, 50), &io_handles.font, Alignment::Center)
            });
            self.undraw_bbs[1] = timed!("push box WELCOME 2", {
                mid_text.bounding_box()
            });
            timed!("draw WELCOME 2", {
                mid_text.draw(io_handles.screen.borrow_mut())?
            });
            timed!("Change font size", {
                RefCell::borrow_mut(&io_handles.font.font).set_size(FontSize::Sz7)?
            });
            let push_text = timed!("Create push rotenc to continue", {
                Text::with_alignment(TEXT_PRESSENC[self.cur_lang], Point::new(64, 140), &io_handles.font, Alignment::Center)
            });
            self.undraw_bbs[2] = timed!("push box rotenc", {
                push_text.bounding_box()
            });
            timed!("Draw rotenc", {
                push_text.draw(io_handles.screen.borrow_mut())?
            });
            // push_text.draw(io_handles.screen.borrow_mut());
            self.cur_lang = next_lang(self.cur_lang);
            let result = select(
                Timer::after(Duration::from_secs(1)), 
                async {
                    while io_handles.button.receive().await.button_type != ButtonType::Rotenc { }
                }
            ).await;
            match result {
                Either::Second(_) => break,
                _ => {},
            };
        }
        // Ok(MenuSignal::Transition(MenuSelection::LanguageMenu)) // TODO: fix this
        let lang = PB_GLOBAL_SETTINGS.read().await.lang;
        Ok(MenuSignal::Transition(Menu::ComponentMenu(ComponentMenu::Lang)))
    }
}
