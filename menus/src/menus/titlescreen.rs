//! The definitions of the functionality for the title screen.
//! This menu will display the welcome text to pumpbot, while cycling the welcome text language at a
//! constant rate.

use crate::{MenuSignal, MenuBehaviour, IOHandles, Event, MenuSelection};
use global_settings::lang::{Lang, TEXT_WELCOME, TEXT_PRESSENC, TEXT_WELCOME_A};
// use ilidriver::ILIDriver;
use fontfile::FontSize;
use log::info;
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{Rectangle, PrimitiveStyleBuilder}
};
use profiler::{timed, SpanGuard};
use alloc::vec::Vec;

/// represents the internal state of the title screen - what language it's on and how long it has
/// until it swaps to a different language.
pub struct TitleState {
    cur_lang: Lang,
    counter: u8,
    undraw_bbs: [Rectangle;3]
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
            counter: 20,
            undraw_bbs: [Rectangle::zero(); 3]
        }
    }
}

impl MenuBehaviour for TitleState {
    // Args = ()
    fn init(&mut self, io_handles: &mut IOHandles) -> anyhow::Result<MenuSignal> {
        Ok(MenuSignal::None)
    }

    fn update(&mut self, io_handles: &mut IOHandles, events: &mut Vec<Event>) -> anyhow::Result<MenuSignal> {
        // info!("update loop iteration {}", self.counter);
        // self.counter = self.counter.wrapping_add(1);
        // if self.counter == 0 {
            timed!("Set font size", {
                io_handles.font.font.borrow_mut().set_size(FontSize::Sz12)?;
            });
            // io_handles.screen.clear(Rgb565::BLACK);
            let black = timed!("Create black style", {
                PrimitiveStyleBuilder::new()
                        .fill_color(Rgb565::BLACK)
                        .build()
            });

            timed!("Draw all screen clearing rects", {
                for rect in self.undraw_bbs {
                    rect.into_styled(black).draw(&mut io_handles.screen)?;
                }
            });

            let top_text = timed!("Create WELCOME", {
                Text::with_alignment(TEXT_WELCOME[self.cur_lang], Point::new(64, 30), io_handles.font.clone(), Alignment::Center)
            });
            self.undraw_bbs[0] = timed!("push box WELCOME", {
                top_text.bounding_box()
            });
            timed!("Draw WELCOME", {
                top_text.draw(&mut io_handles.screen)?
            });
            let mid_text = timed!("Create WELCOME 2", {
                Text::with_alignment(TEXT_WELCOME_A[self.cur_lang], Point::new(64, 50), io_handles.font.clone(), Alignment::Center)
            });
            self.undraw_bbs[1] = timed!("push box WELCOME 2", {
                mid_text.bounding_box()
            });
            timed!("draw WELCOME 2", {
                mid_text.draw(&mut io_handles.screen)?
            });
            timed!("Change font size", {
                io_handles.font.font.borrow_mut().set_size(FontSize::Sz7)?
            });
            let push_text = timed!("Create push rotenc to continue", {
                Text::with_alignment(TEXT_PRESSENC[self.cur_lang], Point::new(64, 140), io_handles.font.clone(), Alignment::Center)
            });
            self.undraw_bbs[2] = timed!("push box rotenc", {
                push_text.bounding_box()
            });
            timed!("Draw rotenc", {
                push_text.draw(&mut io_handles.screen)?
            });
            // push_text.draw(&mut io_handles.screen);
            self.cur_lang = next_lang(self.cur_lang);
            self.counter = 20;
        // }
        self.counter -= 1;

        if let Some(Event::Button(v)) = events.pop() {
            if v.pin == 18 {
                // info!("next menu!");
                return Ok(MenuSignal::Transition(MenuSelection::Unimplemented));
            }
        }
        Ok(MenuSignal::None)
    }
}
