//! 示例 1：GPIO 点灯（板载 LED）

use bike_cadence_sensor::{board::led, init};

fn main() {
    init();
    led::run_blink();
}
