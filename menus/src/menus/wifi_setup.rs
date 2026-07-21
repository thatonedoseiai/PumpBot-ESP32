use crate::{MenuSignal, MenuBehaviour, IOHandles, Event, MenuSelection};
use crate::menus::components::static_draw_two_cursors;
use rotenc::Direction;
use button_idf::ButtonType;
use fontfile::FontSize;
use global_settings::{
    lang::{
        Lang,
        LanguageString,
        TEXT_SEARCHING,
        TEXT_WIFI_SETTINGS,
    }, 
    PB_GLOBAL_SETTINGS};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{Rectangle, PrimitiveStyleBuilder, PrimitiveStyle, Line}
};
use embassy_futures::select::{select, Either};
use embassy_executor::Spawner;

pub struct SetupWifiState { }

impl SetupWifiState {
    pub fn new() -> Self {
        SetupWifiState { }
    }
}

impl MenuBehaviour for SetupWifiState {
    async fn init(&mut self, spawner: Spawner, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        io_handles.font.font.borrow_mut().set_size(FontSize::Sz7)?;
        Text::with_alignment(TEXT_WIFI_SETTINGS[Lang::En], Point::new(64, 20), io_handles.font.clone(), Alignment::Center).draw(io_handles.screen.borrow_mut())?;

        Ok(MenuSignal::None)
    }

    async fn update(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
    }
}