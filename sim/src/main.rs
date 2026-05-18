use menus::{run_menu_loop, MenuSelection, IOHandles, Event};
use embedded_graphics_simulator::{SimulatorDisplay, Window, OutputSettingsBuilder, SimulatorEvent};
use fontfile::{FontSize, PbFont, pb_font_renderer::PbFontRenderer};
use embedded_graphics::{
    prelude::*,
    pixelcolor::{Rgb565, raw::RawU16},
    primitives::{Triangle, Rectangle, PrimitiveStyle},
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    text::Text,
};

fn main() -> anyhow::Result<()> {
    let mut screen = SimulatorDisplay::<Rgb565>::new(Size::new(128,160));
    let mut font = PbFont::new();
    font.set_size(FontSize::Sz14)?;
    let pb_font_style = PbFontRenderer::new(font);
    let yoffset = 10;
    let thin_stroke = PrimitiveStyle::with_stroke(Rgb565::BLUE, 1);
    let res = Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(&mut screen);

    // let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello Rust!", Point::new(20, 50), pb_font_style).draw(&mut screen)?;
    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Hello World", &output_settings);//.update(&screen);
    loop {
        window.update(&screen);
        for event in window.events() {
            if event == SimulatorEvent::Quit {
                return Ok(());
            }
        }
    }
}
