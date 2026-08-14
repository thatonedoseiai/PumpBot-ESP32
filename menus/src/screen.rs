#[cfg(not(feature = "sim"))]
pub mod screen {
    use embedded_graphics::{
        prelude::*,
        pixelcolor::Rgb565,
        primitives::Rectangle,
    };
    use ili9341::Ili9341;
    use st7735_lcd::ST7735;
    use core::convert::Infallible;
    // use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver};
    // use esp_idf_hal::spi::{SpiDriver, SpiDeviceDriver};
    use esp_hal::gpio::Output;
    use esp_hal::spi::master::{Spi};
    use esp_hal::Blocking;
    use esp_hal::delay::Delay;
    use dummy_pin::DummyPin;
    use embedded_hal_bus::spi::ExclusiveDevice;
    use display_interface_spi::SPIInterface;

    /// A generalized driver that wraps both kinds of screens. This wrapper can either contain an 
    /// ILI9341 driver, or an ST7735 driver. If a new kind of screen with a new kind of driver is
    /// required, we would put that new driver in here. This helps with modularity so that we can have
    /// more freedom with which screens we might want to use in the future.
    pub enum Screen<'a> {
        // ILI(Ili9341<SPIInterface<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, AnyIOPin, Output>>, PinDriver<'a, AnyIOPin, Output>>),
        ILI(Ili9341<SPIInterface<ExclusiveDevice<Spi<'a, Blocking>, DummyPin, Delay>, Output<'a>>, Output<'a>>),
        ST(ST7735<ExclusiveDevice<Spi<'a, Blocking>, DummyPin, Delay>, Output<'a>, Output<'a>>),
    }

    /// Wraps both kinds of errors that we can expect from the screen drawing.
    /// Either an ILI9341 is connected, and in such a case we would use [ili9341::DisplayError],
    /// otherwise an ST7735 is connected, in which case we would use [st7735_lcd::ST7735Error].
    #[derive(Debug)]
    pub enum ScreenDrawError {
        ILI(ili9341::DisplayError),
        ST(st7735_lcd::ST7735Error),
        Sim,
    }

    impl From<ili9341::DisplayError> for ScreenDrawError {
        fn from(val: ili9341::DisplayError) -> Self {
            ScreenDrawError::ILI(val)
        }
    }

    impl From<st7735_lcd::ST7735Error> for ScreenDrawError {
        fn from(val: st7735_lcd::ST7735Error) -> Self {
            ScreenDrawError::ST(val)
        }
    }

    // simulator is infallible. It is not necessary to include its error defns.
    impl From<Infallible> for ScreenDrawError {
        fn from(_: Infallible) -> Self {
            ScreenDrawError::Sim
        }
    }

    impl core::fmt::Display for ScreenDrawError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                ScreenDrawError::ILI(s) => write!(f, "{:?}", s),
                ScreenDrawError::ST(s) => write!(f, "{:?}", s),
                ScreenDrawError::Sim => write!(f, "Simulator draw error!"),
            }
        }
    }

    impl core::error::Error for ScreenDrawError { }

    /// This is necessary for [crate::Screen] to be usable with [embedded_graphics]. Required for
    /// [DrawTarget]
    impl OriginDimensions for Screen<'_> {
        fn size(&self) -> Size {
            match self {
                Screen::ILI(s) => s.size(),
                Screen::ST(s) => s.size(),
            }
        }
    }

    /// This is necessary for [crate::Screen] to be usable with [embedded_graphics].
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
}

#[cfg(feature = "sim")]
pub mod screen {
    use embedded_graphics_simulator::{SimulatorDisplay, Window, OutputSettingsBuilder, SimulatorEvent};
    use std::marker::PhantomData;
    use std::convert::Infallible;
    use embedded_graphics::{
        prelude::*,
        pixelcolor::Rgb565,
        primitives::Rectangle,
    };

    pub struct Screen<'a> {
        pub disp: SimulatorDisplay<Rgb565>,
        _marker: PhantomData<&'a ()>,
    }

    pub type ScreenDrawError = Infallible;

    impl Screen<'_> {
        pub fn new(disp: SimulatorDisplay<Rgb565>) -> Self {
            Screen { disp, _marker: PhantomData }
        }
    }

    /// This is necessary for [crate::Screen] to be usable with [embedded_graphics]. Required for
    /// [DrawTarget]
    impl OriginDimensions for Screen<'_> {
        fn size(&self) -> Size {
            self.disp.size()
        }
    }

    /// This is necessary for [crate::Screen] to be usable with [embedded_graphics].
    impl DrawTarget for Screen<'_> {
        type Color = Rgb565; // temporary
        type Error = Infallible;
        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
            where I: IntoIterator<Item = Pixel<Self::Color>> {
            self.disp.draw_iter::<I>(pixels)
        }

        fn fill_contiguous<I>(
            &mut self,
            area: &Rectangle,
            colors: I,
        ) -> Result<(), Self::Error>
           where I: IntoIterator<Item = Self::Color> {
               self.disp.fill_contiguous::<I>(area, colors)
        }

        fn fill_solid(
            &mut self,
            area: &Rectangle,
            color: Self::Color,
        ) -> Result<(), Self::Error> {
            self.disp.fill_solid(area, color)
        }

        fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
            self.disp.clear(color)
        }
    }
}
