#![no_std]
#![no_main]

mod ili9341;

pub use ili9341::{Ili9341, Orientation, DisplaySize240x320, ILIError};
use fontfile::{PbFont};
// use esp_idf_hal::gpio::{PinDriver, AnyIOPin, Output};
// use esp_idf_hal::delay::{Delay};
// use esp_idf_hal::spi::{config::{DriverConfig, Config}, SPI2, SpiDeviceDriver, SpiDriver};
use display_interface_spi::SPIInterface;
use esp_hal::Blocking;
use esp_hal::spi::master::{Spi, Config, ConfigError};
use esp_hal::gpio::{Output, AnyPin, OutputConfig, Level, Input, InputConfig, Pull};
use esp_hal::delay::Delay;
use esp_hal::peripherals::SPI2;
use esp_hal::time::Rate;
use embedded_hal_bus::spi::ExclusiveDevice;
use dummy_pin::DummyPin;
// use std::error::Error;
use core::error;
use log::info;

// type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct ILIDriver<'a> {
    // pub display: Ili9341<SPIInterface<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, AnyIOPin, Output>>, PinDriver<'a, AnyIOPin, Output>>,
    // pub backlight: PinDriver<'a, AnyIOPin, Output>,
    pub display: Ili9341<SPIInterface<ExclusiveDevice<Spi<'a, Blocking>, DummyPin, Delay>, Output<'a>>, Output<'a>>,
    pub backlight: Output<'a>
}

// const PARALLEL_LINES: usize = 16;

// pub struct Bbox {
//     x_1: u16,
//     x_2: u16,
//     y_1: u16,
//     y_2: u16,
// }

// pub enum IntersectionKind {
//     SplitX,
//     SplitY,
//     ClipWest,
//     ClipEast,
//     ClipNorth,
//     ClipSouth,
// }

// impl Bbox {
//     pub fn intersects(&self, &other: Bbox) -> Option<IntersectionKind> {
        
//     }
// }

impl<'a> ILIDriver<'a> {
    pub fn new(spi2: SPI2<'a>,
               dc: AnyPin<'a>,
               sclk: AnyPin<'a>,
               sdo: AnyPin<'a>,
               sdi: AnyPin<'a>,
               rst: AnyPin<'a>,
               bl: AnyPin<'a>,
        ) -> Result<ILIDriver<'a>, ILIError> {
        let config = OutputConfig::default();
        let inputconfig = InputConfig::default().with_pull(Pull::Up);

        let sclk_driver = Output::new(sclk, Level::Low, config);
        let mosi_driver = Output::new(sdo, Level::Low, config);
        let miso_driver = Input::new(sdi, inputconfig);
        let spi = Spi::new(
            spi2,
            Config::default()
                .with_frequency(Rate::from_mhz(26)),
        )?.with_sck(sclk_driver)
            .with_mosi(mosi_driver)
            .with_miso(miso_driver);  // ConfigError
        let Ok(spidevice) = ExclusiveDevice::new(spi, DummyPin::new_low(), Delay::new());
        let dc_output = Output::new(dc, Level::Low, config);
        let rst_output = Output::new(rst, Level::Low, config);
        // let dc_output = PinDriver::output(dc)?;
        // let rst_output = PinDriver::output(rst)?;
        // let cspin: Option<AnyIOPin> = None;
        // let spi_device_driver = SpiDeviceDriver::new_single(
        //     spi, 
        //     sclk, 
        //     sdo,
        //     Some(sdi),
        //     cspin,
        //     &DriverConfig::default(),
        //     &Config::default(),
        // )?;
        let interface = SPIInterface::new(spidevice, dc_output);
        let display = Ili9341::new(interface, rst_output, &mut Delay::new(), Orientation::Landscape, DisplaySize240x320)?;
        // let mut backlight = PinDriver::output(bl)?;
        let mut backlight = Output::new(bl, Level::Low, config);
        backlight.set_high();
        info!("backlight high");
        Ok(ILIDriver { display, backlight })
    }

    pub fn draw_string(&mut self, x: u16, y: u16, to_draw: &str, font: &mut PbFont, newline_offset: i16) -> Result<(), ILIError> {
        let mut start_x: u16 = x; // x.try_into().map_err(|_| ILIError::WritingOffScreen(x, y))?;
        let mut start_y: u16 = y; // y.try_into().map_err(|_| ILIError::WritingOffScreen(x, y))?;
        for c in to_draw.encode_utf16() {
            info!("loading char {}: {} at {} {}", c, char::from_u32(c as u32).unwrap(), start_x, start_y);
            let (char_metrics, char_slice) = font.load_char(c)?;
            if char_metrics.width != 0 {
                let true_height = char_metrics.height / 3;
                let char_start_x: u16 = (start_x.checked_sub_signed(char_metrics.x)).ok_or(ILIError::WritingOffScreen(start_x, start_y))?;
                let char_start_y: u16 = (start_y.checked_sub_signed(char_metrics.y)).ok_or(ILIError::WritingOffScreen(start_x, start_y))?;
                if usize::from(char_start_x+char_metrics.width-1) > self.display.width() {
                    if newline_offset > 0 {
                        start_y = start_y.checked_sub_signed(newline_offset).ok_or(ILIError::WritingOffScreen(start_x, start_y))?;
                        start_x = x.try_into().map_err(|_| ILIError::WritingOffScreen(char_start_x, char_start_y))?;
                        continue;
                    } else {
                        return Err(ILIError::WritingOffScreen(char_start_x, char_start_y));
                    }
                }
                // info!("draw {}: {} boundaries: {} {} {} {} height: {} y: {}", c, char::from_u32(c as u32).unwrap(), char_start_y, char_start_x, char_start_y+true_height-1, char_start_x+char_metrics.width-1, true_height, char_metrics.y);
                self.display.draw_raw_slice(
                    // char_start_y-true_height+1, 
                    char_start_y, 
                    char_start_x, 
                    char_start_y+true_height-1, 
                    char_start_x+char_metrics.width-1, 
                    char_slice.as_slice())?;
            }
            start_x += char_metrics.advance;
            // start_x += i16::try_from(char_metrics.advance).map_err(|_| ILIError::WritingOffScreen(start_x, start_y))?;
        }
        Ok(())
    }
}
