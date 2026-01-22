use crate::menu::{MenuSignal, MenuBehaviour};
use crate::event::Event;
use log::info;

pub struct TitleState {
    counter: u8,
}

impl TitleState {
    pub fn new() -> Self {
        TitleState {
            counter: 0,
        }
    }
}

impl MenuBehaviour for TitleState {
    // Args = ()
    fn update(&mut self, _events: &Vec<Event>) -> anyhow::Result<MenuSignal> {
        info!("update loop iteration {}", self.counter);
        self.counter = self.counter.wrapping_add(1);
        Ok(MenuSignal::None)
    }
}
