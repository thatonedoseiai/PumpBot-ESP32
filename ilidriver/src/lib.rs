mod ili9341;

pub use ili9341::{Ili9341, Orientation, DisplaySize240x320, ILIError};
use fontfile::{PbFont};
use esp_idf_hal::gpio::{PinDriver, AnyIOPin, Output};
use esp_idf_hal::delay::{Delay};
use esp_idf_hal::spi::{config::{DriverConfig, Config}, SPI2, SpiDeviceDriver, SpiDriver};
use display_interface_spi::SPIInterface;
use std::error::Error;
use log::info;

// type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct ILIDriver<'a> {
    pub display: Ili9341<SPIInterface<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, AnyIOPin, Output>>, PinDriver<'a, AnyIOPin, Output>>,
    pub backlight: PinDriver<'a, AnyIOPin, Output>,
}

// const PARALLEL_LINES: usize = 16;

impl ILIDriver<'_> {
    pub fn new(spi: SPI2, dc: AnyIOPin, sclk: AnyIOPin, sdo: AnyIOPin, sdi: AnyIOPin, rst: AnyIOPin, bl: AnyIOPin) -> Result<Self, ILIError> {
        let dc_output = PinDriver::output(dc)?;
        let rst_output = PinDriver::output(rst)?;
        let cspin: Option<AnyIOPin> = None;
        let spi_device_driver = SpiDeviceDriver::new_single(
            spi, 
            sclk, 
            sdo,
            Some(sdi),
            cspin,
            &DriverConfig::default(),
            &Config::default(),
        )?;
        let interface = SPIInterface::new(spi_device_driver, dc_output);
        let display = Ili9341::new(interface, rst_output, &mut Delay::new_default(), Orientation::Landscape, DisplaySize240x320)?;
        let mut backlight = PinDriver::output(bl)?;
        backlight.set_high()?;
        Ok(ILIDriver { display, backlight })
    }

    pub fn draw_string(&mut self, x: u16, y: u16, to_draw: &str, font: &mut PbFont, newline_offset: i16) -> Result<(), ILIError> {
        let mut start_x: i16 = x.try_into().map_err(|_| ILIError::WritingOffScreen)?;
        let mut start_y: i16 = y.try_into().map_err(|_| ILIError::WritingOffScreen)?;
        for c in to_draw.encode_utf16() {
            let (char_metrics, char_slice) = font.load_char(c)?;
            if char_metrics.width != 0 {
                let true_height = char_metrics.height / 3;
                let char_start_x: u16 = (start_x+char_metrics.x).try_into().map_err(|_| ILIError::WritingOffScreen)?;
                let char_start_y: u16 = (start_y-char_metrics.y).try_into().map_err(|_| ILIError::WritingOffScreen)?;
                if usize::from(char_start_x+char_metrics.width-1) > self.display.width() {
                    if newline_offset > 0 {
                        start_y -= newline_offset;
                        start_x = x.try_into().map_err(|_| ILIError::WritingOffScreen)?;
                        continue;
                    } else {
                        return Err(ILIError::WritingOffScreen);
                    }
                }
                info!("draw {} boundaries: {} {} {} {} height: {} y: {}", c, char_start_y, char_start_x, char_start_y+true_height-1, char_start_x+char_metrics.width-1, true_height, char_metrics.y);
                self.display.draw_raw_slice(
                    // char_start_y-true_height+1, 
                    char_start_y, 
                    char_start_x, 
                    char_start_y+true_height-1, 
                    char_start_x+char_metrics.width-1, 
                    char_slice.as_slice())?;
            }
            start_x += i16::try_from(char_metrics.advance).map_err(|_| ILIError::WritingOffScreen)?;
        }
        Ok(())
    }
}
