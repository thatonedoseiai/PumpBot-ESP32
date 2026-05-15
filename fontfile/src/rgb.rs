//! This submodule defines several utility functions for handling RGB colours with 24bbp bitdepth. 

use std::slice::from_raw_parts;

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
