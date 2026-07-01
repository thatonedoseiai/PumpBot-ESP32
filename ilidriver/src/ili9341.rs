// #![no_std]

//! ILI9341 Display Driver
//!
//! ### Usage
//!
//! To control the display you need to set up:
//!
//! * Interface for communicating with display ([display-interface-spi crate] for SPI)
//! * Configuration (reset pin, delay, orientation and size) for display
//!
//! ```ignore
//! let iface = SPIInterface::new(spi, dc, cs);
//!
//! let mut display = Ili9341::new(
//!     iface,
//!     reset_gpio,
//!     &mut delay,
//!     Orientation::Landscape,
//!     ili9341::DisplaySize240x320,
//! )
//! .unwrap();
//!
//! display.clear(Rgb565::RED).unwrap()
//! ```
//!
//! [display-interface-spi crate]: https://crates.io/crates/display-interface-spi
use embedded_hal::delay::DelayNs;
// use esp_idf_hal::delay::Delay;
// use esp_idf_hal::gpio::OutputPin;
use embedded_hal::digital::OutputPin;

use display_interface::DataFormat;
use display_interface::WriteOnlyDataCommand;
// use std::fmt;
use fontfile::{RGB, ColorConversionError, FontFileError};
// use esp_idf_hal::sys::EspError;
use esp_hal::spi::master::ConfigError;
use esp_hal::delay::Delay;
use log::info;
use core::error::Error;
use core::fmt;

// #[cfg(feature = "graphics")]
// mod graphics_core;

// pub use esp_idf_hal::spi::config::MODE_0 as SPI_MODE;

pub use display_interface::DisplayError;

#[derive(Debug, Clone)]
pub enum ILIError {
    Disp(DisplayError),
    Conv(ColorConversionError),
    // Esp(EspError),
    FF(FontFileError),
    WritingOffScreen(u16, u16),
    SpiConfigError(ConfigError),
}

impl Error for ILIError { }
impl fmt::Display for ILIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ILIError::Disp(d) => match d {
                DisplayError::InvalidFormatError => write!(f, "ILIERROR: Invalid Formatting Error"),
                DisplayError::BusWriteError => write!(f, "ILIERROR: Bus Write Error"),
                DisplayError::DCError => write!(f, "ILIERROR: DC Error"),
                DisplayError::CSError => write!(f, "ILIERROR: CS Error"),
                DisplayError::DataFormatNotImplemented => write!(f, "ILIERROR: Data Format not implemented!"),
                DisplayError::RSError => write!(f, "ILIERROR: RS Error"),
                DisplayError::OutOfBoundsError => write!(f, "ILIERROR: Out Of Bounds Error"),
                _ => write!(f, "ILIERROR: unhandled error type!"),
            },
            ILIError::Conv(c) => match c {
                ColorConversionError::NonHomogeneousIterator => write!(f, "ILIERROR: color iterator has mixed color formats!"),
                ColorConversionError::TooMuchColorData => write!(f, "ILIERROR: too much color data sent to write function!"),
            },
            ILIError::SpiConfigError(c) => write!(f, "Spi Configuration error! {}", c),
            ILIError::FF(ff) => write!(f, "{}", ff),
            ILIError::WritingOffScreen(x, y) => write!(f, "Error: writing off screen at ({}, {})!", x, y),
        }
    }
}

impl From<DisplayError> for ILIError {
    fn from(val: DisplayError) -> ILIError {
        ILIError::Disp(val)
    }
}

impl From<ColorConversionError> for ILIError {
    fn from(val: ColorConversionError) -> ILIError {
        ILIError::Conv(val)
    }
}

impl From<ConfigError> for ILIError {
    fn from(val: ConfigError) -> ILIError {
        ILIError::SpiConfigError(val)
    }
}

// impl From<EspError> for ILIError {
//     fn from(val: EspError) -> ILIError {
//         ILIError::Esp(val)
//     }
// }

impl From<FontFileError> for ILIError {
    fn from(val: FontFileError) -> ILIError {
        ILIError::FF(val)
    }
}

type Result<T = (), E = ILIError> = core::result::Result<T, E>;

/// Trait that defines display size information
pub trait DisplaySize {
    /// Width in pixels
    const WIDTH: usize;
    /// Height in pixels
    const HEIGHT: usize;
}

/// Generic display size of 240x320 pixels
pub struct DisplaySize240x320;

impl DisplaySize for DisplaySize240x320 {
    const WIDTH: usize = 240;
    const HEIGHT: usize = 320;
}

/// Generic display size of 320x480 pixels
pub struct DisplaySize320x480;

impl DisplaySize for DisplaySize320x480 {
    const WIDTH: usize = 320;
    const HEIGHT: usize = 480;
}

/// For quite a few boards (ESP32-S2-Kaluga-1, M5Stack, M5Core2 and others),
/// the ILI9341 initialization command arguments are slightly different
///
/// This trait provides the flexibility for users to define their own
/// initialization command arguments suitable for the particular board they are using
pub trait PictureMode {
    fn mode(&self) -> u8;

    fn is_landscape(&self) -> bool;
}

/// The default implementation of the PictureMode trait from above
/// Should work for most (but not all) boards
pub enum Orientation {
    Portrait,
    PortraitFlipped,
    Landscape,
    LandscapeFlipped,
}

impl PictureMode for Orientation {
    fn mode(&self) -> u8 {
        match self {
            Self::Portrait => 0x40 | 0x08,
            Self::Landscape => 0x20 | 0x08,
            Self::PortraitFlipped => 0x80 | 0x08,
            Self::LandscapeFlipped => 0x40 | 0x80 | 0x20 | 0x08,
        }
    }

    fn is_landscape(&self) -> bool {
        match self {
            Self::Landscape | Self::LandscapeFlipped => true,
            Self::Portrait | Self::PortraitFlipped => false,
        }
    }
}

/// Specify state of specific mode of operation
pub enum ModeState {
    On,
    Off,
}

/// There are two method for drawing to the screen:
/// [Ili9341::draw_raw_iter] and [Ili9341::draw_raw_slice]
///
/// In both cases the expected pixel format is rgb565.
///
/// The hardware makes it efficient to draw rectangles on the screen.
///
/// What happens is the following:
///
/// - A drawing window is prepared (with the 2 opposite corner coordinates)
/// - The starting point for drawint is the top left corner of this window
/// - Every pair of bytes received is intepreted as a pixel value in rgb565
/// - As soon as a pixel is received, an internal counter is incremented,
///   and the next word will fill the next pixel (the adjacent on the right, or
///   the first of the next row if the row ended)
pub struct Ili9341<IFACE, RESET> {
    interface: IFACE,
    reset: RESET,
    width: usize,
    height: usize,
    landscape: bool,
}

const INIT_COMMANDS: [(Command, &[u8]); 23] = [
    (Command::PowerControlB, &[0x00, 0x83, 0x30]),
    (Command::PowerOnSequenceControl, &[0x64, 0x03, 0x12, 0x81]),
    (Command::DriverTimingControlA, &[0x85, 0x01, 0x79]),
    (Command::PowerControlA, &[0x39, 0x2c, 0x00, 0x34, 0x02]),
    (Command::PumpRatioControl, &[0x20]),
    (Command::DriverTimingControlB, &[0x00, 0x00]),
    (Command::PowerControl1, &[0x26]),
    (Command::PowerControl2, &[0x11]),
    (Command::VComControl1, &[0x35, 0x3e]),
    (Command::VComControl2, &[0xbe]),
    (Command::MemoryAccessControl, &[0x18]),
    (Command::PixelFormatSet, &[0x66]),
    (Command::NormalModeFrameRate, &[0x00, 0x10]),
    (Command::Enable3G, &[0x08]),
    (Command::GammaSet, &[0x01]),
    (Command::VerticalScrollDefine, &[0x00, 0x00, 0x01, 0x40, 0x00, 0x00]),
    (Command::PositiveGammaCorrection, &[0x1f, 0x1a, 0x18, 0x0a, 0x0f, 0x06, 0x45, 0x87, 0x32, 0x0a, 0x07, 0x02, 0x07, 0x05, 0x00]),
    (Command::NegativeGammaCorrection, &[0x00, 0x25, 0x27, 0x05, 0x10, 0x09, 0x3a, 0x78, 0x4d, 0x05, 0x18, 0x0d, 0x38, 0x3a, 0x1f]),
    (Command::ColumnAddressSet, &[0x00, 0x00, 0x00, 0xef]),
    (Command::PageAddressSet, &[0x00, 0x00, 0x01, 0x3f]),
    (Command::MemoryWrite, &[0x00]),
    (Command::EntryModeSet, &[0x07]),
    (Command::DisplayFunctionControl, &[0x0a, 0x82, 0x27, 0x00]),
];
impl<IFACE, RESET> Ili9341<IFACE, RESET>
where
    IFACE: WriteOnlyDataCommand,
    RESET: OutputPin,
{
    pub fn new<SIZE, MODE>(
        interface: IFACE,
        reset: RESET,
        delay: &mut Delay,
        mode: MODE,
        _display_size: SIZE,
    ) -> Result<Self>
    where
        SIZE: DisplaySize,
        MODE: PictureMode,
    {
        let mut ili9341 = Ili9341 {
            interface,
            reset,
            width: SIZE::WIDTH,
            height: SIZE::HEIGHT,
            landscape: false,
        };

        // Do hardware reset by holding reset low for at least 10us
        ili9341.reset.set_low().map_err(|_| ILIError::Disp(DisplayError::RSError))?;
        let _ = delay.delay_ms(1);
        // Set high for normal operation
        ili9341
            .reset
            .set_high()
            .map_err(|_| ILIError::Disp(DisplayError::RSError))?;

        // Wait 5ms after reset before sending commands
        // and 120ms before sending Sleep Out
        let _ = delay.delay_ms(5);

        // Do software reset
        ili9341.command(Command::SoftwareReset, &[])?;

        // Wait 5ms after reset before sending commands
        // and 120ms before sending Sleep Out
        let _ = delay.delay_ms(120);

        ili9341.set_orientation(mode)?;

        // Set pixel format to 16 bits per pixel
        ili9341.set_colormode()?;
        // ili9341.command(Command::PixelFormatSet, &[0x55])?;

        for &(cmd, args) in INIT_COMMANDS.iter() {
            ili9341.command(cmd, args)?;
        }

        info!("SENT INIT ARGS THROUGH SPI!");

        ili9341.sleep_mode(ModeState::Off)?;

        // Wait 5ms after Sleep Out before sending commands
        let _ = delay.delay_ms(5);

        ili9341.display_mode(ModeState::On)?;

        Ok(ili9341)
    }
}

impl<IFACE, RESET> Ili9341<IFACE, RESET>
where
    IFACE: WriteOnlyDataCommand,
{
    fn command(&mut self, cmd: Command, args: &[u8]) -> Result {
        self.interface.send_commands(DataFormat::U8(&[cmd as u8]))?;
        Ok(self.interface.send_data(DataFormat::U8(args))?)
    }

    fn write_iter<I: IntoIterator<Item = RGB>>(&mut self, data: I) -> Result {
        self.command(Command::MemoryWrite, &[])?;
        use DataFormat::U8Iter;
        Ok(self.interface.send_data(U8Iter(&mut data.into_iter().flat_map(|x| {
            <RGB as Into<[u8;3]>>::into(x)
        })))?)
    }

    fn write_slice(&mut self, data: &[RGB]) -> Result {
        self.command(Command::MemoryWrite, &[])?;
        // let len = data.len().checked_mul(3).ok_or(ILIError::Conv(ColorConversionError::TooMuchColorData))?;
        // let ptr = data.as_ptr().cast();
        // let new: &[u8] = unsafe {
        //     from_raw_parts(ptr, len)
        // };
        Ok(self.interface.send_data(DataFormat::U8(RGB::to_byte_array(data)?))?)
    }

    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result {
        self.command(
            Command::ColumnAddressSet,
            &[
                (x0 >> 8) as u8,
                (x0 & 0xff) as u8,
                (x1 >> 8) as u8,
                (x1 & 0xff) as u8,
            ],
        )?;
        self.command(
            Command::PageAddressSet,
            &[
                (y0 >> 8) as u8,
                (y0 & 0xff) as u8,
                (y1 >> 8) as u8,
                (y1 & 0xff) as u8,
            ],
        )
    }

    /// Configures the screen for hardware-accelerated vertical scrolling.
    pub fn configure_vertical_scroll(
        &mut self,
        fixed_top_lines: u16,
        fixed_bottom_lines: u16,
    ) -> Result<Scroller> {
        let height = if self.landscape {
            self.width
        } else {
            self.height
        } as u16;
        let scroll_lines = height as u16 - fixed_top_lines - fixed_bottom_lines;

        self.command(
            Command::VerticalScrollDefine,
            &[
                (fixed_top_lines >> 8) as u8,
                (fixed_top_lines & 0xff) as u8,
                (scroll_lines >> 8) as u8,
                (scroll_lines & 0xff) as u8,
                (fixed_bottom_lines >> 8) as u8,
                (fixed_bottom_lines & 0xff) as u8,
            ],
        )?;

        Ok(Scroller::new(fixed_top_lines, fixed_bottom_lines, height))
    }

    pub fn scroll_vertically(&mut self, scroller: &mut Scroller, num_lines: u16) -> Result {
        scroller.top_offset += num_lines;
        if scroller.top_offset > (scroller.height - scroller.fixed_bottom_lines) {
            scroller.top_offset = scroller.fixed_top_lines
                + (scroller.top_offset + scroller.fixed_bottom_lines - scroller.height)
        }

        self.command(
            Command::VerticalScrollAddr,
            &[
                (scroller.top_offset >> 8) as u8,
                (scroller.top_offset & 0xff) as u8,
            ],
        )
    }

    /// Draw a rectangle on the screen, represented by top-left corner (x0, y0)
    /// and bottom-right corner (x1, y1).
    ///
    /// The border is included.
    ///
    /// This method accepts an iterator of rgb565 pixel values.
    ///
    /// The iterator is useful to avoid wasting memory by holding a buffer for
    /// the whole screen when it is not necessary.
    pub fn draw_raw_iter<I: IntoIterator<Item = RGB>>(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        data: I,
    ) -> Result {
        self.set_window(x0, y0, x1, y1)?;
        self.write_iter(data)
    }

    /// Draw a rectangle on the screen, represented by top-left corner (x0, y0)
    /// and bottom-right corner (x1, y1).
    ///
    /// The border is included.
    ///
    /// This method accepts a raw buffer of words that will be copied to the screen
    /// video memory.
    ///
    /// The expected format is rgb565.
    pub fn draw_raw_slice(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, data: &[RGB]) -> Result {
        self.set_window(x0, y0, x1, y1)?;
        self.write_slice(data)
    }

    pub fn set_colormode(&mut self) -> Result {
        self.command(Command::PixelFormatSet, &[0x66])
    }

    /// Change the orientation of the screen
    pub fn set_orientation<MODE>(&mut self, mode: MODE) -> Result
    where
        MODE: PictureMode,
    {
        self.command(Command::MemoryAccessControl, &[mode.mode()])?;

        if self.landscape ^ mode.is_landscape() {
            core::mem::swap(&mut self.height, &mut self.width);
        }
        self.landscape = mode.is_landscape();
        Ok(())
    }

    /// Fill entire screen with specfied color u16 value
    pub fn clear_screen(&mut self, color: RGB) -> Result {
        let color = core::iter::repeat(color).take(self.width * self.height);
        self.draw_raw_iter(0, 0, self.width as u16, self.height as u16, color)
    }

    /// Control the screen sleep mode:
    pub fn sleep_mode(&mut self, mode: ModeState) -> Result {
        match mode {
            ModeState::On => self.command(Command::SleepModeOn, &[]),
            ModeState::Off => self.command(Command::SleepModeOff, &[]),
        }
    }

    /// Control the screen display mode
    pub fn display_mode(&mut self, mode: ModeState) -> Result {
        match mode {
            ModeState::On => self.command(Command::DisplayOn, &[]),
            ModeState::Off => self.command(Command::DisplayOff, &[]),
        }
    }

    /// Invert the pixel color on screen
    pub fn invert_mode(&mut self, mode: ModeState) -> Result {
        match mode {
            ModeState::On => self.command(Command::InvertOn, &[]),
            ModeState::Off => self.command(Command::InvertOff, &[]),
        }
    }

    /// Idle mode reduces the number of colors to 8
    pub fn idle_mode(&mut self, mode: ModeState) -> Result {
        match mode {
            ModeState::On => self.command(Command::IdleModeOn, &[]),
            ModeState::Off => self.command(Command::IdleModeOff, &[]),
        }
    }

    /// Set display brightness to the value between 0 and 255
    pub fn brightness(&mut self, brightness: u8) -> Result {
        self.command(Command::SetBrightness, &[brightness])
    }

    /// Set adaptive brightness value equal to [AdaptiveBrightness]
    pub fn content_adaptive_brightness(&mut self, value: AdaptiveBrightness) -> Result {
        self.command(Command::ContentAdaptiveBrightness, &[value as _])
    }

    /// Configure [FrameRateClockDivision] and [FrameRate] in normal mode
    pub fn normal_mode_frame_rate(
        &mut self,
        clk_div: FrameRateClockDivision,
        frame_rate: FrameRate,
    ) -> Result {
        self.command(
            Command::NormalModeFrameRate,
            &[clk_div as _, frame_rate as _],
        )
    }

    /// Configure [FrameRateClockDivision] and [FrameRate] in idle mode
    pub fn idle_mode_frame_rate(
        &mut self,
        clk_div: FrameRateClockDivision,
        frame_rate: FrameRate,
    ) -> Result {
        self.command(Command::IdleModeFrameRate, &[clk_div as _, frame_rate as _])
    }
}

impl<IFACE, RESET> Ili9341<IFACE, RESET> {
    /// Get the current screen width. It can change based on the current orientation
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the current screen heighth. It can change based on the current orientation
    pub fn height(&self) -> usize {
        self.height
    }
}

/// Scroller must be provided in order to scroll the screen. It can only be obtained
/// by configuring the screen for scrolling.
pub struct Scroller {
    top_offset: u16,
    fixed_bottom_lines: u16,
    fixed_top_lines: u16,
    height: u16,
}

impl Scroller {
    fn new(fixed_top_lines: u16, fixed_bottom_lines: u16, height: u16) -> Scroller {
        Scroller {
            top_offset: fixed_top_lines,
            fixed_top_lines,
            fixed_bottom_lines,
            height,
        }
    }
}

/// Available Adaptive Brightness values
pub enum AdaptiveBrightness {
    Off = 0x00,
    UserInterfaceImage = 0x01,
    StillPicture = 0x02,
    MovingImage = 0x03,
}

/// Available frame rate in Hz
pub enum FrameRate {
    FrameRate119 = 0x10,
    FrameRate112 = 0x11,
    FrameRate106 = 0x12,
    FrameRate100 = 0x13,
    FrameRate95 = 0x14,
    FrameRate90 = 0x15,
    FrameRate86 = 0x16,
    FrameRate83 = 0x17,
    FrameRate79 = 0x18,
    FrameRate76 = 0x19,
    FrameRate73 = 0x1a,
    FrameRate70 = 0x1b,
    FrameRate68 = 0x1c,
    FrameRate65 = 0x1d,
    FrameRate63 = 0x1e,
    FrameRate61 = 0x1f,
}

/// Frame rate clock division
pub enum FrameRateClockDivision {
    Fosc = 0x00,
    FoscDiv2 = 0x01,
    FoscDiv4 = 0x02,
    FoscDiv8 = 0x03,
}

#[derive(Clone, Copy)]
enum Command {
    SoftwareReset = 0x01,
    MemoryAccessControl = 0x36,
    PixelFormatSet = 0x3a,
    SleepModeOn = 0x10,
    SleepModeOff = 0x11,
    InvertOff = 0x20,
    InvertOn = 0x21,
    GammaSet = 0x26,
    DisplayOff = 0x28,
    DisplayOn = 0x29,
    ColumnAddressSet = 0x2a,
    PageAddressSet = 0x2b,
    MemoryWrite = 0x2c,
    VerticalScrollDefine = 0x33,
    VerticalScrollAddr = 0x37,
    IdleModeOff = 0x38,
    IdleModeOn = 0x39,
    SetBrightness = 0x51,
    ContentAdaptiveBrightness = 0x55,
    NormalModeFrameRate = 0xb1,
    IdleModeFrameRate = 0xb2,
    DisplayFunctionControl = 0xb6,
    EntryModeSet = 0xb7,
    PowerControl1 = 0xc0,
    PowerControl2 = 0xc1,
    VComControl1 = 0xc5,
    VComControl2 = 0xc7,
    PowerControlA = 0xcb,
    PowerControlB = 0xcf,
    PositiveGammaCorrection = 0xe0,
    NegativeGammaCorrection = 0xe1,
    DriverTimingControlA = 0xe8,
    DriverTimingControlB = 0xea,
    PowerOnSequenceControl = 0xed,
    Enable3G = 0xf2,
    PumpRatioControl = 0xf7,
}
