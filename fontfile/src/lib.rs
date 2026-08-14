//! This crate contains the tools to read CBF and CBI file extensions used for PB. For rendering
//! purposes, it also houses the main definition of RGB used throughout the project and by LEDc.
//! Functionalities include:
//! - Decoding horizontal and vertical characters from CBF file [PbFont::load_char]
//! - Decoding the background image data stored in a CBI file [PbBg::load_bgimg]
//!
//! Each CBF file can only contain a font at one particular size. Thus, in order to swap sizes, we
//! must close the current CBF file and open the CBF file with the size we want. Only one CBF file
//! can be open at a time.

#![no_std]
#![no_main]
#![feature(iter_array_chunks)]

extern crate alloc;

// mod rgb;
pub mod pb_font_renderer;

use littlefs2::io;
use littlefs2::io::{Read, SeekFrom};
use littlefs2::fs::Filesystem;
use littlefs2::path;
use littlefs2::path::Path;
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use alloc::{
    vec::Vec, 
    string::{ToString, String},
    vec};
use core::fmt;
use core::error::Error;
use global_settings::{rgb::RGB, rgb};
use flash_storage::PbFlashStorage;
use core::fmt::Debug;
use core::cell::RefCell;
use alloc::sync::Arc;

use profiler::timed;

// use embedded_graphics::{
//     text::{
//         renderer::{TextMetrics, CharacterStyle, TextRenderer},
//         Baseline
//     }, 
//     pixelcolor::{Rgb565, Rgb888},
//     geometry::Point,
//     prelude::DrawTarget,
//     image::{Image, ImageRaw},
// };
// use az::SaturatingAs;

// Font file constants
#[cfg(target_os = "none")]
const FONT_NAME_SIZE_7:  &Path = path!("/NC_7.cbf");
#[cfg(target_os = "none")]
const FONT_NAME_SIZE_12: &Path = path!("/NC_12.cbf");
#[cfg(target_os = "none")]
const FONT_NAME_SIZE_14: &Path = path!("/NC_14.cbf");
#[cfg(target_os = "none")]
const FONT_NAME_SIZE_18: &Path = path!("/NC_18.cbf");
#[cfg(target_os = "none")]
const FONT_NAME_SIZE_24: &Path = path!("/NC_24.cbf");
#[cfg(target_os = "none")]
const FONT_NAME_SIZE_42: &Path = path!("/NC_42.cbf");

#[cfg(not(target_os = "none"))]
const FONT_NAME_SIZE_7:  &Path = path!("./fs/NC_7.cbf");
#[cfg(not(target_os = "none"))]
const FONT_NAME_SIZE_12: &Path = path!("./fs/NC_12.cbf");
#[cfg(not(target_os = "none"))]
const FONT_NAME_SIZE_14: &Path = path!("./fs/NC_14.cbf");
#[cfg(not(target_os = "none"))]
const FONT_NAME_SIZE_18: &Path = path!("./fs/NC_18.cbf");
#[cfg(not(target_os = "none"))]
const FONT_NAME_SIZE_24: &Path = path!("./fs/NC_24.cbf");
#[cfg(not(target_os = "none"))]
const FONT_NAME_SIZE_42: &Path = path!("./fs/NC_42.cbf");

type File<'a, 'b, 'c> = littlefs2::fs::File<'a, 'b, PbFlashStorage<'c>>;
type Mutex<T> = CriticalSectionMutex<T>;

/// Represents an instance of the CBF file decoding engine.
pub struct PbFont {
    // font_file: Option<File<'a>>,
    fs: Arc<Filesystem<'static, PbFlashStorage<'static>>>,
    font_size: Option<FontSize>,
    font_metadata: FontMetadata,
}

/// Represents an instance of the CBI file decoding engine.
pub struct PbBg {
    // bg_file: Option<File<'a>>,
    fs: Arc<Filesystem<'static, PbFlashStorage<'static>>>,
    bg_filename: Option<String>,
    foreground_color: RGB,
    background_color: RGB,
}

/// Represents a font size.
#[derive(Debug, Clone, Copy)]
pub enum FontSize {
    Sz7,
    Sz12,
    Sz14,
    Sz18,
    Sz24,
    Sz42,
}

impl From<FontSize> for u16 {
    fn from(val: FontSize) -> u16 {
        match val {
            FontSize::Sz7 => 7,
            FontSize::Sz12 => 12,
            FontSize::Sz14 => 14,
            FontSize::Sz18 => 18,
            FontSize::Sz24 => 24,
            FontSize::Sz42 => 42,
        }
    }
}

struct CharCache {
    contents: Vec<(u16, u32)>,
    lru: [(u16, u32); 32],
    lru_index: usize,
}

static CHAR_CACHE: Mutex<RefCell<CharCache>> = Mutex::new(RefCell::new(CharCache { contents: Vec::new(), lru: [(0,0); 32], lru_index: 0 }));

impl CharCache {
    fn set_contents(&mut self, n: Vec<(u16, u32)>) {
        self.contents = n;
        self.lru = [(0,0); 32];
    }
}

// Metadata structures
/// Represents the metadata for a CBF font file.
#[derive(Debug, Clone, Copy)]
pub struct FontMetadata {
    pub font_size: u16,
    pub num_glyphs: u32,
}

/// Represents all the stored metadata for a particular character in a CBF font file.
#[derive(Debug, Clone, Copy)]
pub struct CharMetadata {
    pub vertical: bool,
    pub advance: u16,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl From<[u8; 10]> for CharMetadata {
    fn from(val: [u8; 10]) -> CharMetadata {
        let mut advance = u16::from_le_bytes([val[0], val[1]]);
        let vertical = (advance & 0x1000) == 0x1000;
        advance &= 0x4fff;

        CharMetadata {
            vertical,
            advance,
            x: i16::from_le_bytes([val[2], val[3]]), 
            y: i16::from_le_bytes([val[4], val[5]]), 
            width: u16::from_le_bytes([val[6], val[7]]), 
            height: u16::from_le_bytes([val[8], val[9]])
        }
    }
}

// Global color references (these should be initialized elsewhere)
// Correct magic bytes
const CORRECT_MBYTES: [u8; 10] = [0x63, 0x62, 0x66, 0xe5, 0x9c, 0xa7, 0xe5, 0xad, 0x97, 0x02];

/// Represents an IO error that occurs when the board fails to open or read from a file, or
/// something like that.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct IOErrorInfo {
    e: io::Error,
}

impl fmt::Display for IOErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "littlefs2 io error: code {}", self.e.code())
        // self.e.fmt(f)
    }
}

// Error codes
/// All the different kinds of errors that could occur when reading from CBF files.
#[derive(Copy, Clone, Debug)]
pub enum FontFileError {
    FileNotFound,
    BadFormat,
    BadSize,
    BadChar(u16, u16, u16),
    InvalidChar(u16),
    IndexOutOfBounds,
    IOError(IOErrorInfo),
    FileNotOpen,
    BadVersion,
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
            FontFileError::IOError(e) => write!(f, "FF: IOError {}", e),
            FontFileError::FileNotOpen => write!(f, "FF: FileNotOpen"),
            FontFileError::BadVersion => write!(f, "FF: Background file is the wrong version!"),
        }
    }
}
impl Error for FontFileError { }

impl From<io::Error> for FontFileError {
    fn from(val: io::Error) -> FontFileError {
        FontFileError::IOError(IOErrorInfo {
            e: val,
        })
    }
}

impl FontSize {
    fn as_path(&self) -> &Path {
        match self {
            FontSize::Sz7 =>  FONT_NAME_SIZE_7,
            FontSize::Sz12 => FONT_NAME_SIZE_12,
            FontSize::Sz14 => FONT_NAME_SIZE_14,
            FontSize::Sz18 => FONT_NAME_SIZE_18,
            FontSize::Sz24 => FONT_NAME_SIZE_24,
            FontSize::Sz42 => FONT_NAME_SIZE_42,
        }
    }
}

impl PbFont {
    /// Creates a new instance of the CBF file reading engine.
    pub fn new(fs: Arc<Filesystem<'static, PbFlashStorage>>) -> Self {
        PbFont {
            // font_file: None,
            fs: fs,
            font_size: None,
            font_metadata: FontMetadata {
                font_size: 0,
                num_glyphs: 0,
            },
        }
    }

    // Set font size
    /// Opens the file associate with the font size `sz`. Any newly available sizes must be added
    /// to [FontSize] before they can be used, as this function does not accept a number for the
    /// size but instead accepts an entry in [FontSize] to work. This is because attempting to
    /// render a font size which does not have an associated file in the filesystem would result in
    /// an error and/or not display correctly.
    pub fn set_size(&mut self, sz: FontSize) -> Result<(), FontFileError> {

        // Open new font file
        // let mut file = File::open(font_name).map_err(|_| FontFileError::FileNotFound)?;
        if !self.fs.exists(sz.as_path()) {
            return Err(FontFileError::FileNotFound);
        }
        self.font_metadata = self.fs.open_file_with_options_and_then(
            |options| options.read(true).write(true).create(false),
            sz.as_path(),
            |file: &File| {
                // log::info!("opened file!");
                let res = Self::verify_font_file(file);
                if let Err(e) = res {
                    return Ok(Err(e));
                }
                Ok(Self::read_header(file))
            }
        )??;

        // self.font_metadata = Self::read_header(&mut file)?;

        if self.font_metadata.font_size != u16::from(sz) {
            return Err(FontFileError::FileNotFound);
        }

        // self.font_file = Some(file);
        self.font_size = Some(sz);
        self.cache_char_index()?;

        Ok(())
    }

    // Verify font file
    fn verify_font_file(font: &File) -> Result<(), FontFileError> {
        let mut mbytes_buffer = [0u8; 10];
        // info!("READING HEADER");
        font.read_exact(&mut mbytes_buffer)?;
        if mbytes_buffer != CORRECT_MBYTES {
            Err(FontFileError::BadFormat)
        } else {
            Ok(())
        }
    }

    // Read header
    fn read_header(font: &littlefs2::fs::File<'_, '_, PbFlashStorage<'_>>) -> Result<FontMetadata, FontFileError> {
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

    // loads the char index and caches it in RAM.
    fn cache_char_index(&mut self) -> Result<(), FontFileError> {
        if let Some(sz) = self.font_size {
            self.fs.open_file_and_then(
                sz.as_path(),
                |f| {
                    f.seek(SeekFrom::Start(16_u32))?;
                    let mut bytes = vec![0u8; 6 * self.font_metadata.num_glyphs as usize];
                    f.read_exact(&mut bytes)?;
                    let entries: Vec<(u16, u32)> = bytes
                        .chunks_exact(6)
                        .map(|chunk| {
                            let codepoint = u16::from_le_bytes([chunk[0], chunk[1]]);
                            let offset = u32::from_le_bytes([chunk[2], chunk[3], chunk[4], chunk[5]]);
                            (codepoint, offset)
                        })
                        .collect();
                    CHAR_CACHE.lock(|k| {
                        k.borrow_mut().set_contents(entries);
                    });
                    Ok(())
                }
            )?;
        }
        Ok(())
    }

    fn binary_search_cache(&mut self, k: u16) -> Result<u32, FontFileError> {
        let result = CHAR_CACHE.lock(|f| {
            let cache = f.borrow();
            for (entry, output) in &cache.lru {
                if *entry == k {
                    return Ok(*output);
                }
            }
            let c_ref = &cache.contents;
            let res = c_ref.binary_search_by_key(&k, |&(codepoint, _)| codepoint)
                .ok()
                .map(|i| c_ref[i].1)
                .ok_or(FontFileError::InvalidChar(k));
            drop(cache);
            if let Ok(r) = res {
                let mut mut_cache = f.borrow_mut();
                let inx = mut_cache.lru_index;
                mut_cache.lru[inx] = (k, r);
                let len = mut_cache.lru.len();
                mut_cache.lru_index = (inx + 1) % len;
            }
            res
        })?;
        // let result = contents_ref
        //     .binary_search_by_key(&k, |&(codepoint, _)| codepoint)
        //     .ok()
        //     .map(|i| contents_ref[i].1)
        //     .ok_or(FontFileError::InvalidChar(k))?;
        Ok(result)
    }

    // Binary search, returns the offset into the file for the glyph
    // fn binary_search(&mut self, k: u16) -> Result<u32, FontFileError> {
    //     let mut offset: i64 = (self.font_metadata.num_glyphs >> 1).into();
    //     if let Some(sz) = self.font_size {
    //         Ok(self.fs.open_file_and_then(
    //             sz.as_path(),
    //             |f| {
    //                 f.seek(SeekFrom::Start((offset * 6 + 16) as u32))?;
    //                 let mut total_off: u64 = (offset * 6 + 16) as u64;

    //                 let mut buf = [0u8; 2];
    //                 f.read_exact(&mut buf)?;
    //                 let mut prev_entry = u16::from_le_bytes(buf);
    //                 let mut curr_entry = prev_entry;

    //                 while curr_entry != k {
    //                     // if curr_entry > self.max
    //                     if prev_entry < k && curr_entry > k && offset == 1 {
    //                         return Ok(Err(FontFileError::InvalidChar(k)));
    //                     }

    //                     offset >>= 1;
    //                     if offset < 1 {
    //                         offset = 1;
    //                     }

    //                     let offset_amt = if curr_entry > k {
    //                         -offset * 6 - 2
    //                     } else {
    //                         offset * 6 - 2
    //                     };

    //                     f.seek(SeekFrom::Current(offset_amt as i32))?;
    //                     match total_off.checked_add_signed(offset_amt+2) {
    //                         Some(k) => {
    //                             // info!("{} + {} = {}", total_off, offset_amt, k);
    //                             total_off = k;
    //                         },
    //                         None => {
    //                             info!("BAD: {} + {} = OOB", total_off, offset_amt); 
    //                             panic!(); 
    //                         }
    //                     }
    //                     if total_off > (self.font_metadata.num_glyphs * 6 + 16) as u64 {
    //                         return Ok(Err(FontFileError::InvalidChar(k)));
    //                     }
    //                     prev_entry = curr_entry;
                        
    //                     let mut buf = [0u8; 2];
    //                     f.read_exact(&mut buf)?;
    //                     curr_entry = u16::from_le_bytes(buf);
    //                 }
                    
    //                 let mut buf = [0u8; 4];
    //                 f.read_exact(&mut buf)?;
    //                 Ok(Ok(u32::from_le_bytes(buf)))
    //             })??)
    //     } else {
    //         Err(FontFileError::FileNotOpen)
    //     }
    // }

    // Load character
    /// Loads a character with the character code `curchar`, returning either an error or the
    /// character metadata and visual data.
    pub fn load_char(&mut self, curchar: u16) -> Result<(CharMetadata, Vec<RGB>), FontFileError> {
        // let mut font_file = unsafe { FONT_FILE.take() }.unwrap();
        let offset = timed!("binary search", self.binary_search_cache(curchar)?);

        let Some(sz) = self.font_size else { return Err(FontFileError::FileNotOpen); };
        self.fs.open_file_and_then(
            sz.as_path(),
            |font_file| -> Result<Result<(CharMetadata, Vec<RGB>), FontFileError>, littlefs2::io::Error> {
                timed!("seek", font_file.seek(SeekFrom::Start(offset))?);

                let mut buf = [0u8; 10];
                timed!("read metadata", font_file.read_exact(&mut buf)?);
                let cm = timed!("convert metadata", CharMetadata::from(buf));

                // let data_len = (unsafe { CM.width } * unsafe { CM.height } + 2) as usize;
                let data_len: usize = (cm.width * cm.height + 2) as usize;
                let mut data = vec![0u8; data_len];
                timed!("read char data", font_file.read_exact(&mut data)?);

                if curchar == 0x20 {
                    return Ok(Ok((CharMetadata {
                        advance: cm.advance,
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                        vertical: false,
                    }, vec![])));
                }

                if cm.width == 0 || cm.height == 0 {
                    // println!("BAD CHAR: {} has width {} height {}", curchar, unsafe { CM.width }, unsafe { CM.height });
                    return Ok(Err(FontFileError::BadChar(curchar, cm.width, cm.height)));
                }

                let decompressed_len: usize = (cm.width * cm.height) as usize;

                let decompressed = timed!("decode data", if !cm.vertical {
                    decode(&data)
                } else {
                    decode_vert(&data, cm.height, cm.width)
                });
                // info!("DECOMPRESSED LEN: {:?}", decompressed.len());
                // info!("DECOMPRESSED: {:?}", decompressed);

                // Convert to RGB
                // info!("decompressed length: {}", decompressed.len());
                let mut rgb_vec = timed!("create decompressed vec", vec![rgb![0,0,0]; decompressed_len / 3]); // rgb AND transpose as well
                let (mut x, mut y) = (0,0);
                let bufwidth = cm.width as usize;
                let bufheight = (cm.height / 3).into();
                timed!("transpose", {
                for pixel in (&decompressed).iter().array_chunks::<3>() {
                    rgb_vec[y * bufwidth + x] = rgb![*pixel[0], *pixel[1], *pixel[2]];
                    y = y + 1;
                    if y == bufheight {
                        y = 0;
                        x = x + 1;
                    }
                    // rgb_vec.push(rgb![*pixel[0], *pixel[1], *pixel[2]]);
                }
                });
                Ok(Ok((cm, rgb_vec)))
            })?
    }

    pub fn load_char_metadata(&mut self, curchar: u16) -> Result<CharMetadata, FontFileError> {
        // let mut font_file = unsafe { FONT_FILE.take() }.unwrap();
        let offset = self.binary_search_cache(curchar)?;

        let Some(sz) = self.font_size else { return Err(FontFileError::FileNotOpen); };
        self.fs.open_file_and_then(
            sz.as_path(),
            |font_file| {
                font_file.seek(SeekFrom::Start(offset))?;

                let mut buf = [0u8; 10];
                font_file.read_exact(&mut buf)?;
                let cm = CharMetadata::from(buf);

                if curchar == 0x20 {
                    return Ok(Ok(CharMetadata {
                        advance: cm.advance,
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                        vertical: false,
                    }));
                }

                if cm.width == 0 || cm.height == 0 {
                    // println!("BAD CHAR: {} has width {} height {}", curchar, unsafe { CM.width }, unsafe { CM.height });
                    return Ok(Err(FontFileError::BadChar(curchar, cm.width, cm.height)))
                }

                Ok(Ok(cm))
            })?
    }
}

impl PbBg {
    const PB_BG_VERSION: u8 = 2;

    /// Creates an instance of the CBI decoding engine
    pub fn new(fs: Arc<Filesystem<'static, PbFlashStorage<'static>>>) -> Self {
        PbBg {
            // bg_file: None,
            fs: fs,
            bg_filename: None,
            foreground_color: rgb![0],
            background_color: rgb![0],
        }
    }

    // Load background image
    /// Decodes the `index`th background image, reading from the file `name`. If `force_load` is
    /// `True`, the file is forced to be opened anew regardless of whether the previous background image
    /// was from the same file.
    pub fn load_bgimg(&mut self, name: &Path, _force_load: bool, index: i32) -> Result<Vec<RGB>, FontFileError> {
        // let mut image_file = self.bg_file;
        // TODO: figure out force_load

        let res: Result<Result<Vec<RGB>, FontFileError>, littlefs2::io::Error> = self.fs.open_file_and_then(
            name, 
            |file| {
                // Read header
                let mut header = [0u8; 3];
                file.read_exact(&mut header)?;
                if header != [0x63, 0x62, 0x69] { // CBI in ascii
                    return Ok(Err(FontFileError::BadFormat));
                }

                let mut version_byte = [0u8; 1];
                file.read_exact(&mut version_byte)?;
                if version_byte[0] != Self::PB_BG_VERSION {
                    return Ok(Err(FontFileError::BadVersion));
                }

                // Read index
                let mut num_bgs = [0u8; 1];
                file.read_exact(&mut num_bgs)?;
                if index >= num_bgs[0] as i32 {
                    return Ok(Err(FontFileError::IndexOutOfBounds));
                }

                // Seek to image data offset
                file.seek(SeekFrom::Start(5 + 4 * index as u32))?;
                let mut offset_buf = [0u8; 4];
                file.read_exact(&mut offset_buf)?;
                let imagedata_offset = u32::from_le_bytes(offset_buf);

                file.seek(SeekFrom::Start(imagedata_offset))?;

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
                    return Ok(Err(FontFileError::BadSize));
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

                Ok(Ok(buf))
        });
        res.map_err(|_| FontFileError::FileNotFound)?
    }

    // Initialize global colors
    /// Set the foreground and background colours to `foreground` and `background` respectively
    pub fn init_colors(&mut self, foreground: RGB, background: RGB) {
        self.foreground_color = foreground;
        self.background_color = background;
    }
}

// Decode function
/// RLE decoding for the visual image data of both fonts and background images. This function is
/// used for horizontally encoded characters.
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
/// RLE decoding for visual image data of characters and background images. This function is used
/// for characters that have been encoded vertically.
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


#[cfg(test)]
mod test {
    use std::io::{Cursor, Read};
    use crate::CharMetadata;

    #[test]
    fn read_char_metadata() {
        let mut buf = [0u8; 10];
        Cursor::new(b"\xbc\x8a\x34\x12\x78\x56\xad\xde\xef\xbe").read(&mut buf).unwrap();
        let cd = CharMetadata::from(buf);
        assert_eq!(cd.vertical, true);
        assert_eq!(cd.advance, 0xabc);
        assert_eq!(cd.x, 0x1234);
        assert_eq!(cd.y, 0x5678);
        assert_eq!(cd.width, 0xdead);
        assert_eq!(cd.height, 0xbeef);
    }
}
