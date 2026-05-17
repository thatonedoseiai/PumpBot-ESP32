//! This crate defines the structure of the global settings that is shared by the whole board.
pub mod lang;

use crate::lang::Lang;
use std::string::ToString;

/// The main structure of global settings that the entire board will use
pub struct PbGlobalSettings {
    pub lang: Lang,
}

impl PbGlobalSettings {
    /// Create a new instance of the global settings
    pub fn new() -> Self {
        PbGlobalSettings {
            lang: Lang::En,
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