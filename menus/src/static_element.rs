use fontfile::{PbFont, pb_font_renderer::PbFontRenderer, FontSize, FontFileError};
use global_settings::{lang::LanguageString, ThemedColor, PB_GLOBAL_SETTINGS};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
};
use crate::screen::screen::Screen;

pub enum StaticElement {
    Text(Point, FontSize, &'static LanguageString, ThemedColor, Alignment),
}

impl StaticElement {
    pub async fn draw(&self, f: &mut PbFontRenderer, d: &mut Screen<'_>) -> anyhow::Result<()> {
        match self {
            StaticElement::Text(p, fs, s, t, a) => {
                let lang = &PB_GLOBAL_SETTINGS.read().await.lang;
                let text = s[*lang];
                f.font.borrow_mut().set_size(*fs)?;
                f.fgcol = t.get().await;
                Text::with_alignment(text, *p, &*f, *a).draw(d)?;
                Ok(())
            }
        }
    }
}
