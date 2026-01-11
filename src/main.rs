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
use pwm::{OutputCtl, OutputPeripherals, Action};
use std::sync::Arc;
use esp_idf_hal::sys::{uxTaskGetStackHighWaterMark};

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
    let outputctl = OutputCtl::new(outputperipherals, peripherals.timer10)?;
    outputctl.buffer_action(Action::SetDuty(0, OutputCtl::max_duty / 2), 2000)?;
    outputctl.buffer_action(Action::SetDuty(1, OutputCtl::max_duty), 2000)?;
    outputctl.buffer_action(Action::On(0), 3000)?;
    outputctl.buffer_action(Action::Off(0), 4000)?;
    esp_idf_hal::delay::FreeRtos::delay_ms(5000);
    outputctl.buffer_action(Action::On(1), 1000)?;

    // let mut taskstatuses = [TaskStatus_t::default(); 5];
    // let mut runtime: u32 = 0;
    // let runtimeptr: *mut u32 = &mut runtime;
    // unsafe {
    //     uxTaskGetSystemState(taskstatuses.as_mut_ptr(), taskstatuses.len() as u32, runtimeptr);
    // }

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

fn get_stack_size() {
    unsafe {
        let stack: u32 = uxTaskGetStackHighWaterMark(std::ptr::null_mut());
        info!("FREE STACK SPACE: {}", stack);
    }
}
