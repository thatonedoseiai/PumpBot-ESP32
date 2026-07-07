//! This module defines the functionality that uses the ESP32 radio.
//!
//! This includes connecting and disconnecting from Wifi, any BLE stuff, or even the HTTP server.
//! Server logic DOES NOT GO HERE. 
#![no_std]
#![no_main]
// #![feature(str_split_remainder)]

extern crate alloc;

// mod http_server;

// use esp_idf_svc::{
//     hal::modem::WifiModemPeripheral,
//     eventloop::EspSystemEventLoop,
//     nvs::EspDefaultNvsPartition,
//     sys::EspError,
//     wifi::{AuthMethod, BlockingWifi, ClientConfiguration, EspWifi, AccessPointInfo},
//     wifi,
// };

use embassy_executor::Spawner;
use embassy_net::{Config as EConfig, StackResources, DhcpConfig, Stack, Runner};
// use static_cell::make_static;
use embassy_futures::block_on;
use esp_radio::wifi::{ControllerConfig, WifiController, Config, sta::{StationConfig, ConnectedInfo}, AuthenticationMethod, WifiError, Interface, ap::AccessPointInfo, scan::ScanConfig, Ssid};
use esp_hal::rng::Rng;
use esp_hal::peripherals::WIFI;
// pub use crate::http_server::{PbHttpServer, PbHttpServerError};
use log::info;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
// use std::cell::RefCell;

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

/// Represents the state of the ESP32 radio. Currently only supports blocking wifi modes.
pub struct PbWifi<'a> {
    // wifi_mod: BlockingWifi<EspWifi<'a>>, // NOTE: Does this need to be blocking? Might need interior mutability
    // http_config: server::Configuration,
    wifi_mod: WifiController<'a>,
    netstack: Stack<'a>, // Runner<'a, Interface<'a>>,
    connected_info: Option<ConnectedInfo>,
    prev_pass: Option<String>
}

impl<'a> PbWifi<'a> {
    /// Turn on the radio and initialize it to a blank state.
    /// We require the modem peripheral, the event loop and default partition for the board to
    /// initialize the wifi radio to the right state.
    // pub fn new<M: WifiModemPeripheral + 'a>(modem: M, sys_loop: EspSystemEventLoop, nvs: EspDefaultNvsPartition) -> Result<Self, EspError> {
        // let wifi_mod = BlockingWifi::wrap(
        //     EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, 
        //     sys_loop)?;
    pub fn new(spawner: Spawner, wifi: WIFI<'a>) -> Result<Self, WifiError> {
        let wifi_mod = WifiController::new(wifi, ControllerConfig::default())?;

        let dhcpv4_config = EConfig::dhcpv4(DhcpConfig::default());
        let rng = Rng::new();
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;

        let (netstack, runner) = embassy_net::new(Interface::station(), dhcpv4_config, mk_static!(StackResources<3>, StackResources::<3>::new()), seed);

        spawner.spawn(run_netstack(runner).unwrap());

        Ok(PbWifi {
            wifi_mod,
            netstack,
            connected_info: None,
            prev_pass: None
        })
    }

    /// Connect the ESP radio to an existing wifi network. The network name is given by `ssid` and
    /// the password is given by `pass`. Currently does not support different authentication
    /// methods.
    pub fn connect(&mut self, ssid: &str, pass: &str) -> Result<(), WifiError> {
        info!("Attempting to connect to {} with password {}.", &ssid, &pass);

        let auth_method = if pass.len() == 0 {
            AuthenticationMethod::None
        } else {
            AuthenticationMethod::Wpa2Personal
        };

        let wifi_configuration = Config::Station(StationConfig::default()
            .with_ssid(ssid)
            .with_password(pass.to_string())
            .with_auth_method(auth_method));

        self.wifi_mod.set_config(&wifi_configuration)?;
        self.connected_info = Some(block_on(self.wifi_mod.connect_async())?);

        block_on(self.netstack.wait_config_up());
        if let Some(config) = self.netstack.config_v4() {
            info!("Wifi netif up at IP {}", config.address);
        }

        self.prev_pass = Some(pass.to_string());

        // let wifi_configuration: wifi::Configuration = wifi::Configuration::Client(ClientConfiguration {
        //     ssid: ssid,
        //     bssid: None,
        //     auth_method: if pass.len() == 0 { AuthMethod::None } else { AuthMethod::WPA2Personal },
        //     password: pass,
        //     channel: None,
        //     ..Default::default()
        // });

        // self.wifi_mod.set_configuration(&wifi_configuration)?;

        // self.wifi_mod.start()?;
        // info!("Wifi started.");

        // self.wifi_mod.connect()?;
        // info!("Wifi connected");

        // self.wifi_mod.wait_netif_up()?;
        // info!("Wifi netif up at IP {}.", self.wifi_mod.wifi().sta_netif().get_ip_info()?.ip);

        Ok(())
    }

    pub fn get_wifis(&mut self) -> Result<Vec<AccessPointInfo>, WifiError> {
        // Ok(self.wifi_mod.scan_n::<10>()?.0.to_vec())
        let scan_config = ScanConfig::default().with_max(10);
        block_on(self.wifi_mod.scan_async(&scan_config))
    }

    pub fn currently_connected(&self) -> Result<(Ssid, String), WifiError> {
        if let Some(config) = &self.connected_info {
            Ok((config.ssid, self.prev_pass.clone().unwrap_or("".to_string())))
        } else {
            Ok(("".into(), "".to_string())) // TODO: replace this with an error of some kind!
        }
        // if let wifi::Configuration::Client(k) = self.wifi_mod.get_configuration()? {
        //     return Ok((k.ssid.to_string(), k.password.to_string()));
        // }
        // Ok(("".to_string(), "".to_string())) // TODO: replace this with an error of some kind!
    }
}

#[embassy_executor::task]
async fn run_netstack(mut runner: Runner<'static, Interface>) {
    runner.run().await
}
