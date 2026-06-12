//! 示例 5：BLE CSC GATT Server
//!
//! 发布标准 Cycling Speed and Cadence 服务 (0x1816)，
//! 模拟 80 RPM 踏频数据并通过 CSC Measurement (0x2A5B) 通知客户端。
//!
//! 验证：
//! - nRF Connect 扫描 "DIY Cadence Sensor"，连接后订阅 0x2A5B
//! - 华为 GT5 Pro：设置 → 健康与健身设备 → 添加设备 → 踏频器

use std::sync::Arc;

use bike_cadence_sensor::ble::csc::{ms_to_csc_time, CadenceData};
use bike_cadence_sensor::ble::server::CscServer;
use bike_cadence_sensor::init;
use esp_idf_svc::bt::ble::gatt::server::EspGatts;
use esp_idf_svc::bt::ble::gap::EspBleGap;
use esp_idf_svc::bt::{Ble, BtDriver};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

/// 模拟 80 RPM → 每 750ms 一次脉冲
const SIM_INTERVAL_MS: u64 = 750;

fn main() -> anyhow::Result<()> {
    init();

    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let bt = Arc::new(BtDriver::new(peripherals.modem, Some(nvs))?);
    let gap = Arc::new(EspBleGap::new(bt.clone())?);
    let gatts = Arc::new(EspGatts::new(bt.clone())?);

    let server = CscServer::new(gap, gatts);
    server.begin()?;

    log::info!("05_ble_csc: simulating {SIM_INTERVAL_MS}ms interval (~80 RPM)");

    let mut revolutions = 0u16;
    let mut elapsed_ms = 0u64;

    loop {
        FreeRtos::delay_ms(SIM_INTERVAL_MS as u32);

        revolutions = revolutions.wrapping_add(1);
        elapsed_ms += SIM_INTERVAL_MS;

        let data = CadenceData::new(revolutions, ms_to_csc_time(elapsed_ms));
        server.notify_measurement(&data)?;

        log::info!(
            "CSC notify: revs={revolutions} time={} (~80 RPM)",
            data.last_event_time
        );
    }
}
