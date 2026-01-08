mod ili9341;

pub use ili9341::{Ili9341, Orientation, DisplaySize240x320, ILIError, RGB};
use esp_idf_hal::gpio::{PinDriver, AnyIOPin, Output};
use esp_idf_hal::delay::{Delay};
use esp_idf_hal::spi::{config::{DriverConfig, Config}, SPI2, SpiDeviceDriver, SpiDriver};
use display_interface_spi::SPIInterface;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

struct ILIDriver<'a> {
    display: Ili9341<SPIInterface<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, AnyIOPin, Output>>, PinDriver<'a, AnyIOPin, Output>>,
}

const PARALLEL_LINES: usize = 16;

impl ILIDriver<'_> {
    fn new(spi: SPI2, dc: AnyIOPin, sclk: AnyIOPin, sdo: AnyIOPin, sdi: AnyIOPin, rst: AnyIOPin) -> Result<Self> {
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
        Ok(ILIDriver { display })
    }
}
