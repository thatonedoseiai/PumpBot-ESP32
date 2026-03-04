mod event;
mod menu;
mod menus;
mod lang;

use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::queue::Queue;
use button_idf::button_init;
// use rotenc::{rotary_encoder_init, grab};
use rotenc::start_rotenc_thread;
use log::{info, error};
use event::Event;
use ledc::{LedController, LedPeripherals, LedMode};
use pwm::{OutputCtl, OutputPeripherals, Action};
use fontfile::{RGB, FontSize, PbFont, rgb};
// use ilidriver::ILIDriver;
// #[cfg(feature = "ILI")]
use ili9341::{DisplaySize240x320, Ili9341, Orientation as ILIOrientation};
// #[cfg(feature = "ST")]
use st7735_lcd::{ST7735, Orientation as STOrientation};

use std::sync::Arc;
use esp_idf_hal::delay::{Delay};
use esp_idf_hal::sys::{uxTaskGetStackHighWaterMark, EspError};
use esp_idf_hal::spi::{SpiDeviceDriver, config::{DriverConfig, Config}, SpiDriver, SpiError};
use esp_idf_sys::{esp_vfs_littlefs_conf_t, esp_vfs_littlefs_register};
use crate::menu::{run_menu_loop, MenuSelection, IOHandles};
use display_interface_spi::SPIInterface;
use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb565,
    primitives::{Triangle, PrimitiveStyle, Rectangle},
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    text::Text,
};

enum Screen<'a> {
    ILI(Ili9341<SPIInterface<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, AnyIOPin, Output>>, PinDriver<'a, AnyIOPin, Output>>),
    ST(ST7735<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, AnyIOPin, Output>, PinDriver<'a, AnyIOPin, Output>, {128*160*2}>)
}

#[derive(Debug)]
enum ScreenDrawError {
    ILI(ili9341::DisplayError),
    ST(st7735_lcd::Error<SpiError,GpioError>)
}

impl From<ili9341::DisplayError> for ScreenDrawError {
    fn from(val: ili9341::DisplayError) -> Self {
        ScreenDrawError::ILI(val)
    }
}

impl From<st7735_lcd::Error<SpiError,GpioError>> for ScreenDrawError {
    fn from(val: st7735_lcd::Error<SpiError,GpioError>) -> Self {
        ScreenDrawError::ST(val)
    }
}

impl std::fmt::Display for ScreenDrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenDrawError::ILI(s) => write!(f, "{:?}", s),
            ScreenDrawError::ST(s) => write!(f, "{:?}", s),
        }
    }
}

impl std::error::Error for ScreenDrawError { }

impl OriginDimensions for Screen<'_> {
    fn size(&self) -> Size {
        match self {
            Screen::ILI(s) => s.size(),
            Screen::ST(s) => s.size(),
        }
    }
}

impl DrawTarget for Screen<'_> {
    type Color = Rgb565; // temporary
    type Error = ScreenDrawError;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where I: IntoIterator<Item = Pixel<Self::Color>> {
        match self {
            Screen::ILI(s) => Ok(s.draw_iter::<I>(pixels)?),
            Screen::ST(s) => Ok(s.draw_iter::<I>(pixels)?),
        }
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
       where I: IntoIterator<Item = Self::Color> {
        match self {
            Screen::ILI(s) => Ok(s.fill_contiguous::<I>(area, colors)?),
            Screen::ST(s) => Ok(s.fill_contiguous::<I>(area, colors)?),
        }
    }

    fn fill_solid(
        &mut self,
        area: &Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        match self {
            Screen::ILI(s) => Ok(s.fill_solid(area, color)?),
            Screen::ST(s) => Ok(s.fill_solid(area, color)?),
        }
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        match self {
            Screen::ILI(s) => Ok(s.clear(color)?),
            Screen::ST(s) => Ok(s.clear(color)?),
        }
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default(); // Everything is fine when removing this line
    
    register_filesystem()?;

    // let contents = fs::read("/fs/test.txt")?;
    // let data = str::from_utf8(contents.as_slice())?;
    // info!("FILE READ: {}", data);

    info!("STARTING APP!");

    let peripherals = Peripherals::take()?;
    let button_queue: Arc<Queue<Event>> = Arc::new(Queue::new(4));
    button_init(vec![peripherals.pins.gpio0.downgrade(), peripherals.pins.gpio3.downgrade(), peripherals.pins.gpio18.downgrade()], peripherals.timer00, button_queue.clone())?;
    // rotary_encoder_init(peripherals.pins.gpio17.downgrade(),
    //     peripherals.pins.gpio8.downgrade(),
    //     button_queue.clone())?;
    start_rotenc_thread(button_queue.clone(), peripherals.pins.gpio17.downgrade(), peripherals.pins.gpio8.downgrade())?;
    let ledperipherals = LedPeripherals::new(
        peripherals.pins.gpio14.downgrade(), 
        peripherals.pins.gpio21.downgrade(),
        peripherals.pins.gpio47.downgrade(),
        peripherals.ledc.channel0,
        peripherals.ledc.channel1,
        peripherals.ledc.channel2,
        peripherals.ledc.timer0,
    );
    let leddriver = LedController::new(ledperipherals, LedMode::Off);
    leddriver.set_brightness(128);
    let outputperipherals = OutputPeripherals::new(
        ( peripherals.pins.gpio4.downgrade(),
          peripherals.pins.gpio5.downgrade(),
          peripherals.pins.gpio6.downgrade(),
          peripherals.pins.gpio7.downgrade(),
        ),
        ( peripherals.ledc.channel3,
          peripherals.ledc.channel4,
          peripherals.ledc.channel5,
          peripherals.ledc.channel6,
        ),
        peripherals.ledc.timer1
    );
    // let mut screen = ILIDriver::new(
    //     peripherals.spi2, 
    //     peripherals.pins.gpio11.downgrade(),
    //     peripherals.pins.gpio9.downgrade(),
    //     peripherals.pins.gpio46.downgrade(),
    //     peripherals.pins.gpio10.downgrade(),
    //     peripherals.pins.gpio12.downgrade(),
    //     peripherals.pins.gpio13.downgrade(),
    // )?;

    let mut screen = if cfg!(feature = "ILI") {
        let cspin: Option<AnyIOPin> = None;
        let spi_device_driver = SpiDeviceDriver::new_single(
            peripherals.spi2,
            peripherals.pins.gpio9.downgrade(),
            peripherals.pins.gpio46.downgrade(),
            Some(peripherals.pins.gpio10.downgrade()),
            cspin,
            &DriverConfig::default(),
            &Config::default()
        )?;
        let interface = SPIInterface::new(
            spi_device_driver,
            PinDriver::output(peripherals.pins.gpio11.downgrade())?
        );
        let s = Ili9341::new(
            interface,
            PinDriver::output(peripherals.pins.gpio12.downgrade())?,
            &mut Delay::new_default(),
            ILIOrientation::Landscape,
            DisplaySize240x320
        ).unwrap();
        info!("screen set to ILI");
        Screen::ILI(s)
    } else if cfg!(feature = "ST") {
        let cspin: Option<AnyIOPin> = None;
        let sdipin: Option<AnyIOPin> = None;
        let spi_device_driver = SpiDeviceDriver::new_single(
            peripherals.spi2,
            peripherals.pins.gpio9.downgrade(),
            peripherals.pins.gpio10.downgrade(),
            sdipin,
            cspin,
            &DriverConfig::default(),
            &Config::default()
        )?;
        let mut s = ST7735::new(
            // interface,
            spi_device_driver,
            PinDriver::output(peripherals.pins.gpio11.downgrade())?,
            Some(PinDriver::output(peripherals.pins.gpio12.downgrade())?),
            // DC
            // RST
            true, // rgb
            false, //inverted
            128,
            160,
        );
        let k = s.init(&mut Delay::new_default(),);
        match k {
            Ok(x) => {info!("initialization okay!")},
            Err(x) => error!("ST error: {:?}", x),
        }
        let f = s.set_orientation(&STOrientation::PortraitSwapped).map_err(|s| ScreenDrawError::from(s));
        match f {
            Ok(x) => {info!("orientation set okay!")},
            Err(x) => error!("ST error: {:?}", x),
        }
        info!("screen set to ST");
        Screen::ST(s)
    } else {
        panic!("Either ILI or ST feature must be enabled! Which display are you using?");
    };
    info!("screen clearing!");
    screen.clear(Rgb565::BLACK).expect_err("clearing screen");
    info!("screen cleared!");

    let mut font = PbFont::new();
    font.set_size(FontSize::Sz14)?;

    let (char_metrics, char_slice) = font.load_char(0x3d)?;

    info!("{:?}", char_metrics);
    info!("{:?}", char_slice.len());
    // info!("{:?}\n{:?}", char_metrics, char_slice);
    let color_vec: Vec<RGB> = (0..100).map(|x| { rgb![255-x] }).collect();
    // screen.display.draw_raw_slice(10, 10, 19, 19, color_vec.as_slice())?;
    // screen.display.draw_raw_slice(30, 30, 29+(char_metrics.height / 3), 29+char_metrics.width, char_slice.as_slice())?;
    // screen.draw_string(50, 50, "Hello blue!", &mut font, 16)?;

    let mut backlight = PinDriver::output(peripherals.pins.gpio13.downgrade())?;
    backlight.set_high()?;

    let yoffset = 10;
    let thin_stroke = PrimitiveStyle::with_stroke(Rgb565::BLUE, 1);
    let res = Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(&mut screen);

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let _ = Text::new("Hello Rust!", Point::new(20, 30), style)
        .draw(&mut screen);

    match res {
        Err(x) => panic!("error in drawing triangle: {:?}", x),
        _ => {}
    };
    info!("ALL DRAWING FINISHED");

    let outputctl = OutputCtl::new(outputperipherals, peripherals.timer10)?;
//     outputctl.buffer_action(Action::SetDuty(0, OutputCtl::max_duty / 2), 2000)?;
//     outputctl.buffer_action(Action::SetDuty(1, OutputCtl::max_duty), 2000)?;
//     outputctl.buffer_action(Action::On(0), 3000)?;
//     outputctl.buffer_action(Action::Off(0), 4000)?;
//     esp_idf_hal::delay::FreeRtos::delay_ms(5000);
//     outputctl.buffer_action(Action::On(1), 1000)?;

    // let mut taskstatuses = [TaskStatus_t::default(); 5];
    // let mut runtime: u32 = 0;
    // let runtimeptr: *mut u32 = &mut runtime;
    // unsafe {
    //     uxTaskGetSystemState(taskstatuses.as_mut_ptr(), taskstatuses.len() as u32, runtimeptr);
    // }

    // run_menu_loop(MenuSelection::TitleMenu, &mut IOHandles::new(/* screen,*/ leddriver, outputctl, font), button_queue)?;

    loop {
        info!("loop");
        esp_idf_hal::delay::FreeRtos::delay_ms(5000);
    }

    Ok(())

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
    // }
}

fn register_filesystem() -> Result<(), EspError> {
    let mut fs_config: esp_vfs_littlefs_conf_t = esp_vfs_littlefs_conf_t {
        base_path: c"/fs".as_ptr(),
        partition_label: c"filesystem".as_ptr(),
        ..Default::default()
    };
    fs_config.set_format_if_mount_failed(false as u8);
    fs_config.set_dont_mount(false as u8);

    unsafe {
        let res = esp_vfs_littlefs_register(&fs_config);
        EspError::convert(res)
    }
}

fn get_stack_size() {
    unsafe {
        let stack: u32 = uxTaskGetStackHighWaterMark(std::ptr::null_mut());
        info!("FREE STACK SPACE: {}", stack);
    }
}
