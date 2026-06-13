//! 霍尔踏频传感器主固件
//!
//! KY-003 → GPIO4 → CadenceCalculator
//!   ├─ SSD1306 OLED（GPIO5/6 I2C）显示 RPM / 转数
//!   └─ NimBLE CSC 服务 0x1816 → 0x2A5B Notify

use bike_cadence_sensor::{app, init};

fn main() {
    init();
    app::run_full();
}
