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
