//! 示例 1：GPIO 点灯（板载 LED）

use bike_cadence_sensor::{board, init};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;

fn main() {
    init();

    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio8).unwrap();

    log::info!("01_blink: toggling GPIO{} every 500ms", board::LED_GPIO);

    loop {
        led.toggle().unwrap();
        FreeRtos::delay_ms(500);
    }
}
