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
    prelude::{DrawTarget, RgbColor, Size},
    image::{Image, ImageRaw},
    Drawable,
    draw_target::DrawTargetExt,
    primitives::Rectangle,
};
use esp_idf_hal::gpio::{Gpio14, PinDriver};
use az::SaturatingAs;
use std::cell::RefCell;
use std::rc::Rc;

use crate::PbFont;
use profiler::{SpanGuard, timed};

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

        let mut DBG_PINDRIVER = unsafe {
            PinDriver::output(Gpio14::new()).unwrap()
        };

        for c in text.encode_utf16() {

            DBG_PINDRIVER.set_low().unwrap();

            let (metrics, coldata) = timed!("load char", { 
                self.font.borrow_mut().load_char(c).unwrap() // TODO: fix this!
            });
            if metrics.width != 0 {
                let true_height = timed!("true_height = ", metrics.height / 3);
                // let bottom_right = start_char_point + Point::new((metrics.width - 1).into(), (metrics.y.saturating_sub_unsigned(metrics.height - 1)).into());
                let byteslice = timed!("set byteslice", {
                                coldata.into_iter()
                                       .map(|x| -> [u8;3] { x.into() })
                                       .flatten()
                                       .collect::<Vec<u8>>()
                    });
                let rawimage = timed!("convert to ImageRaw", {
                    ImageRaw::<Rgb888>::new(
                    &byteslice.as_slice(), metrics.width.into())
                });
                // println!("character {:?} metrics {:?}", char::from_u32(c as u32), &metrics);
                let image = Image::new(&rawimage, start_char_point - Point::new(0, metrics.y.into()));
                timed!("draw char as image", {
                    image.draw(&mut timed!("convert char color", target.color_converted()))?
                });
                // target.draw_iter(
                //     start_char_point.y,
                //     start_char_point.x,
                //     bottom_right.y,
                //     bottom_right.x,
                //     coldata.as_slice())?;
            }
            start_char_point += Point::new(metrics.advance.into(), 0);

            DBG_PINDRIVER.set_high().unwrap();

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
        // let mut bounding_box = Rectangle::new(position, Size::new(0, 0));
        let mut bb_top_left = position;
        let mut bb_bottom_right = position;
        let mut start_char_point = position;
        for c in text.encode_utf16() {
            let metrics = self.font.borrow_mut().load_char_metadata(c).unwrap(); // TODO: fix this!
            let top_left = start_char_point - Point::new(0, metrics.y.into());
            let bottom_right = top_left + Size::new(metrics.width.into(), (metrics.height / 3).into());
            if top_left.x < bb_top_left.x {
                bb_top_left.x = top_left.x;
            }
            if top_left.y < bb_top_left.y {
                bb_top_left.y = top_left.y;
            }
            if bottom_right.x > bb_bottom_right.x {
                bb_bottom_right.x = bottom_right.x;
            }
            if bottom_right.y > bb_bottom_right.y {
                bb_bottom_right.y = bottom_right.y;
            }
            start_char_point += Point::new(metrics.advance.into(), 0);
        }
        let bounding_box = Rectangle::with_corners(bb_top_left, bb_bottom_right);
        TextMetrics {
            bounding_box, 
            next_position: start_char_point,
        }
        // todo!();
    }

    /// this function currently returns a constant. This is a TODO.
    fn line_height(&self) -> u32 {
        return 30u32;
        // todo!();
    }
}

