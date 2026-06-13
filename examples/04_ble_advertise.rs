//! 示例 4：NimBLE 广播设备名 "DIY Cadence Sensor"

use bike_cadence_sensor::{ble::advertise, init};

fn main() {
    init();
    advertise::run_advertise();
}
