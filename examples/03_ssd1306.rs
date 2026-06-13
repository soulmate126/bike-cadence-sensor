//! 示例 3：SSD1306 OLED 显示 "Hello Bike"

use bike_cadence_sensor::{display, init};

fn main() {
    init();
    display::run_hello_demo();
}
