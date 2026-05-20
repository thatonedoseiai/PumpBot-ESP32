//! This submodule defines the font rendering behaviour following the trait defined in
//! [embedded_graphics]. Without this submodule, our font would be incompatible with the display
//! driving method used in `embedded_graphics`.

use embedded_graphics::{
    text::{
        renderer::{TextMetrics, CharacterStyle, TextRenderer},
        Baseline
    }, 
    pixelcolor::{Rgb565, Rgb888},
    geometry::Point,
    prelude::{DrawTarget, RgbColor},
    image::{Image, ImageRaw},
    Drawable,
    draw_target::DrawTargetExt,
};
use az::SaturatingAs;
use std::cell::RefCell;
use std::rc::Rc;

use crate::PbFont;

/// Defines the global state of the renderer. Font size and everything is already included in
/// `PbFont`, so we only need to add `bgcol` (background colour) and `fgcol` (foreground colour).
#[derive(Clone)]
pub struct PbFontRenderer {
    // need a cell or something
    pub font: Rc<RefCell<PbFont>>,
    bgcol: Option<Rgb888>,
    fgcol: Rgb888,
}

impl PbFontRenderer {
    /// Creates a new instance of the renderer state from an existing instance of the open font.
    /// When reopening a new font, the `PbFont` doesn't get reloaded so the renderer can continue
    /// to be universal.
    pub fn new(font: PbFont) -> Self {
        PbFontRenderer {
            font: Rc::new(RefCell::new(font)),
            bgcol: Some(Rgb888::GREEN),
            fgcol: Rgb888::BLUE,
        }
    }
}

impl TextRenderer for PbFontRenderer {
    type Color = Rgb565;

    fn draw_string<D>(
        &self,
        text: &str,
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
       where D: DrawTarget<Color = Self::Color> {
        let mut start_char_point = position;
        for c in text.encode_utf16() {
            let (metrics, coldata) = self.font.borrow_mut().load_char(c).unwrap(); // TODO: fix this!
            if metrics.width != 0 {
                let true_height = metrics.height / 3;
                // let bottom_right = start_char_point + Point::new((metrics.width - 1).into(), (metrics.y.saturating_sub_unsigned(metrics.height - 1)).into());
                // println!("drawing letter {:?} {:?}", std::char::from_u32(c as u32), metrics);
                let byteslice = coldata.into_iter()
                                       .map(|x| -> [u8;3] { x.into() })
                                       .flatten()
                                       .collect::<Vec<u8>>();
                let rawimage = ImageRaw::<Rgb888>::new(
                    &byteslice.as_slice(), metrics.width.into());
                println!("character {:?} metrics {:?}", char::from_u32(c as u32), &metrics);
                let image = Image::new(&rawimage, start_char_point - Point::new(0, metrics.y.into()));
                image.draw(&mut target.color_converted())?;
                // target.draw_iter(
                //     start_char_point.y,
                //     start_char_point.x,
                //     bottom_right.y,
                //     bottom_right.x,
                //     coldata.as_slice())?;
            }
            start_char_point += Point::new(metrics.advance.into(), 0);
        }

        Ok(start_char_point)
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
       where D: DrawTarget<Color = Self::Color> {
        Ok(position + Point::new(width.saturating_as(), 0))
    }

    /// this function is unimplemented. We need to implement it.
    fn measure_string(
        &self,
        text: &str,
        position: Point,
        baseline: Baseline,
    ) -> TextMetrics {
        todo!();
    }

    /// this function currently returns a constant. This is a TODO.
    fn line_height(&self) -> u32 {
        return 30u32;
        // todo!();
    }
}

