//! # PumpBot!
//!
//! Welcome to the source code for PumpBot. PumpBot is a programmable and remote-controllable PWM
//! controller for motors, lights, and other such things.
//!
//! Different functionalities of the real-time operating system are separated into 
//! individual crates for ease of programming and separation of responsibilities. 
//! - [rotenc] - polls the rotary encoder safely.
//! - [ledc] - controls the LED colours and brightnesses via PWM
//! - [fontfile] - handles the cbi and cbf font files.
//! - [ili9341] - drives the ILI9341 display.
//! - [st7735_lcd] - drives the ST7735 display.
//! - [button_idf] - polls the left and right buttons with debouncing
//! - [wifi] - handles the BLE and wifi radio, as well as HTTP server behaviour.
//! - [pwm] - drives and generates PWM signals using timers.
//! - [global_settings] - defines all the global data for the board e.g. languages and settings
//! - [menus] - defines all the shared behaviour for the menus, and contains a module for unifying
//! various events from different places into one shared event type
//!
//! There is a different module [sim] which we plan to use to simulate the screens. This can be
//! used for visual testing of menus.
//!
//! The main module's responsibility is to start the `main` function, which will initialize the board
//! and begin on the start menu. The user will then navigate through the menus, ending at the main
//! control menu. If the board has been previously initialized, `main` will skip all its
//! functionality and bring the user directly to the control menu.
//! TODO:
//! - [ ] Lua functionality
//! - [ ] Make everything async: we will start by using block_on.
//! - [ ] Wifi drivers
//!     - [x] draft
//!     - [ ] test
//! - [ ] http drivers
//!     - [x] draft
//!     - [ ] test
//! - [ ] menus

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

extern crate alloc;

use esp_backtrace as _;

use esp_hal::gpio::{Output, OutputConfig, Input, InputConfig, AnyPin, Level, Pull};
use esp_hal::peripherals::{GPIO9, GPIO10, GPIO11, GPIO12, GPIO46, SPI2, FLASH, Peripherals, SW_INTERRUPT, TIMG0};
use esp_hal::delay::Delay;
use esp_alloc::psram_allocator;
use esp_println::logger::init_logger_from_env;
use esp_rtos::start;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ram;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::spi::master::{Spi, Config};
use esp_hal::time::Rate;
use esp_hal::clock::CpuClock;
// use esp_alloc::HEAP;

use littlefs2::io;
use littlefs2::fs::{Filesystem, Allocation};
use static_cell::StaticCell;
use button_idf::button_init;
// use rotenc::{rotary_encoder_init, grab};
use rotenc::start_rotenc_thread;
use log::{info, error, warn};
use ledc::{LedController, LedPeripherals, LedMode};
// use pwm::{OutputCtl, OutputPeripherals, Action};
use pwm::{Pwm, PwmAction, Command};
use fontfile::{RGB, FontSize, PbFont, rgb, pb_font_renderer::PbFontRenderer};
// use ilidriver::ILIDriver;
use flash_storage::PbFlashStorage;
use dummy_pin::DummyPin;
use embedded_hal_bus::spi::ExclusiveDevice;
use ili9341::{DisplaySize240x320, Ili9341, Orientation as ILIOrientation};
use st7735_lcd::{ST7735, Orientation as STOrientation};
// use std::sync::Arc;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use menus::{run_menu_loop, MenuSelection, IOHandles, Event, Screen};
use global_settings::PbGlobalSettings;
use display_interface_spi::SPIInterface;
use embedded_graphics::{
    prelude::*,
    pixelcolor::{Rgb565, raw::RawU16},
    primitives::{Triangle, Rectangle, PrimitiveStyle},
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    text::Text,
};
use embassy_sync::{
    rwlock::RwLock,
    blocking_mutex::raw::CriticalSectionRawMutex
};
use wifi::PbWifi;
use embassy_executor::Spawner;
use core::convert::Infallible;
use core::fmt;
use embedded_hal::delay::DelayNs;
use embassy_time::{Timer, Duration};

#[cfg(all(not(feature = "ST"), not(feature = "ILI")))]
compile_error!("Declare a screen to compile!");

#[cfg(any(all(feature = "ILI", feature = "ST")))]
compile_error!("You may only have one screen active at a time!");

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

#[derive(Debug, Copy, Clone)]
enum PbError {
    FSError(io::Error)
}

impl core::error::Error for PbError { }
impl fmt::Display for PbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PbError::FSError(e) => write!(f, "FS error! {:?}", e),
        }
    }
}

/// Initialize the appropriate screen.
#[cfg(feature = "ILI")]
fn init_screen<'a>(spi2: SPI2<'a>, dc: GPIO11<'a>, sclk: GPIO9<'a>, mosi: GPIO10<'a>, miso: GPIO46<'a>, rst: GPIO12<'a>) -> anyhow::Result<Screen<'a>> {
    info!("SCREEN: using ILI");
    let config = OutputConfig::default();
    let inputconfig = InputConfig::default().with_pull(Pull::Down);

    let sclk_driver = Output::new(sclk, Level::Low, config);
    let mosi_driver = Output::new(mosi, Level::Low, config);
    let miso_driver = Input::new(miso, inputconfig);
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
    let interface = SPIInterface::new(spidevice, dc_output);
    let display = Ili9341::new(interface, rst_output, &mut Delay::new(), Orientation::Landscape, DisplaySize240x320)?;
    Ok(Screen::ILI(display))
}

#[cfg(feature = "ST")]
fn init_screen<'a>(spi2: SPI2<'a>, sclk: GPIO9<'a>, mosi: GPIO10<'a>, dc: GPIO11<'a>, rst: GPIO12<'a>) -> anyhow::Result<Screen<'a>> {
    let config = OutputConfig::default();

    let sclk_driver = Output::new(sclk, Level::Low, config);
    let mosi_driver = Output::new(mosi, Level::Low, config);
    let spi = Spi::new(
        spi2,
        Config::default()
            .with_frequency(Rate::from_mhz(26)),
    )?.with_sck(sclk_driver)
        .with_mosi(mosi_driver);  // ConfigError
    let Ok(spidevice) = ExclusiveDevice::new(spi, DummyPin::new_low(), Delay::new());
    let dc_output = Output::new(dc, Level::Low, config);
    let rst_output = Output::new(rst, Level::Low, config);
    let mut s = ST7735::new(
        spidevice, // SPI
        dc_output, // DC
        Some(rst_output), // RST
        true,  // rgb
        false, // inverted
        128,   // width
        160    // height
    );
    // let mut delay = FreeRtos;
    let mut delay = Delay::new();
    let res = s.init(&mut delay)?;
    s.set_orientation(&STOrientation::PortraitSwapped)?;
    s.set_offset(1, 2);
    // info!("clearing screen!");
    s.clear(Rgb565::BLUE)?;
    // info!("screen result!");
    Ok(Screen::ST(s))
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let program_result = init_board(spawner, peripherals).await;

    match program_result {
        Ok(_) => info!("MAIN RETURNED."),
        Err(e) => error!("MAIN ERRORED OUT WITH CODE {}!", e)
    }

    loop { }
}

/// Initializes the board, then starts all the menus. `main` will stop in case of an error, causing
/// the board to reset. This is why it returns an `anyhow::Result<()>`
async fn init_board(spawner: Spawner, peripherals: Peripherals) -> anyhow::Result<()> {

    // These GPIO pins are in use by some feature of the module and should not be used.
    let _ = peripherals.GPIO27;
    let _ = peripherals.GPIO28;
    let _ = peripherals.GPIO29;
    let _ = peripherals.GPIO30;
    let _ = peripherals.GPIO31;
    let _ = peripherals.GPIO32;
    let _ = peripherals.GPIO33;
    let _ = peripherals.GPIO34;
    let _ = peripherals.GPIO35;
    let _ = peripherals.GPIO36;
    let _ = peripherals.GPIO37;

    esp_alloc::heap_allocator!(size: 128*1024);

    let psram_config = esp_hal::psram::PsramConfig {
        size: esp_hal::psram::PsramSize::Size(16 * 1024 * 1024), // or ::Size(8 * 1024 * 1024) if you want to force it
        // mode is picked up from ESP_HAL_CONFIG_PSRAM_MODE at build time on S3
        ..Default::default()
    };

    psram_allocator!(peripherals.PSRAM, esp_hal::psram, psram_config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    // let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // esp_idf_svc::sys::link_patches();
    // esp_idf_svc::log::EspLogger::initialize_default(); // Everything is fine when removing this line
    let fs = register_filesystem(peripherals.FLASH).await.map_err(|e| PbError::FSError(e))?;

    info!("STARTING APP!");
    let mut settings = PbGlobalSettings::new();

    // let peripherals = Peripherals::take()?;
    // let sys_loop = EspSystemEventLoop::take()?;
    // let nvs = EspDefaultNvsPartition::take()?;

    // let mut pb_wifi = PbWifi::new(peripherals.modem, sys_loop, nvs)?;
    // pb_wifi.connect("hidden".try_into().unwrap(), "".try_into().unwrap())?;
    // let http_server = PbHttpServer::start()?;

    let mut pb_wifi = PbWifi::new(spawner, peripherals.WIFI);

    // let button_queue: Arc<Queue<Event>> = Arc::new(Queue::new(4));
    // button_init(vec![peripherals.pins.gpio0.downgrade(), peripherals.pins.gpio3.downgrade(), peripherals.pins.gpio18.downgrade()], peripherals.timer00, button_queue.clone())?;
    // start_rotenc_thread(button_queue.clone(), peripherals.pins.gpio17.downgrade(), peripherals.pins.gpio8.downgrade())?;
    let button_receiver = button_init(spawner, vec![peripherals.GPIO0.into(), peripherals.GPIO3.into(), peripherals.GPIO18.into()]);
    let rotenc_receiver = start_rotenc_thread(spawner, peripherals.GPIO17.into(), peripherals.GPIO8.into());

    let ledperipherals = LedPeripherals::new(
        peripherals.GPIO14.into(),
        peripherals.GPIO21.into(),
        peripherals.GPIO47.into(),
        peripherals.LEDC,
    );
    let leddriver = LedController::new(spawner, ledperipherals, LedMode::Off);
    leddriver.set_brightness(128);
    let outputperipherals = Pwm::new(
        &spawner, 
        [
            peripherals.GPIO4.into(),
            peripherals.GPIO5.into(),
            peripherals.GPIO6.into(),
            peripherals.GPIO7.into(),
        ]);
    info!("initializing screen");

    #[cfg(feature = "ILI")]
    let mut screen = init_screen(
        peripherals.SPI2,
        peripherals.GPIO9,
        peripherals.GPIO46,
        peripherals.GPIO10,
        peripherals.GPIO11,
        peripherals.GPIO12)?;
    #[cfg(feature = "ST")]
    let mut screen = init_screen(
        peripherals.SPI2,
        peripherals.GPIO9,
        peripherals.GPIO10,
        peripherals.GPIO11,
        peripherals.GPIO12)?;


    // let mut screen = if cfg!(feature = "ILI") {
    // } else if cfg!(feature = "ST") {
    // } else if cfg!(feature = "sim") {
    // } else {
    //     panic!("enable a screen feature")
    // };

    let mut font = PbFont::new(fs.clone());
    font.set_size(FontSize::Sz14)?;

    let pb_font_style = PbFontRenderer::new(font);
    let color_vec: Vec<RGB> = (0..100).map(|x| { rgb![255-x] }).collect();

    // let mut backlight = PinDriver::output(peripherals.pins.gpio13.downgrade())?;
    let mut backlight = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
    backlight.set_high();

    let yoffset = 10;
    let thin_stroke = PrimitiveStyle::with_stroke(Rgb565::BLUE, 1);
    // let res = Triangle::new(
    //     Point::new(16, 16 + yoffset),
    //     Point::new(16 + 16, 16 + yoffset),
    //     Point::new(16 + 8, yoffset),
    // )
    // .into_styled(thin_stroke)
    // .draw(&mut screen);

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    // let _ = Text::new("Hello Rust!", Point::new(20, 30), pb_font_style)
    //     .draw(&mut screen);

    // match res {
    //     Err(x) => panic!("error in drawing triangle: {:?}", x),
    //     _ => {}
    // };

    // let outputctl = OutputCtl::new(outputperipherals, peripherals.timer10)?;

    run_menu_loop(spawner, MenuSelection::TitleMenu, &mut IOHandles::new(
                screen,
                leddriver,
                outputperipherals,
                pb_font_style,
                button_receiver,
                rotenc_receiver,
            )).await?;

    // loop {
    //     if let Some((ev, _)) = button_queue.recv_front(10) {
    //         match ev {
// Event::Button(x) => info!("Button Event! {}", x),
    //             Event::Rotenc(x) => info!("Rotenc Event! {}", x),
    //         }
    //     }
    //     // let mut info = grab()?;
    //     // info.pin_a.enable_interrupt()?;
    //     // info.pin_b.enable_interrupt()?;
        // info!("looping...");
        // http_server.update(&mut settings, &mut pb_wifi)?;
        // esp_idf_hal::delay::FreeRtos::delay_ms(5000);
    // }

    Ok(())
}

static PB_FLASH_STORAGE: StaticCell<PbFlashStorage> = StaticCell::new();
static PB_FLASH_ALLOC: StaticCell<Allocation<PbFlashStorage>> = StaticCell::new();
// static PB_FS: RwLock<CriticalSectionRawMutex, Option<Filesystem<'static, PbFlashStorage>>> = RwLock::new(None); // StaticCell::new();

/// Links the filesystem to FreeRTOS. This function is a wrapper that uses a bunch of unsafe C.
/// Please do not change this, as it is known to work. If it fails, it will return an `Err(EspError)`
async fn register_filesystem(flash: FLASH<'static>) -> Result<Arc<Filesystem<'static, PbFlashStorage>>, io::Error> {
    let pb_flash_storage = PB_FLASH_STORAGE.init(PbFlashStorage::new(flash));

    let alloc = PB_FLASH_ALLOC.init(Filesystem::allocate());
    let fs = Filesystem::mount_or_else(
            alloc, 
            pb_flash_storage, 
            |_,storage,_| {
                info!("filesystem not found or formatted incorrectly... formatting before mounting!");
                Filesystem::format(storage)
            })?;

    use littlefs2::path;
    info!("READING /:");
    fs.read_dir_and_then(path!("/"), |contents| {
        for entry in contents {
            info!("{}", entry?.path());
        }
        Ok(())
        // contents.for_each(|entry| info!("{}", entry?.path()));
    })?;

    // Filesystem::format(pb_flash_storage)?;
    // let fs = PB_FS.init(Filesystem::mount(alloc, pb_flash_storage)?);

    // unsafe {
    //     // 0x210000 is the physical offset, size is 4096 bytes
    //     esp_hal::rom::Cache_Invalidate_Addr(0x210000, 4096);
    // }

    // let mut fs_lock = PB_FS.write().await;
    // *fs_lock = Some(fs);

    // test_storage(unsafe {fs.borrow_storage_mut()});

    Ok(Arc::new(fs))
}

fn test_storage(flash: &mut PbFlashStorage<'static>) {
    use littlefs2::driver::Storage;
    // const TEST_PARTITION_OFFSET: usize = 0x210000;
    const TEST_PARTITION_OFFSET: usize = 0;
    let test_write_data: [u8; 16] = [0xAA, 0xBB, 0xCC, 0xDD, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0xAA, 0xFF];
    let mut test_read_buf: [u8; 16] = [0; 16];

    info!("=== BEGIN STORAGE SANITY CHECK ===");

    // let &mut flash = PB_FLASH_STORAGE;

    // if let Err(e) = flash.write(TEST_PARTITION_OFFSET, &[0xFF; 4096]) {
    if let Err(e) = flash.erase(TEST_PARTITION_OFFSET, 4096) {
        error!("Flash Erase Failed! Error: {:?}", e);
    } else {
        info!("Erase command sent successfully.");
    }

    if let Err(e) = flash.read(TEST_PARTITION_OFFSET, &mut test_read_buf) {
        error!("Flash Read (post-erase) Failed! Error: {:?}", e);
    } else {
        info!("Read post-erase: {:?}", test_read_buf);
        if test_read_buf != [0xFF; 16] {
            warn!("CRITICAL: Sector did not return all 0xFF after erase! Cache or hardware lock issue.");
        }
    }

    info!("Writing test pattern to offset 0x{:X}...", TEST_PARTITION_OFFSET);
    if let Err(e) = flash.write(TEST_PARTITION_OFFSET, &test_write_data) {
        error!("Flash Write Failed! Error: {:?}", e);
    } else {
        info!("Write command sent successfully.");
    }

    info!("Reading back data to verify match...");
    if let Err(e) = flash.read(TEST_PARTITION_OFFSET, &mut test_read_buf) {
        error!("Flash Read (post-write) Failed! Error: {:?}", e);
    } else {
        info!("Read post-write: {:?}", test_read_buf);
        if test_read_buf == test_write_data {
            info!("SUCCESS: Raw flash driver is reading and writing properly!");
        } else {
            error!("FAILURE: Data mismatch! Expected {:?}, got {:?}", test_write_data, test_read_buf);
        }
    }

    info!("--- END FLASH HARDWARE SANITY CHECK ---");
}


use core::ffi::c_char;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strspn(s: *const c_char, accept: *const c_char) -> usize {
    let mut count = 0;
    let mut s_ptr = s;

    while *s_ptr != 0 {
        let mut a_ptr = accept;
        let mut found = false;
        while *a_ptr != 0 {
            if *s_ptr == *a_ptr {
                found = true;
                break;
            }
            a_ptr = a_ptr.add(1);
        }
        if !found {
            break;
        }
        count += 1;
        s_ptr = s_ptr.add(1);
    }
    count
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcspn(s: *const c_char, reject: *const c_char) -> usize {
    let mut count = 0;
    let mut s_ptr = s;

    while *s_ptr != 0 {
        let mut r_ptr = reject;
        while *r_ptr != 0 {
            if *s_ptr == *r_ptr {
                return count;
            }
            r_ptr = r_ptr.add(1);
        }
        count += 1;
        s_ptr = s_ptr.add(1);
    }
    count
}
