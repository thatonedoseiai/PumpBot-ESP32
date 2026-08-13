use crate::{MenuSignal, IOHandles};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment}
};
use global_settings::PB_GLOBAL_SETTINGS;

pub struct HomeMenu {
    
}

impl HomeMenu {
    pub const fn new() -> Self {
        HomeMenu {

        }
    }

    pub async fn run(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        h.screen.clear(theme.bg().as_rgb565())?;
        h.font.fgcol = theme.fg();
        h.font.bgcol = theme.bg();
        loop {
            let text = Text::with_alignment("menu", Point::new(10, 20), &h.font, Alignment::Left);
            text.draw(&mut h.screen)?;
        }
    }
}
