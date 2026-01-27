use esp_idf_hal::task::queue::Queue;
use crate::event::Event;
use crate::menus::titlescreen::{TitleState};
use ilidriver::ILIDriver;
use pwm::OutputCtl;
use ledc::LedController;
use std::sync::Arc;
use fontfile::PbFont;

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

pub struct IOHandles<'a> {
    pub screen: ILIDriver<'a>, 
    pub leddriver: LedController,
    pub pwm_output: OutputCtl<'a>,
    pub font: PbFont, 
}

impl<'a> IOHandles<'a> {
    pub fn new(screen: ILIDriver<'a>, leddriver: LedController, pwm_output: OutputCtl<'a>, font: PbFont) -> IOHandles<'a> {
        Self { screen, leddriver, pwm_output, font }
    }
}

pub trait MenuBehaviour: Sized {
    // type Args: Into<Self> + From<MenuSelection>;
    fn init(&mut self, io_handles: &mut IOHandles) -> anyhow::Result<MenuSignal>;
    fn update(&mut self, io_handles: &mut IOHandles, events: &mut Vec<Event>) -> anyhow::Result<MenuSignal>;
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
    fn init(&mut self, io_handles: &mut IOHandles) -> anyhow::Result<MenuSignal> {
        match self {
            MenuStates::Title(t) => t.init(io_handles),
        }
    }

    fn update(&mut self, io_handles: &mut IOHandles, events: &mut Vec<Event>) -> anyhow::Result<MenuSignal> {
        match self {
            MenuStates::Title(t) => t.update(io_handles, events),
        }
    }
}

pub fn run_menu_loop(start_menu: MenuSelection, io_handles: &mut IOHandles, q: Arc<Queue<Event>>) -> anyhow::Result<()> {

    let mut cur_menu: MenuStates = start_menu.into();
    let mut events = vec![];
    cur_menu.init(io_handles)?;

    loop {
        // q.recv(10);
        if let Some((ev, _)) = q.recv_front(10) {
            events.push(ev);
        }
        let response = cur_menu.update(io_handles, &mut events)?;
        match response {
            MenuSignal::Transition(m) => { 
                cur_menu = m.into();
                cur_menu.init(io_handles)?;
            },
            MenuSignal::Return => { return Ok(()); },
            _ => {}
        }
        events.clear();
    }
}