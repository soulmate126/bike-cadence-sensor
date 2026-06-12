//! 示例 4：BLE 广播设备名 "DIY Cadence Sensor"

use bike_cadence_sensor::{board, init};
use esp_idf_svc::bt::ble::gap::{AdvConfiguration, BleGapEvent, EspBleGap};
use esp_idf_svc::bt::{Ble, BtDriver, BtStatus};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use std::sync::Arc;

const DEVICE_NAME: &str = "DIY Cadence Sensor";

fn main() -> anyhow::Result<()> {
    init();

    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let bt: Arc<BtDriver<'static, Ble>> =
        Arc::new(BtDriver::new(peripherals.modem, Some(nvs))?);
    let gap = EspBleGap::new(bt)?;

    gap.set_device_name(DEVICE_NAME)?;
    log::info!("04_ble_advertise: device name set to \"{DEVICE_NAME}\"");

    gap.subscribe(move |event| {
        if let BleGapEvent::AdvertisingConfigured(status) = event {
            if status == BtStatus::Success {
                log::info!("BLE advertising started — scan with nRF Connect / 华为运动健康");
            }
        }
    })?;

    gap.set_adv_conf(&AdvConfiguration {
        include_name: true,
        include_txpower: true,
        ..Default::default()
    })?;

    loop {
        FreeRtos::delay_ms(1000);
    }
}
