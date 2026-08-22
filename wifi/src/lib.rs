//! This module defines the functionality that uses the ESP32 radio.
//!
//! This includes connecting and disconnecting from Wifi, any BLE stuff, or even the HTTP server.
//! Server logic DOES NOT GO HERE. 
#![no_std]
#![no_main]
// #![feature(str_split_remainder)]

mod wget;

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
// use embassy_futures::block_on;
use esp_radio::wifi::{ControllerConfig, WifiController, Config, sta::{StationConfig, ConnectedInfo}, AuthenticationMethod, WifiError, Interface, ap::AccessPointInfo, scan::{ScanConfig, ScanTypeConfig}};
use esp_hal::rng::Rng;
use esp_hal::peripherals::WIFI;
// pub use crate::http_server::{PbHttpServer, PbHttpServerError};
use log::{info, warn};
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use core::cell::{RefCell, Ref};
use alloc::rc::Rc;
use esp_hal::time::Duration;
use wget::wget;
use embedded_io_async::ErrorKind;
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
    pub netstack: Stack<'a>, // Runner<'a, Interface<'a>>,
    connected_info: Option<ConnectedInfo>,
    ap_cache: Rc<RefCell<Vec<AccessPointInfo>>>,
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

        let (netstack, runner) = embassy_net::new(Interface::station(), dhcpv4_config, mk_static!(StackResources<6>, StackResources::<6>::new()), seed);

        spawner.spawn(run_netstack(runner).unwrap());

        Ok(PbWifi {
            wifi_mod,
            netstack,
            connected_info: None,
            ap_cache: Rc::new(RefCell::new(Vec::new())),
        })
    }

    /// Connect the ESP radio to an existing wifi network. The network name is given by `ssid` and
    /// the password is given by `pass`. Currently does not support different authentication
    /// methods.
    pub async fn connect(&mut self, ap: &AccessPointInfo, pass: &str) -> Result<(), WifiError> {
        info!("Attempting to connect to {} with password {}.", &ap.ssid.as_str(), &pass);

        let wifi_configuration = if ap.auth_method.unwrap_or(AuthenticationMethod::None) == AuthenticationMethod::None {
            warn!("ignoring password because there is no auth method");
            Config::Station(StationConfig::default()
                .with_ssid(ap.ssid)
                .with_auth_method(AuthenticationMethod::None))
        } else {
            Config::Station(StationConfig::default()
                .with_ssid(ap.ssid)
                .with_password(pass.to_string())
                .with_auth_method(ap.auth_method.unwrap_or(AuthenticationMethod::None)))
        };

        self.wifi_mod.set_config(&wifi_configuration)?;
        self.connected_info = Some(self.wifi_mod.connect_async().await?);
        info!("connect_async connected!");

        self.netstack.wait_config_up().await;
        if let Some(config) = self.netstack.config_v4() {
            info!("Wifi netif up at IP {}", config.address);
        }

        Ok(())
    }

    pub fn get_wifis(&self) -> Ref<'_, Vec<AccessPointInfo>> {
        self.ap_cache.borrow()
        // Ok(self.wifi_mod.scan_n::<10>()?.0.to_vec())
    }

    pub async fn scan(&mut self) -> Result<(), WifiError> {
        let scan_config = ScanConfig::default()
            .with_max(10)
            .with_scan_type(ScanTypeConfig::Active {
                min: Duration::from_millis(10), 
                max: Duration::from_millis(300)
            });
        let scan_result = self.wifi_mod.scan_async(&scan_config).await?;
        if !scan_result.is_empty() {
            self.ap_cache.replace(scan_result);
        }
        Ok(())
    }

    pub async fn wget(&self, url: &str) -> anyhow::Result<String> {
        if let Some(_) = self.connected_info {
            Ok(wget(self.netstack, url).await?)
        } else {
            Err(reqwless::Error::Network(ErrorKind::NotConnected).into())
        }
    }
}

#[embassy_executor::task]
async fn run_netstack(mut runner: Runner<'static, Interface>) {
    runner.run().await
}
