use esp_idf_hal::task::queue::Queue;
use crate::event::Event;
use crate::menus::titlescreen::{TitleState};
use std::sync::Arc;

pub enum MenuSignal {
    None,
    Return,
    Transition(MenuSelection)
}

pub enum MenuSelection {
    TitleMenu,
}

enum MenuStates {
    Title(TitleState),
}

pub trait MenuBehaviour: Sized {
    // type Args: Into<Self> + From<MenuSelection>;
    fn update(&mut self, events: &Vec<Event>) -> anyhow::Result<MenuSignal>;
}

impl From<MenuSelection> for MenuStates {
    fn from(val: MenuSelection) -> MenuStates {
        match val {
            MenuSelection::TitleMenu => MenuStates::Title(TitleState::new()),
        }
    }
}

impl MenuBehaviour for MenuStates {
    // Args = MenuSelection;
    fn update(&mut self, events: &Vec<Event>) -> anyhow::Result<MenuSignal> {
        match self {
            MenuStates::Title(t) => t.update(events),
        }
    }
}

pub fn run_menu_loop(start_menu: MenuSelection, q: Arc<Queue<Event>>) -> anyhow::Result<()> {

    let mut cur_menu: MenuStates = start_menu.into();
    let mut events = vec![];

    loop {
        // q.recv(10);
        if let Some((ev, _)) = q.recv_front(10) {
            events.push(ev);
        }
        let response = cur_menu.update(&events)?;
        match response {
            MenuSignal::Transition(m) => { cur_menu = m.into(); },
            MenuSignal::Return => { return Ok(()); },
            _ => {}
        }
        events.clear();
    }
}