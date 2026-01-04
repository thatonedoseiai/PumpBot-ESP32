use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use std::vec::Vec;
use esp_idf_hal::task::queue::Queue;
use button_idf::button_init;
use log::info;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default(); // Everything is fine when removing this line

    info!("STARTING APP!");

    let peripherals = Peripherals::take()?;
    let button_queue = button_init(vec![peripherals.pins.gpio0.downgrade(), peripherals.pins.gpio3.downgrade(), peripherals.pins.gpio18.downgrade()], peripherals.timer00)?;

    info!("INITIALIZED BUTTONS!");
    loop {
        if let Some((ev, _)) = button_queue.recv_front(10) {
            info!("EVENT! {}", ev);
        }
    }
}
