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

/// represents the internal state of the title screen - what language it's on and how long it has
/// until it swaps to a different language.
pub struct TitleState {
    cur_lang: Lang,
    counter: u8,
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
        if self.counter == 0 {
            io_handles.font.font.borrow_mut().set_size(FontSize::Sz12)?;
            io_handles.screen.clear(Rgb565::BLACK);
            Text::with_alignment(TEXT_WELCOME[self.cur_lang], Point::new(64, 30), io_handles.font.clone(), Alignment::Center).draw(&mut io_handles.screen);
            Text::with_alignment(TEXT_WELCOME_A[self.cur_lang], Point::new(64, 50), io_handles.font.clone(), Alignment::Center).draw(&mut io_handles.screen);
            io_handles.font.font.borrow_mut().set_size(FontSize::Sz7)?;
            let push_text = Text::with_alignment(TEXT_PRESSENC[self.cur_lang], Point::new(64, 140), io_handles.font.clone(), Alignment::Center).draw(&mut io_handles.screen);
            // push_text.draw(&mut io_handles.screen);
            self.cur_lang = next_lang(self.cur_lang);
            self.counter = 20;
        }
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
