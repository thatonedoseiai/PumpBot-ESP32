//! This crate defines the structure of the global settings that is shared by the whole board.

#![no_std]
#![no_main]

extern crate alloc;

pub mod lang;
pub mod rgb;

use crate::lang::Lang;
use crate::rgb::RGB;
// use fontfile::rgb::RGB;
use alloc::string::{ToString, String};
use alloc::format;

use embassy_sync::{rwlock::RwLock, blocking_mutex::raw::CriticalSectionRawMutex};

pub static PB_GLOBAL_SETTINGS: RwLock<CriticalSectionRawMutex, PbGlobalSettings> = RwLock::new(PbGlobalSettings::new());

/// The main structure of global settings that the entire board will use
pub struct PbGlobalSettings {
    pub lang: Lang,
    pub theme: Theme,
}

impl PbGlobalSettings {
    /// Create a new instance of the global settings
    pub const fn new() -> Self {
        PbGlobalSettings {
            lang: Lang::En,
            theme: Theme::Dark,
        }
    }

    // break up this method later
    /// Set all the global settings from a string in the format `key=value&key=value...`
    /// TODO: stop unwrapping things and make like proper errors for this damn thing
    pub fn set_from_string(&mut self, s: String) {
        let mut split = s.split("&");
        let _ = split.next(); // theme
        let _ = split.next(); // theme color
        let _ = split.next(); // brightness
        let _ = split.next(); // rgb brightness
        let _ = split.next(); // rgb speed
        let _ = split.next(); // rgb mode
        let _ = split.next(); // rgb color 1
        let _ = split.next(); // rgb color 2
        let _ = split.next(); // pressure units
        self.lang = split.next().unwrap()[2..].parse::<u8>().unwrap().into(); // language
    }
}

//            1    2   3    4   5    6    7    8    9   10   11  12
// var ids=["ws","wp","t","tc","b","rb","rs","rm","rc","rc2","p","l"];
/// Turns the global settings into a string. DOES NOT INCLUDE WIFI OR PASSWORD.
impl ToString for PbGlobalSettings {
    fn to_string(&self) -> String {
        format!("0,#000000,0,0,0,0,#000000,#000000,0,{}", <Lang as Into<u8>>::into(self.lang))
    }
}

pub enum Theme {
    Dark,
    Light,
    Custom(RGB),
}

impl Theme {
    pub const fn bg(&self) -> RGB {
        match self {
            Theme::Dark => rgb![0, 10, 0],
            Theme::Light => rgb![255, 240, 255],
            Theme::Custom(r) => *r,
        }
    }

    pub const fn bg_secondary(&self) -> RGB {
        match self {
            Theme::Dark => rgb![20, 20, 20],
            Theme::Light => rgb![230, 230, 230],
            Theme::Custom(r) => {
                if r.luminance() > 100 {
                    rgb![30, 30, 30]
                } else {
                    rgb![172, 172, 172]
                }
            }
        }
    }

    pub const fn fg(&self) -> RGB {
        match self {
            Theme::Dark => rgb![255, 255, 255],
            Theme::Light => rgb![0, 0, 0],
            Theme::Custom(r) => if r.luminance() > 100 {
                rgb![0, 0, 0]
            } else {
                rgb![255, 255, 255]
            },
        }
    }

    pub const fn highlight(&self) -> RGB {
        rgb![255, 0, 0] // TODO: make this an appropriate colour
    }
}

pub enum ThemedColor {
    Bg,
    SecondaryBg,
    Fg,
    Highlight,
}

impl ThemedColor {
    pub async fn get(&self) -> RGB {
        let theme = &PB_GLOBAL_SETTINGS.read().await.theme;
        match self {
            Self::Bg => theme.bg(),
            Self::Fg => theme.fg(),
            Self::SecondaryBg => theme.bg_secondary(),
            Self::Highlight => theme.highlight(),
        }
    }
}