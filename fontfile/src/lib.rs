#![feature(iter_array_chunks)]

mod rgb;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::io;
use std::fmt;
use std::error::Error;
pub use rgb::{RGB, ColorConversionError};
use log::info;

// Font file constants
const FONT_NAME_SIZE_12: &str = "NC_12.cbf";
const FONT_NAME_SIZE_14: &str = "NC_14.cbf";
const FONT_NAME_SIZE_18: &str = "NC_18.cbf";
const FONT_NAME_SIZE_24: &str = "NC_24.cbf";
const FONT_NAME_SIZE_42: &str = "NC_42.cbf";

pub struct PbFont {
    font_file: Option<File>,
    font_metadata: FontMetadata,
}

pub struct PbBg {
    bg_file: Option<File>,
    bg_filename: Option<String>,
    foreground_color: RGB,
    background_color: RGB,
}

#[derive(Debug, Clone, Copy)]
pub enum FontSize {
    Sz12,
    Sz14,
    Sz18,
    Sz24,
    Sz42,
}

impl From<FontSize> for u16 {
    fn from(val: FontSize) -> u16 {
        match val {
            FontSize::Sz12 => 12,
            FontSize::Sz14 => 14,
            FontSize::Sz18 => 18,
            FontSize::Sz24 => 24,
            FontSize::Sz42 => 42,
        }
    }
}

// Metadata structures
#[derive(Debug, Clone, Copy)]
pub struct FontMetadata {
    pub font_size: u16,
    pub num_glyphs: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CharMetadata {
    pub advance: u16,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub vertical: bool,
}

// Global color references (these should be initialized elsewhere)
// Correct magic bytes
const CORRECT_MBYTES: [u8; 10] = [0x63, 0x62, 0x66, 0xe5, 0x9c, 0xa7, 0xe5, 0xad, 0x97, 0x02];

// Error codes
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FontFileError {
    FileNotFound,
    BadFormat,
    BadSize,
    BadChar(u16, u16, u16),
    InvalidChar(u16),
    IndexOutOfBounds,
    IOError,
    FileNotOpen,
}

impl fmt::Display for FontFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FontFileError::FileNotFound => write!(f, "FF: FileNotFound"),
            FontFileError::BadFormat => write!(f, "FF: BadFormat"),
            FontFileError::BadSize => write!(f, "FF: BadSize"),
            FontFileError::BadChar(a, b, c) => write!(f, "FF: BadChar = char: {}, wxh {}x{}", a, b, c),
            FontFileError::InvalidChar(x) => write!(f, "FF: InvalidChar = {}", x),
            FontFileError::IndexOutOfBounds => write!(f, "FF: IndexOutOfBounds"),
            FontFileError::IOError => write!(f, "FF: IOError"),
            FontFileError::FileNotOpen => write!(f, "FF: FileNotOpen"),
        }
    }
}
impl Error for FontFileError { }

impl From<io::Error> for FontFileError {
    fn from(_: io::Error) -> FontFileError {
        FontFileError::IOError
    }
}

impl PbFont {
    pub fn new() -> Self {
        PbFont {
            font_file: None,
            font_metadata: FontMetadata {
                font_size: 0,
                num_glyphs: 0,
            },
        }
    }

    // Set font size
    pub fn set_size(&mut self, sz: FontSize) -> Result<(), FontFileError> {
        let font_name = format!("/fs/{}", match sz {
            FontSize::Sz12 => FONT_NAME_SIZE_12,
            FontSize::Sz14 => FONT_NAME_SIZE_14,
            FontSize::Sz18 => FONT_NAME_SIZE_18,
            FontSize::Sz24 => FONT_NAME_SIZE_24,
            FontSize::Sz42 => FONT_NAME_SIZE_42,
        });

        // Open new font file
        let mut file = File::open(font_name).map_err(|_| FontFileError::FileNotFound)?;
        Self::verify_font_file(&mut file)?;

        self.font_metadata = Self::read_header(&mut file)?;
        
        if self.font_metadata.font_size != u16::from(sz) {
            return Err(FontFileError::FileNotFound);
        }

        self.font_file = Some(file);

        Ok(())
    }

    // Verify font file
    fn verify_font_file(font: &mut File) -> Result<(), FontFileError> {
        let mut mbytes_buffer = [0u8; 10];
        font.read_exact(&mut mbytes_buffer)?;
        if mbytes_buffer != CORRECT_MBYTES {
            Err(FontFileError::BadFormat)
        } else {
            Ok(())
        }
    }

    // Read header
    fn read_header(font: &mut File) -> Result<FontMetadata, FontFileError> {
        font.seek(SeekFrom::Start(10)).map_err(|_| FontFileError::BadFormat)?;

        let mut buf = [0u8; 2];
        font.read_exact(&mut buf).map_err(|_| FontFileError::BadFormat)?;

        let mut buf1 = [0u8; 4];
        font.read_exact(&mut buf1).map_err(|_| FontFileError::BadFormat)?;
        Ok(FontMetadata {
            font_size: u16::from_le_bytes(buf),
            num_glyphs: u32::from_le_bytes(buf1)
        })
    }

    // Binary search, returns the offset into the file for the glyph
    fn binary_search(&mut self, k: u16) -> Result<u32, FontFileError> {
        let mut offset: i64 = (self.font_metadata.num_glyphs >> 1).into();
        if let Some(ref mut f) = self.font_file {
            f.seek(SeekFrom::Start((offset * 6 + 16) as u64))?;

            let mut buf = [0u8; 2];
            f.read_exact(&mut buf)?;
            let mut prev_entry = u16::from_le_bytes(buf);
            let mut curr_entry = prev_entry;

            while curr_entry != k {
                if prev_entry < k && curr_entry > k && offset == 1 {
                    return Err(FontFileError::InvalidChar(k));
                }
                
                offset >>= 1;
                if offset < 1 {
                    offset = 1;
                }
                
                offset = if curr_entry > k {
                    (-offset * 6 - 2) as i64
                } else {
                    (offset * 6 - 2) as i64
                };
                
                f.seek(SeekFrom::Current(offset))?;
                prev_entry = curr_entry;
                
                let mut buf = [0u8; 2];
                f.read_exact(&mut buf)?;
                curr_entry = u16::from_le_bytes(buf);
            }
            
            let mut buf = [0u8; 4];
            f.read_exact(&mut buf)?;
            Ok(u32::from_le_bytes(buf))
        } else {
            Err(FontFileError::FileNotOpen)
        }
    }

    // Load character
    pub fn load_char(&mut self, curchar: u16) -> Result<(CharMetadata, Vec<RGB>), FontFileError> {
        // let mut font_file = unsafe { FONT_FILE.take() }.unwrap();
        let offset = self.binary_search(curchar)?;

        let Some(ref mut font_file) = self.font_file else { return Err(FontFileError::FileNotOpen); };
        font_file.seek(SeekFrom::Start(offset as u64))?;

        let mut buf1 = [0u8; 2];
        font_file.read_exact(&mut buf1)?;
        let mut advance = u16::from_le_bytes(buf1);

        let mut buf1 = [0u8; 2];
        font_file.read_exact(&mut buf1)?;
        let x = i16::from_le_bytes(buf1);

        let mut buf1 = [0u8; 2];
        font_file.read_exact(&mut buf1)?;
        let y = i16::from_le_bytes(buf1);

        let mut buf1 = [0u8; 2];
        font_file.read_exact(&mut buf1)?;
        let width = u16::from_le_bytes(buf1);

        let mut buf1 = [0u8; 2];
        font_file.read_exact(&mut buf1)?;
        let height = u16::from_le_bytes(buf1);

        if curchar == 0x20 {
            return Ok((CharMetadata {
                advance: advance,
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                vertical: false,
            }, vec![]));
        }

        // let data_len = (unsafe { CM.width } * unsafe { CM.height } + 2) as usize;
        let data_len: usize = (width * height + 2) as usize;
        let mut data = vec![0u8; data_len];
        font_file.read_exact(&mut data)?;

        if width == 0 || height == 0 {
            // println!("BAD CHAR: {} has width {} height {}", curchar, unsafe { CM.width }, unsafe { CM.height });
            return Err(FontFileError::BadChar(curchar, width, height))
        }

        let vertical = (advance & 0x1000) != 0;
        advance &= 0x4fff;
        // unsafe { CM.vertical } = (unsafe { CM.advance } & 0x1000) >> 12;
        // unsafe { CM.advance } &= 0x4fff;

        let decompressed_len: usize = (width * height) as usize;
        // let mut decompressed = vec![0u8; decompressed_len];
        let decompressed;

        // info!("DATA: {:?}", data);
        if vertical {
            decompressed = decode_vert(&data, height, width);
        } else {
            decompressed = decode(&data);
        }
        // info!("DECOMPRESSED LEN: {:?}", decompressed.len());
        // info!("DECOMPRESSED: {:?}", decompressed);

        // Convert to RGB
        // info!("decompressed length: {}", decompressed.len());
        let mut rgb_vec = Vec::with_capacity(decompressed_len);
        for pixel in (&decompressed).iter().array_chunks::<3>() {
            rgb_vec.push(rgb![*pixel[0], *pixel[1], *pixel[2]]);
        }

        // buf = Some(rgb_vec);

        // FONT_FILE = Some(font_file);
        Ok((CharMetadata {
            advance, x, y, width, height, vertical,
        }, rgb_vec))
    }
}

impl PbBg {
    pub fn new() -> Self {
        PbBg {
            bg_file: None,
            bg_filename: None,
            foreground_color: rgb![0],
            background_color: rgb![0],
        }
    }

    // Load background image
    pub fn load_bgimg(&mut self, name: &str, force_load: bool, index: i32) -> Result<Vec<RGB>, FontFileError> {
        // let mut image_file = self.bg_file;

        // Check if we need to load a new file
        if force_load || self.bg_filename != Some(name.to_string()) {
            match File::open(name) {
                Ok(f) => {
                    self.bg_file = Some(f);
                }
                Err(_) => return Err(FontFileError::FileNotFound),
            }
        }

        if let Some(ref mut file) = self.bg_file {
            // Read header
            let mut header = [0u8; 3];
            match file.read_exact(&mut header) {
                Ok(_) => {
                    if header != [0x63, 0x62, 0x69] {  // "cbi" in ASCII
                        return Err(FontFileError::BadFormat);
                    }
                }
                Err(_) => return Err(FontFileError::FileNotFound),
            }

            let mut header_byte = [0u8; 1];
            match file.read_exact(&mut header_byte) {
                Ok(_) => {
                    if header_byte[0] != 2 {
                        return Err(FontFileError::BadSize);
                    }
                }
                Err(_) => return Err(FontFileError::FileNotFound),
            }

            // Read index
            let mut header_byte = [0u8; 1];
            match file.read_exact(&mut header_byte) {
                Ok(_) => {
                    if index >= header_byte[0] as i32 {
                        return Err(FontFileError::IndexOutOfBounds);
                    }
                }
                Err(_) => return Err(FontFileError::FileNotFound),
            }

            // Seek to image data offset
            file.seek(SeekFrom::Start(5 + 4 * index as u64))?;
            let mut offset_buf = [0u8; 4];
            file.read_exact(&mut offset_buf)?;
            let imagedata_offset = u32::from_le_bytes(offset_buf);

            file.seek(SeekFrom::Start(imagedata_offset as u64))?;

            // Read image properties
            let mut flags_buf = [0u8; 1];
            file.read_exact(&mut flags_buf)?;
            let flags = flags_buf[0];

            let mut width_buf = [0u8; 2];
            file.read_exact(&mut width_buf)?;
            let width = u16::from_le_bytes(width_buf);

            let mut height_buf = [0u8; 2];
            file.read_exact(&mut height_buf)?;
            let height = u16::from_le_bytes(height_buf);

            if width != 240 || height != 320 {
                // println!("width: {}, height: {}", width, height);
                return Err(FontFileError::BadSize);
            }

            let bitdepth = if flags & 0x1 != 0 { 1 } else { 3 };
            let compressed_len = (240 * 320 * bitdepth) as usize;
            let mut compressed = vec![0u8; compressed_len];
            file.read_exact(&mut compressed)?;

            let mut buf = Vec::new();
            if flags & 0x1 != 0 {
                // Decompress and blend
                let decompressed = decode(&compressed); // vec![0u8; 240 * 320];
                // decode(&compressed, &mut decompressed);

                for i in 0..(240 * 320) {
                    let alpha = decompressed[i] as u32;
                    let inv_alpha = 255 - alpha;

                    let fg = self.foreground_color;
                    let bg = self.background_color;

                    buf.push( rgb![
                        ((alpha * fg.r as u32 + inv_alpha * bg.r as u32) / 255) as u8,
                        ((alpha * fg.g as u32 + inv_alpha * bg.g as u32) / 255) as u8,
                        ((alpha * fg.b as u32 + inv_alpha * bg.b as u32) / 255) as u8,
                    ]);
                }
            } else {
                // Direct copy
                for i in 0..(240 * 320) {
                    buf.push( rgb![
                        compressed[i * 3],
                        compressed[i * 3 + 1],
                        compressed[i * 3 + 2]
                    ]);
                }
            }

            self.bg_filename = Some(name.to_string());

            Ok(buf)
        } else {
            Err(FontFileError::FileNotFound)
        }
    }

    // Initialize global colors
    pub fn init_colors(&mut self, foreground: RGB, background: RGB) {
        self.foreground_color = foreground;
        self.background_color = background;
    }
}

// Decode function
pub fn decode(indata: &[u8]) -> Vec<u8> {
    let mut ret = Vec::new();
    let mut i = 0;
    let mut j;
    let mut k = 0;

    while i < indata.len() && indata[i] != 0xff {
        if indata[i] & 0x80 != 0 {
            j = k + (indata[i] ^ 0x80) as usize;
            // info!("OPCODE {} COPY {} BYTES", indata[i], j-k);
            while k <= j {
                i += 1;
                ret.push(indata[i]);
                // outdata[k] = indata[i + 1];
                k += 1;
            }
            i += 1;
        } else {
            let count = (indata[i] + 2) as usize;
            // info!("OPCODE {} REPT {} {} times", indata[i], indata[i+1], count);
            for _ in 0..count {
                ret.push(indata[i+1]);
                // outdata[k] = indata[i + 1];
                k += 1;
            }
            i += 2;
        }
    }

    ret
    // k as i32
}

// Vertical decode function
pub fn decode_vert(indata: &[u8], width: u16, height: u16) -> Vec<u8> {
    let mut outdata: Vec<u8> = vec![0u8; (width * height) as usize]; // TODO: get rid of this
                                                                     // somehow and make the decode
                                                                     // vert properly do without
                                                                     // capacity
    let mut i = 0;
    let mut j;
    let mut m;
    let mut x = 0;
    let mut y = 0;

    while i < indata.len() && indata[i] != 0xff {
        if indata[i] & 0x80 != 0 {
            j = (indata[i] ^ 0x80) as usize;
            m = 0;
            while m <= j {
                i += 1;
                let pos = (y * width + x) as usize;
                if pos < outdata.len() {
                    outdata[pos] = indata[i];
                }
                y += 1;
                if y >= height {
                    y = 0;
                    x += 1;
                }
                m += 1;
            }
            i += 1;
        } else {
            let count = (indata[i] + 2) as usize;
            for _ in 0..count {
                let pos = (y * width + x) as usize;
                if pos < outdata.len() {
                    outdata[pos] = indata[i + 1];
                }
                y += 1;
                if y >= height {
                    y = 0;
                    x += 1;
                }
            }
            i += 2;
        }
    }

    outdata
    // (y * width + x) as i32
}

