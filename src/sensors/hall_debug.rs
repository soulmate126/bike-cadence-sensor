//! 霍尔原始边沿监测（无踏频算法，用于接线调试）

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{PinDriver, Pull};
use esp_idf_hal::peripherals::Peripherals;

use crate::board::HALL_GPIO;

/// 示例 2：打印 GPIO 边沿与电平，不返回。
pub fn run_raw_monitor() -> ! {
    let peripherals = Peripherals::take().expect("peripherals");
    let hall = PinDriver::input(peripherals.pins.gpio4, Pull::Floating).expect("hall gpio");

    log::info!("hall_debug: KY-003 S -> GPIO{HALL_GPIO} (+ -> 3V3, - -> GND)");
    log::info!("hall_debug: module LED should light when magnet is near");

    let mut last = hall.is_high();
    let mut pulses: u32 = 0;
    let mut ticks: u32 = 0;

    log::info!(
        "hall_debug: GPIO{HALL_GPIO} initial level: {}",
        if last { "HIGH" } else { "LOW" }
    );

    loop {
        let level = hall.is_high();
        if level != last {
            log::info!(
                "hall_debug: GPIO{HALL_GPIO} edge: {} -> {}",
                if last { "HIGH" } else { "LOW" },
                if level { "HIGH" } else { "LOW" }
            );
            pulses += 1;
            log::info!("hall_debug: raw pulse #{pulses}");
        }
        last = level;

        ticks += 1;
        if ticks % 200 == 0 {
            log::info!(
                "hall_debug: GPIO{HALL_GPIO} level: {} (pulses={pulses})",
                if level { "HIGH" } else { "LOW" }
            );
        }

        FreeRtos::delay_ms(5);
    }
}
