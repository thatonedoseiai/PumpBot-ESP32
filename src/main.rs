mod event;

use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::queue::Queue;
use button_idf::button_init;
// use rotenc::{rotary_encoder_init, grab};
use rotenc::start_rotenc_thread;
use log::info;
use event::Event;
use ledc::{LedController, LedPeripherals, LedMode, RGB};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default(); // Everything is fine when removing this line

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
    let leddriver = LedController::new(ledperipherals, LedMode::Rainbow);
    leddriver.set_brightness(128);

    info!("INITIALIZED BUTTONS!");
    loop {
        if let Some((ev, _)) = button_queue.recv_front(10) {
            match ev {
                Event::Button(x) => info!("Button Event! {}", x),
                Event::Rotenc(x) => info!("Rotenc Event! {}", x),
            }
        }
        // let mut info = grab()?;
        // info.pin_a.enable_interrupt()?;
        // info.pin_b.enable_interrupt()?;
    }
}
