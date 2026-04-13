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

use crate::PbFont;

pub struct PbFontRenderer {
    // need a cell or something
    font: RefCell<PbFont>,
    bgcol: Rgb888,
    fgcol: Rgb888,
}

impl PbFontRenderer {
    pub fn new(font: PbFont) -> Self {
        PbFontRenderer {
            font: RefCell::new(font),
            bgcol: Rgb888::BLACK,
            fgcol: Rgb888::WHITE,
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
                let bottom_right = start_char_point + Point::new((metrics.width - 1).into(), (metrics.height - 1).into());
                let byteslice = coldata.into_iter()
                                       .map(|x| -> [u8;3] { x.into() })
                                       .flatten()
                                       .collect::<Vec<u8>>();
                let rawimage = ImageRaw::<Rgb888>::new(
                    &byteslice.as_slice(), metrics.width.into());
                let image = Image::new(&rawimage, start_char_point - Point::new(0, true_height.into()));
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

    fn measure_string(
        &self,
        text: &str,
        position: Point,
        baseline: Baseline,
    ) -> TextMetrics {
        todo!();
    }

    fn line_height(&self) -> u32 {
        return 30u32;
        // todo!();
    }
}

