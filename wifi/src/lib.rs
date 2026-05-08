use esp_idf_svc::{
    hal::modem::WifiModemPeripheral,
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    sys::EspError,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;
// use std::cell::RefCell;

pub struct PbWifi<'a> {
    wifi_mod: BlockingWifi<EspWifi<'a>>, // NOTE: Does this need to be blocking? Might need interior mutability
}

impl<'a> PbWifi<'a> {
    pub fn new<M: WifiModemPeripheral + 'a>(modem: M, sys_loop: EspSystemEventLoop, nvs: EspDefaultNvsPartition) -> Result<Self, EspError> {
        let wifi_mod = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, 
            sys_loop)?;

        Ok(PbWifi {
            wifi_mod
        })
    }

    pub fn connect(&mut self, ssid: heapless::String<32>, pass: heapless::String<64>) -> Result<(), EspError> {
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
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
