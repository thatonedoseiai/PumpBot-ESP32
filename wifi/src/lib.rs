//! This module defines the functionality that uses the ESP32 radio.
//!
//! This includes connecting and disconnecting from Wifi, any BLE stuff, or even the HTTP server.
//! Server logic DOES NOT GO HERE. 

use esp_idf_svc::{
    hal::modem::WifiModemPeripheral,
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    sys::EspError,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, EspWifi},
    wifi,
    http::server,
};
use log::info;
// use std::cell::RefCell;

/// Represents the state of the ESP32 radio. Currently only supports blocking wifi modes.
pub struct PbWifi<'a> {
    wifi_mod: BlockingWifi<EspWifi<'a>>, // NOTE: Does this need to be blocking? Might need interior mutability
    // http_config: server::Configuration,
}

impl<'a> PbWifi<'a> {
    /// Turn on the radio and initialize it to a blank state.
    /// We require the modem peripheral, the event loop and default partition for the board to
    /// initialize the wifi radio to the right state.
    pub fn new<M: WifiModemPeripheral + 'a>(modem: M, sys_loop: EspSystemEventLoop, nvs: EspDefaultNvsPartition) -> Result<Self, EspError> {
        let wifi_mod = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, 
            sys_loop)?;

        Ok(PbWifi {
            wifi_mod
        })
    }

    /// Connect the ESP radio to an existing wifi network. The network name is given by `ssid` and
    /// the password is given by `pass`. Currently does not support different authentication
    /// methods.
    pub fn connect(&mut self, ssid: heapless::String<32>, pass: heapless::String<64>) -> Result<(), EspError> {
        let wifi_configuration: wifi::Configuration = wifi::Configuration::Client(ClientConfiguration {
            ssid: ssid,
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: pass,
            channel: None,
            ..Default::default()
        });

        self.wifi_mod.set_configuration(&wifi_configuration)?;

        self.wifi_mod.start()?;
        info!("Wifi started");

        self.wifi_mod.connect()?;
        info!("Wifi connected");

        self.wifi_mod.wait_netif_up()?;
        info!("Wifi netif up");

        Ok(())
    }
}
