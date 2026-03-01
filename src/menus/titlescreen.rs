use crate::menu::{MenuSignal, MenuBehaviour, IOHandles};
use crate::event::Event;
use crate::lang::{Lang, TEXT_WELCOME, TEXT_PRESSENC, TEXT_WELCOME_A};
// use ilidriver::ILIDriver;
use fontfile::FontSize;
use log::info;

pub struct TitleState {
    cur_lang: Lang,
    counter: u8,
}

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
            io_handles.font.set_size(FontSize::Sz24)?;
            // io_handles.screen.draw_string(60, 240-195, TEXT_WELCOME[self.cur_lang], &mut io_handles.font, 0)?;
            // io_handles.screen.draw_string(60, 240-154, TEXT_WELCOME_A[self.cur_lang], &mut io_handles.font, 0)?;
            io_handles.font.set_size(FontSize::Sz14)?;
            // io_handles.screen.draw_string(60, 230, TEXT_PRESSENC[self.cur_lang], &mut io_handles.font, 0)?;
            self.cur_lang = next_lang(self.cur_lang);
            self.counter = 20;
        }
        self.counter -= 1;

        if let Some(Event::Button(v)) = events.pop() {
            if v.pin == 18 {
                info!("next menu!");
            }
        }
        Ok(MenuSignal::None)
    }
}
