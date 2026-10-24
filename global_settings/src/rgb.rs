//! This submodule defines several utility functions for handling RGB colours with 24bbp bitdepth. 

use core::slice::from_raw_parts;
use core::ops::{Add, Sub, Mul};
use core::num::Saturating;
use crate::rgb;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};

#[derive(Debug, Clone, Copy)]
pub enum ColorConversionError {
    NonHomogeneousIterator,
    TooMuchColorData,
}

/// The main structure of RGB colours used throughout the project.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

/// Converting from a struct to a slice of bytes. Useful for arrays of RGB and doing vertical
/// decoding.
impl From<RGB> for [u8;3] {
    fn from(val: RGB) -> [u8;3] {
        [val.r, val.g, val.b]
    }
}

impl From<RGB> for Rgb565 {
    fn from(val: RGB) -> Rgb565 {
        Rgb565::new(val.r >> 3, val.g >> 2, val.b >> 3)
    }
}

impl From<RGB> for Rgb888 {
    fn from(val: RGB) -> Rgb888 {
        Rgb888::new(val.r, val.g, val.b)
    }
}

impl Add for RGB {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        rgb![
            (Saturating(self.r) + Saturating(other.r)).0, 
            (Saturating(self.g) + Saturating(other.g)).0, 
            (Saturating(self.b) + Saturating(other.b)).0
        ]
    }
}

impl Sub for RGB {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        rgb![
            (Saturating(self.r) - Saturating(other.r)).0,
            (Saturating(self.g) - Saturating(other.g)).0,
            (Saturating(self.b) - Saturating(other.b)).0
        ]
    }
}

impl Mul for RGB {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        rgb![
            (Saturating(self.r) * Saturating(other.r)).0,
            (Saturating(self.g) * Saturating(other.g)).0,
            (Saturating(self.b) * Saturating(other.b)).0
        ]
    }
}

impl Mul<u8> for RGB {
    type Output = Self;

    fn mul(self, other: u8) -> Self {
        rgb![self.r * other, self.g * other, self.b * other]
    }
}

#[derive(Clone, Copy)]
enum RgbComponent {
    Red,
    Green,
    Blue,
}

/// convert a slice of RGB to a slice of bytes. Useful for arrays of RGB and doing vertical
/// decoding.
impl RGB {
    pub fn to_byte_array(data: &[RGB]) -> Result<&[u8], ColorConversionError> {
        let len = data.len().checked_mul(3).ok_or(ColorConversionError::TooMuchColorData)?;
        let ptr = data.as_ptr().cast();
        let new: &[u8] = unsafe {
            from_raw_parts(ptr, len)
        };
        Ok(new)
    }

    pub const fn luminance(&self) -> u16 {
        ((self.r as u16 * 77) + 
        (self.g as u16 * 150) + 
        (self.b as u16 * 29)) >> 8
    }

    pub const fn as_rgb565(&self) -> Rgb565 {
        Rgb565::new(self.r >> 3, self.g >> 2, self.b >> 3)
    }

    pub const fn lerp(a: &RGB, b: &RGB, y: f32) -> RGB {
        let r = (a.r as f32 * (1.0 - y) + b.r as f32 * y) as u8;
        let g = (a.g as f32 * (1.0 - y) + b.g as f32 * y) as u8;
        let b = (a.b as f32 * (1.0 - y) + b.b as f32 * y) as u8;
        rgb![r, g, b]
    }

    pub const fn hue(&self) -> f32 {
        let rgb_float = [
            (self.r as f32 / 255.0, RgbComponent::Red),
            (self.g as f32 / 255.0, RgbComponent::Green),
            (self.b as f32 / 255.0, RgbComponent::Blue),];
        let (xmax, xmin, max_component) = {
            let mut xmax = rgb_float[0].0;
            let mut xmin = rgb_float[0].0;
            let mut max_component = RgbComponent::Red;
            let mut k = 0;
            while k < 3 {
                let i = rgb_float[k];
                if i.0 > xmax { xmax = i.0; max_component = i.1; }
                if i.0 < xmin { xmin = i.0; }
                k += 1;
            }
            (xmax, xmin, max_component)
        };
        let chroma = xmax - xmin;
        if chroma == 0.0 {
            return 0.0;
        }
        match max_component {
            RgbComponent::Red => 60.0 * (((rgb_float[1].0 - rgb_float[2].0) / chroma) % 6.0),
            RgbComponent::Green => 60.0 * (((rgb_float[2].0 - rgb_float[0].0) / chroma) + 2.0),
            RgbComponent::Blue => 60.0 * (((rgb_float[0].0 - rgb_float[1].0) / chroma) + 4.0),
        }
    }
}

pub const fn hsv_to_rgb(h: f32, s: f32, v: f32) -> RGB {
    let chroma = v * s;
    let x = chroma * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - chroma;
    let (r1, g1, b1) = 
        if h < 60.0 { (chroma, x, 0.0) }
        else if h < 120.0 { (x, chroma, 0.0) }
        else if h < 180.0 { (0.0, chroma, x) }
        else if h < 240.0 { (0.0, x, chroma) }
        else if h < 300.0 { (x, 0.0, chroma) }
        else { (chroma, 0.0, x) };
    rgb![
        ((r1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).clamp(0.0, 255.0) as u8,
    ]
    // todo!()
}

/// The macro used to create RGB because I got really fed up with writing it out all the time.
#[macro_export]
macro_rules! rgb {
    [$red:expr, $green:expr, $blue:expr $(,)?] => {
        RGB {
            r: $red, g: $green, b: $blue
        }
    };
    [$val:expr] => {
        RGB {
            r: $val, g: $val, b: $val
        }
    }
}
