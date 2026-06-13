//! 示例 6：霍尔踏频 + OLED 显示
//!
//! 磁铁 → KY-003 → GPIO4 → CadenceCalculator → SSD1306
//!
//! 烧录：`./cargo run --example 06_hall_oled`

use bike_cadence_sensor::{app, init};

fn main() {
    init();
    app::run_hall_oled();
}
