//! 板载 LED 状态指示（GPIO8，低电平点亮）

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;

use super::LED_GPIO;

/// 示例 1：GPIO8 板载 LED 每 500 ms 翻转一次（不返回）。
pub fn run_blink() -> ! {
    let peripherals = Peripherals::take().expect("peripherals");
    let mut led = PinDriver::output(peripherals.pins.gpio8).expect("led gpio");

    log::info!("blink: toggling GPIO{LED_GPIO} every 500ms");

    loop {
        led.toggle().expect("led toggle");
        FreeRtos::delay_ms(500);
    }
}
