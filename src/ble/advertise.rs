//! 仅广播设备名（无 GATT 服务）

use esp32_nimble::{BLEAdvertisementData, BLEDevice, BLEError};
use esp_idf_svc::hal::delay::FreeRtos;

use super::server::DEVICE_NAME;

/// 示例 4：只广播 `DEVICE_NAME`，不注册 CSC 服务。
pub fn start_only() -> Result<(), BLEError> {
    let ble_device = BLEDevice::take();
    let ble_advertising = ble_device.get_advertising();

    let mut adv_data = BLEAdvertisementData::new();
    adv_data.name(DEVICE_NAME);
    ble_advertising.lock().set_data(&mut adv_data)?;
    ble_advertising.lock().start()?;

    log::info!("ble_advertise: broadcasting \"{DEVICE_NAME}\" — scan with nRF Connect");
    Ok(())
}

/// 示例 4 主循环（不返回）。
pub fn run_advertise() -> ! {
    if let Err(e) = start_only() {
        log::error!("ble_advertise fatal: {e:?}");
    }
    loop {
        FreeRtos::delay_ms(1000);
    }
}
