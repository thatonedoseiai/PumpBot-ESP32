mod ili9341;

use ili9341::{Ili9341, Orientation, DisplaySize240x320, ILIError};
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
    fn new(spi: SPI2, dc: AnyIOPin, sclk: AnyIOPin, sdo: AnyIOPin, sdi: AnyIOPin) -> Result<Self> {
        let mut dc_output = PinDriver::output(dc)?;
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
        let mut display = Ili9341::new(interface, dc_output, &mut Delay::new_default(), Orientation::Landscape, DisplaySize240x320)?;
        Ok(ILIDriver { display })
    }

    // fn lcd_cmd(&mut self, cmd: u8) -> Result {
    //     self.display.command(cmd, &[])?;
    //     Ok(())
    // }

    // fn lcd_data(&mut self, data: &[u8]) -> Result {
    //     self.display.send_data(data)?;
    //     Ok(())
    // }

    // fn lcd_get_id(&mut self) -> Result<u32> {
    //     // Implement getting ID if needed.
    //     Ok(0)
    // }

    fn init(&mut self) -> Result {
        // Reset the display
        let rst = PinDriver::output(esp_idf_hal::gpio::Gpio12::new())?;
        rst.set_low()?;
        Delay::new_default().delay_us(100);
        rst.set_high()?;
        Delay::new_default().delay_us(100);

        // Initialization commands
        self.lcd_cmd(0x04)?; // Example command, adjust as needed.
        self.lcd_data(&[0, 0, 1, 0x40, 0, 0])?;
        Ok(())
    }

    fn vertical_scroll(&mut self) -> Result {
        let data_33 = [0, 0, 1, 0x40, 0, 0];
        let mut data_37 = [0; 2];
        data_37[0] = 16 >> 8;
        data_37[1] = 16 & 0xff;

        self.lcd_cmd(0x33)?;
        self.lcd_data(&data_33)?;

        let mad = 0b00101000;
        self.lcd_cmd(0x36)?;
        self.lcd_data(&[mad])?;

        self.lcd_cmd(0x37)?;
        self.lcd_data(&data_37)?;

        let old_mad = 0b00011000;
        self.lcd_cmd(0x36)?;
        self.lcd_data(&[old_mad])?;
        Ok(())
    }

    fn draw_bg(&mut self, bgbuf: &[RGB]) -> Result {
        for ypos in (0..320).step_by(PARALLEL_LINES) {
            self.display.set_window(ypos as u16, 0, (ypos + PARALLEL_LINES - 1) as u16, 239 as u16)?;
            self.display.draw_raw_slice(bgbuf, ypos * 240, (ypos + PARALLEL_LINES) * 240)?;
        }
        Ok(())
    }

    fn draw_sprite(&mut self, sx: u16, y: u16, width: u16, height: u16, bitmap: &[RGB]) -> Result {
        if sx > 320 {
            return Ok(());
        }
        self.display.set_window(y, sx, y + height - 1, sx + width - 1)?;
        self.display.draw_raw_slice(bitmap, 0, width as usize * height as usize)?;
        Ok(())
    }

    fn scroll_screen(&mut self, value: u16) -> Result {
        let data_33 = [0, 0, 1, 0x40, 0, 0];
        let mut data_37 = [0; 2];
        data_37[0] = value >> 8;
        data_37[1] = value & 0xff;

        self.lcd_cmd(0x33)?;
        self.lcd_data(&data_33)?;

        self.lcd_cmd(0x37)?;
        self.lcd_data(&data_37)?;
        Ok(())
    }

    fn send_color(&mut self, color: RGB) -> Result {
        let pixelCount = 400 * PARALLEL_LINES;
        let mut colorbuf = vec![color; pixelCount];
        for ypos in (0..240).step_by(PARALLEL_LINES) {
            self.display.set_window(ypos as u16, 0, (ypos + PARALLEL_LINES - 1) as u16, 319 as u16)?;
            self.display.draw_raw_slice(&colorbuf, 0, pixelCount)?;
        }
        Ok(())
    }

    fn scroll_buffer(&mut self, screenoffset: i32, resetScroll: bool) -> Result {
        if resetScroll {
            self.scroll_screen(0)?;
        }

        let mut i = 0;
        while i < screenoffset {
            // Implement buffer scrolling logic here.
            i += screenoffset - i;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct RGB {
    pixelR: u8,
    pixelG: u8,
    pixelB: u8,
}
