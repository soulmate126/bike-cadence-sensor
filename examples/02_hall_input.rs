//! 示例 2：GPIO 输入 — 模拟霍尔传感器边沿计数

use bike_cadence_sensor::{board, init};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Input, PinDriver, Pull};
use esp_idf_hal::peripherals::Peripherals;

fn main() {
    init();

    let peripherals = Peripherals::take().unwrap();
    let hall = PinDriver::input(peripherals.pins.gpio3, Pull::Up).unwrap();

    log::info!(
        "02_hall_input: poll GPIO{} (connect hall DO or jumper to GND to simulate pulse)",
        board::HALL_GPIO
    );

    let mut last = hall.is_high();
    let mut pulses: u32 = 0;

    loop {
        let level = hall.is_high();
        // 下降沿：高 -> 低，模拟磁铁经过霍尔开关
        if last && !level {
            pulses += 1;
            log::info!("Hall pulse #{pulses}");
        }
        last = level;
        FreeRtos::delay_ms(5);
    }
}
