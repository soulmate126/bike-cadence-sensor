//! 示例 5：霍尔踏频 + BLE CSC GATT Server（NimBLE）
//!
//! KY-003 → GPIO4 → CadenceCalculator → CSC Measurement (0x2A5B) Notify
//!
//! 验证：
//! - nRF Connect 扫描 "DIY Cadence Sensor"，连接后订阅 0x2A5B
//! - 转曲柄/靠近磁铁，观察通知数据递增
//! - 华为 GT5 Pro：设置 → 健康与健身设备 → 添加设备 → 踏频器

use bike_cadence_sensor::{app, init};

fn main() {
    init();
    app::run_hall_ble();
}
