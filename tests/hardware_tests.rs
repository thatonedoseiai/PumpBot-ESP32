#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::peripherals::Peripherals;
use embassy_executor::Spawner;

use esp_hal::clock::CpuClock;

esp_bootloader_esp_idf::esp_app_desc!();

#[cfg(test)]
#[esp_hal::main]
fn main() -> ! {
    let p = tests::init();
    tests::test_basic_assertion(p);
    loop { }
}

#[cfg(test)]
// #[embedded_test::tests]
mod tests {
    use super::*;

    // #[init]
    pub(crate) fn init() -> Peripherals {
        // let peripherals = Peripherals::take();
        esp_println::logger::init_logger_from_env();
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);
        peripherals
    }

    // #[test]
    pub(crate) fn test_basic_assertion(_state: Peripherals) {
        assert_eq!(2 + 3, 4);
        log::info!("beh");
    }
}