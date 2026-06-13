//! 示例 2：GPIO 输入 — 模拟霍尔传感器边沿计数

use bike_cadence_sensor::{init, sensors::hall_debug};

fn main() {
    init();
    hall_debug::run_raw_monitor();
}
