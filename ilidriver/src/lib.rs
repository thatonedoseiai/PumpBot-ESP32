#![no_main]                // you can drop this if you use a normal `main`
use core::mem;
use std::slice::from_raw_parts;
use esp_idf_hal::{ 
    delay::FreeRtos,
    gpio::{/*GpioPin, */Input, Output, PinDriver, AnyIOPin, /*Floating*/},
    peripherals::Peripherals,
    spi::{ 
        Spi, SPI2, SpiBusDriver, SpiDeviceDriver, SpiConfig, SpiDriverConfig
    },
    sys::EspError,
};
//use esp_idf_hal::sys::printf; //for debug message only, doesn't compile 
use std::{
    sync::{Arc, Mutex},
    thread,
};

// --------------------------------------------------------------------- 
//  RGB colour, 24‑bit (R,G,B) – 3 bytes, no padding
// ---------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct RGB24 {
    pixelR: u8,
    pixelG: u8,
    pixelB: u8,
}

// ---------------------------------------------------------------------
//  Init‑command entry – data is a slice (≤ 16 bytes)
// ---------------------------------------------------------------------
#[derive(Copy, Clone, Debug)]
pub struct LcdInitCmd<'a> {
    pub cmd: u8,
    pub data: &'a [u8],
    pub databytes: u8,     // bit 7 = delay‑after‑cmd, 0xFF = end marker
}

// ---------------------------------------------------------------------
//  Constants – screen resolution (320×240) and data‑driven parallelism 
// ---------------------------------------------------------------------
const SCREEN_WIDTH: usize  = 320;   // X‑dimension (horizontal)
const SCREEN_HEIGHT: usize = 240;   // Y‑dimension (vertical)
const BUFFER_STRIDE: usize = SCREEN_HEIGHT;   // column‑major buffer
const PARALLEL_LINES: usize = 16;   // how many rows are sent in one burst

// ---------------------------------------------------------------------
//  Init command table – identical to the C array in ILIDriver.h        
// ---------------------------------------------------------------------
static ILI9341_INIT_CMDS: &[LcdInitCmd] = &[
    LcdInitCmd { cmd: 0xCF, data: &[0x00, 0x83, 0x30], databytes: 3 },
    LcdInitCmd { cmd: 0xED, data: &[0x64, 0x03, 0x12, 0x81], databytes: 4 },
    LcdInitCmd { cmd: 0xE8, data: &[0x85, 0x01, 0x79], databytes: 3 },
    LcdInitCmd { cmd: 0xCB, data: &[0x39, 0x2C, 0x00, 0x34, 0x02], databytes: 5 },
    LcdInitCmd { cmd: 0xF7, data: &[0x20], databytes: 1 },
    LcdInitCmd { cmd: 0xEA, data: &[0x00, 0x00], databytes: 2 },
    LcdInitCmd { cmd: 0xC0, data: &[0x26], databytes: 1 },
    LcdInitCmd { cmd: 0xC1, data: &[0x11], databytes: 1 },
    LcdInitCmd { cmd: 0xC5, data: &[0x35, 0x3E], databytes: 2 },
    LcdInitCmd { cmd: 0xC7, data: &[0xBE], databytes: 1 },
    LcdInitCmd { cmd: 0x36, data: &[0x18], databytes: 1 },
    LcdInitCmd { cmd: 0x3A, data: &[0x66], databytes: 1 },
    LcdInitCmd { cmd: 0xB1, data: &[0x00, 0x10], databytes: 2 },
    LcdInitCmd { cmd: 0xF2, data: &[0x08], databytes: 1 },
    LcdInitCmd { cmd: 0x26, data: &[0x01], databytes: 1 },
    LcdInitCmd { cmd: 0x33, data: &[0x00, 0x00, 0x01, 0x40, 0x00, 0x00], databytes: 6 },
    LcdInitCmd { cmd: 0xE0, data: &[0x1F, 0x1A, 0x18, 0x0A, 0x0F, 0x06, 0x45, 0x87, 0x32, 0x0A, 0x07, 0x02, 0x07, 0x05, 0x00], databytes: 15 },
    LcdInitCmd { cmd: 0xE1, data: &[0x00, 0x25, 0x27, 0x05, 0x10, 0x09, 0x3A, 0x78, 0x4D, 0x05, 0x18, 0x0D, 0x38, 0x3A, 0x1F], databytes: 15 },
    LcdInitCmd { cmd: 0x2A, data: &[0x00, 0x00, 0x00, 0xEF], databytes: 4 },
    LcdInitCmd { cmd: 0x2B, data: &[0x00, 0x00, 0x01, 0x3F], databytes: 4 },
    LcdInitCmd { cmd: 0x2C, data: &[], databytes: 0 },
    LcdInitCmd { cmd: 0xB7, data: &[0x07], databytes: 1 },
    LcdInitCmd { cmd: 0xB6, data: &[0x0A, 0x82, 0x27, 0x00], databytes: 4 },
    LcdInitCmd { cmd: 0x11, data: &[], databytes: 0x80 },
    LcdInitCmd { cmd: 0x29, data: &[], databytes: 0x80 },
    LcdInitCmd { cmd: 0x00, data: &[], databytes: 0xFF },  // end marker
];

// ---------------------------------------------------------------------
//  Core driver – owns the SPI device, DC / RST pins and a framebuffer  
// ---------------------------------------------------------------------
type SpiBus  = SpiBusDriver<'static, SPI2>;
type SpiDev  = SpiDeviceDriver<'static, SpiBus>;

pub struct Lcd<'a> {
    spi: SpiDev,
    dc:  PinDriver<'a,AnyIOPin, Output>, // pin 11
    rst: PinDriver<'a,AnyIOPin, Output>, //pin 12
    /// The frame buffer is optional – created on demand.
    framebuf: Option<Vec<RGB24>>,
}

impl Lcd<'_> {
    /// Create a new driver instance (returns an Arc‑wrapped Mutex so it can
    /// be shared between threads).
    pub fn new() -> Result<Arc<Mutex<Self>>, Box<dyn core::error::Error>> {
        let peripherals = Peripherals::take();
            //.ok_or("Failed to take peripherals")?; //completely hallucinated lol

        // ---------- SPI pins -------------------------------------------------
        let miso = peripherals.pins.gpio10.into_floating_input();
        let mosi = peripherals.pins.gpio46.into_push_pull_output();
        let sck  = peripherals.pins.gpio9.into_push_pull_output();

        // ---------- CS pin ---------------------------------------------------
        // In the original C code CS is unused – we keep it for completeness.
        let cs = peripherals.pins.gpio38.into_push_pull_output();

        // ---------- SPI bus ---------------------------------------------------
        let spi_bus = Spi::new(
            peripherals.spi2,
            (miso, mosi, sck),
            &SpiConfig::default(),
        )?;

        // ---------- SPI device ------------------------------------------------
        let spi_dev = spi_bus
            .add_device(
                Some(cs),
                &SpiDriverConfig::default(),
            )?;

        // ---------- DC / RST pins -------------------------------------------
        let dc  = peripherals.pins.gpio11.into_push_pull_output();
        let rst = peripherals.pins.gpio12.into_push_pull_output();

        let lcd = Lcd {
            spi: spi_dev,
            dc,
            rst,
            framebuf: None,
        };

        Ok(Arc::new(Mutex::new(lcd)))
    }

    // ---------------------------------------------------------------------
    //  Low‑level write helpers – these take a mutable reference to self
    // ---------------------------------------------------------------------
    fn write_cmd(&mut self, cmd: u8) -> Result<(), EspError> {
        self.dc.set_low()?; //PinDriver function
        self.spi.write(&[cmd])?;
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), EspError> {
        if data.is_empty() { return Ok(()); }
        self.dc.set_high()?; //PinDriver function
        self.spi.write(data)?;
        Ok(())
    }

    /// Send a slice of RGB24 values as raw bytes.
    fn write_rgb24_slice(&mut self, data: &[RGB24]) -> Result<(), EspError> {
        if data.is_empty() { return Ok(()); }
        let bytes: &[u8] = unsafe {
            from_raw_parts(data.as_ptr() as *const u8, data.len() * mem::size_of::<RGB24>())
        };
        self.write_data(bytes)
    }

    // --------------------------------------------------------------------- 
    //  High‑level API – called from the outer wrapper after locking        
    // --------------------------------------------------------------------- 

    /// Send the complete initialisation sequence.
    pub fn init(&mut self) -> Result<(), EspError> {

        // 1. Reset the display 
        self.rst.set_low()?;
        FreeRtos::delay_ms(100);
        self.rst.set_high()?;
        FreeRtos::delay_ms(100);

        //printf("LCD ILI9341 initialised!");

        // 2. Send all init commands 
        for cmd in ILI9341_INIT_CMDS {
            if cmd.databytes == 0xFF { break; }
            self.write_cmd(cmd.cmd)?;
            if !cmd.data.is_empty() { self.write_data(cmd.data)?; }
            if cmd.databytes & 0x80 != 0 { FreeRtos::delay_ms(100); }
        }
        Ok(())
    }

    /// Read the 24‑bit device ID (0x04 command).
    pub fn get_id(&mut self) -> Result<u32, EspError> {
        self.write_cmd(0x04)?;
        // D/C high for data
        self.dc.set_high()?;

        // Dummy TX buffer – the LCD will echo back 3 bytes.
        let tx = [0u8; 3];
        let mut rx = [0u8; 3];
        self.spi.transfer(&tx, &mut rx)?;

        Ok((rx[0] as u32) | ((rx[1] as u32) << 8) | ((rx[2] as u32) << 16))
    }

    /// Fill the internal frame buffer with one colour.
    pub fn buffer_fillcolor(&mut self, col: RGB24) {
        if self.framebuf.is_none() {
            self.framebuf = Some(vec![RGB24::default(); SCREEN_WIDTH * SCREEN_HEIGHT]);
        }
        if let Some(ref mut buf) = self.framebuf {
            for p in buf.iter_mut() { *p = col; }
        }
    }

    /// Copy a sprite into the frame buffer at (x, y).
    /// The buffer layout is column‑major:  index = (x+dx)*240 + (y+dy).
    pub fn buffer_sprite(&mut self,
                         x: usize, y: usize,
                         width: usize, height: usize,
                         bitmap: &[RGB24]) {
        if self.framebuf.is_none() {
            self.framebuf = Some(vec![RGB24::default(); SCREEN_WIDTH * SCREEN_HEIGHT]);
        }
        let buf = self.framebuf.as_mut().unwrap();
        for i in 0..width {
            for j in 0..height {
                let dest = (x + i) * BUFFER_STRIDE + (y + j);
                if dest < buf.len() {
                    buf[dest] = bitmap[i * height + j];
                }
            }
        }
    }

    /// Draw a sprite directly to the display.
    pub fn draw_sprite(&mut self,
                       sx: usize, y: usize,
                       width: usize, height: usize,
                       bitmap: &[RGB24]) -> Result<(), EspError> {
        if sx + width > SCREEN_WIDTH { return Ok(()); }

        // Column address set – note that X ↔ Y are swapped due to the
        // display’s MADCTL (0x36) configuration.
        self.write_cmd(0x2A)?;
        let col = [
            (y >> 8) as u8,
            (y & 0xFF) as u8,
            ((y + height - 1) >> 8) as u8,
            ((y + height - 1) & 0xFF) as u8,
        ];
        self.write_data(&col)?;

        // Page address set – X coordinate 
        self.write_cmd(0x2B)?;
        let page = [
            (sx >> 8) as u8,
            (sx & 0xFF) as u8,
            ((sx + width - 1) >> 8) as u8,
            ((sx + width - 1) & 0xFF) as u8,
        ];
        self.write_data(&page)?;

        // Memory write 
        self.write_cmd(0x2C)?;
        self.write_rgb24_slice(bitmap)?;

        Ok(())
    }

    /// Send a block of columns (num_cols) from the frame buffer.
    pub fn send_lines(&mut self,
                      start_col: usize,
                      bgbuf: &[RGB24],
                      num_cols: usize) -> Result<(), EspError> {
        let cols = num_cols.min(PARALLEL_LINES);
        if cols == 0 { return Ok(()); }

        // 1. Column address set – X dimension 
        self.write_cmd(0x2A)?;
        let col = [
            (start_col >> 8) as u8,
            (start_col & 0xFF) as u8,
            ((start_col + cols - 1) >> 8) as u8,
            ((start_col + cols - 1) & 0xFF) as u8,
        ];
        self.write_data(&col)?;

        // 2. Page address set – full height 
        self.write_cmd(0x2B)?;
        let page = [
            0x00,
            0x00,
            (SCREEN_HEIGHT >> 8) as u8,
            (SCREEN_HEIGHT & 0xFF) as u8,
        ];
        self.write_data(&page)?;

        // 3. Memory write 
        self.write_cmd(0x2C)?;

        // 4. Pixel data – slice of the buffer 
        let start = start_col * BUFFER_STRIDE;
        let end   = start + cols * SCREEN_HEIGHT;
        self.write_rgb24_slice(&bgbuf[start..end])?;

        Ok(())
    }

    /// Scroll the display by a pixel offset (used by vertical scrolling).
    pub fn scroll_screen(&mut self, value: u16) -> Result<(), EspError> {
        // 0x33 – vertical scrolling area definition
        self.write_cmd(0x33)?;
        self.write_data(&[0x00, 0x00, 0x01, 0x40, 0x00, 0x00])?;

        // 0x37 – start address of the scroll area
        self.write_cmd(0x37)?;
        self.write_data(&[(value >> 8) as u8, (value & 0xFF) as u8])?;

        Ok(())
    }

    /// Draw a background image – the caller must supply a buffer that
    /// matches the internal column‑major layout.
    pub fn draw_bg(&self, bgbuf: Vec<RGB24>) {
        let lcd = self.clone();   // Arc<Mutex<…>>
        thread::spawn(move || {
            let mut lcd_guard = lcd.lock().unwrap();
            let mut col = 0;
            while col < SCREEN_WIDTH {
                let cols = std::cmp::min(PARALLEL_LINES, SCREEN_WIDTH - col);
                lcd_guard.send_lines(col, &bgbuf, cols).unwrap();
                col += cols;
            }
        });
    }

    /// Paint the entire screen with a single colour.
    pub fn send_color(&self, color: RGB24) {
        // Fill the internal frame buffer first
        {
            let mut lcd_guard = self.lock().unwrap();
            lcd_guard.buffer_fillcolor(color);
            // Clone the buffer so the thread can own it
            let buf = lcd_guard.framebuf.clone();
            drop(lcd_guard); // release lock before spawning

            if let Some(buf) = buf {
                let lcd_clone = self.clone();
                thread::spawn(move || {
                    let mut lcd_guard = lcd_clone.lock().unwrap();
                    let mut col = 0;
                    while col < SCREEN_WIDTH {
                        let cols = std::cmp::min(PARALLEL_LINES, SCREEN_WIDTH - col);
                        lcd_guard.send_lines(col, &buf, cols).unwrap();
                        col += cols;
                    }
                });
            }
        }
    }

    /// Helper – scroll the framebuffer onto the screen.
    pub fn scroll_buffer(&self, screenoffset: usize, reset_scroll: bool) -> Result<(), EspError> {
        if reset_scroll { self.scroll_screen(0)?; }

        let mut pos = 0;
        while pos < screenoffset {
            let rem  = screenoffset - pos;
            let cols = std::cmp::min(PARALLEL_LINES, rem);
            if let Some(ref buf) = self.framebuf {
                let start = pos * BUFFER_STRIDE;
                let end   = start + cols * SCREEN_HEIGHT;
                self.send_lines(pos, &buf[start..end], cols)?;
                self.scroll_screen((SCREEN_WIDTH - pos) as u16)?;
            }
            pos += cols;
        }
        Ok(())
    }

    // The following two functions are kept for API compatibility but
    // are no‑ops in this simplified implementation. */
    pub fn send_line_finish(&self)  { /* no-op */ }
    pub fn send_scroll_finish(&self) { /* no-op */ }

    // ---------------------------------------------------------------------
    //  Helper: clone the Arc so it can be moved into a thread
    // ---------------------------------------------------------------------
    fn clone(&self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            spi: self.spi.clone(),
            dc:  self.dc.clone(),
            rst: self.rst.clone(),
            framebuf: self.framebuf.clone(),
        }))
    }
}

// ---------------------------------------------------------------------
//  Public driver wrapper – owns the Arc<Mutex<Lcd>> and exposes a clean API
//  that automatically locks/unlocks for the caller.
// ---------------------------------------------------------------------
pub struct LcdDriver <'a> {
    inner: Arc<Mutex<Lcd<'a>>>,
}

impl LcdDriver <'_> {
    /// Create a new driver (this will initialise the peripherals).
    pub fn new() -> Result<Self, Box<dyn core::error::Error>> {
        let lcd = Lcd::new()?;
        Ok(Self { inner: lcd })
    }

    /// Initialise the LCD (send the command table, reset, etc.).
    pub fn init(&self) -> Result<(), EspError> {
        let mut lcd = self.inner.lock().unwrap();
        lcd.init()
    }

    /// Return the 24‑bit ID of the controller.
    pub fn get_id(&self) -> Result<u32, EspError> {
        let mut lcd = self.inner.lock().unwrap();
        lcd.get_id()
    }

    /// Paint the whole frame buffer with a single colour.
    pub fn fill_color(&self, color: RGB24) {
        let mut lcd = self.inner.lock().unwrap();
        lcd.buffer_fillcolor(color);
    }

    /// Copy a sprite into the frame buffer at (x, y).
    pub fn sprite_to_buffer(&self,
                            x: usize, y: usize,
                            width: usize, height: usize,
                            bitmap: &[RGB24]) {
        let mut lcd = self.inner.lock().unwrap();
        lcd.buffer_sprite(x, y, width, height, bitmap);
    }

    /// Draw a sprite directly to the display.
    pub fn draw_sprite(&self,
                       sx: usize, y: usize,
                       width: usize, height: usize,
                       bitmap: &[RGB24]) -> Result<(), EspError> {
        let mut lcd = self.inner.lock().unwrap();
        lcd.draw_sprite(sx, y, width, height, bitmap)
    }

    /// Paint the whole screen with a single colour (spawns a background thread).
    pub fn paint_color(&self, color: RGB24) {
        let lcd = self.inner.clone();
        thread::spawn(move || {
            let lcd = lcd.lock().unwrap();
            lcd.send_color(color);
        });
    }

    /// Scroll the framebuffer onto the display.
    pub fn scroll(&self, screenoffset: usize, reset_scroll: bool) -> Result<(), EspError> {
        let mut lcd = self.inner.lock().unwrap();
        lcd.scroll_buffer(screenoffset, reset_scroll)
    }

    /// Scroll the screen vertically (used for text scrolling etc.).
    pub fn vertical_scroll(&self, value: u16) {
        let mut lcd = self.inner.lock().unwrap();
        lcd.scroll_screen(value).unwrap();
    }

    /// Dummy helpers to keep API compatible.
    pub fn send_line_finish(&self)  { let lcd = self.inner.lock().unwrap(); lcd.send_line_finish(); }
    pub fn send_scroll_finish(&self) { let lcd = self.inner.lock().unwrap(); lcd.send_scroll_finish(); }

    /// Generate a dummy background image and draw it.
    pub fn generate_and_draw_bg(&self) {
        let mut bg = vec![RGB24::default(); SCREEN_WIDTH * SCREEN_HEIGHT];
        // Simple colour gradient
        for col in 0..SCREEN_WIDTH {
            let col_color = RGB24 {
                pixelR: (col % 256) as u8,
                pixelG: ((col * 2) % 256) as u8,
                pixelB: ((col * 3) % 256) as u8,
            };
            for row in 0..SCREEN_HEIGHT {
                bg[col * BUFFER_STRIDE + row] = col_color;
            }
        }
        let lcd = self.inner.clone();
        thread::spawn(move || {
            let lcd = lcd.lock().unwrap();
            lcd.draw_bg(bg);
        });
    }
}

// ---------------------------------------------------------------------
//  Example usage – this would normally go in `main.rs` (or an async task)
// ---------------------------------------------------------------------
#[unsafe(no_mangle)] //DO NOT DO THIS!!! 
pub extern "C" fn app_main() {  // Create the driver
    let driver = LcdDriver::new().expect("Failed to create LCD driver");
    driver.init().expect("Failed to initialise LCD"); // Initialise
    driver.paint_color(RGB24 { pixelR: 0, pixelG: 0, pixelB: 255 });
     FreeRtos::delay_ms(2000); // Wait a bit   
    driver.generate_and_draw_bg();// Generate and draw a background
    // ... rest of your application
}
