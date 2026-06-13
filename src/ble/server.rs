//! BLE CSC GATT Server（NimBLE / esp32-nimble）

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    utilities::BleUuid, BLEAdvertisementData, BLECharacteristic, BLEDevice, BLEError, NimbleProperties,
    NimbleSub,
};
use esp32_nimble::utilities::mutex::Mutex as NimbleMutex;

use crate::board::config::{
    BLE_APPEARANCE_CADENCE, BLE_CONN_INTERVAL_MAX, BLE_CONN_INTERVAL_MIN, BLE_CONN_LATENCY,
    BLE_CONN_SUPERVISION_TIMEOUT, BLE_USE_BONDING,
};

use super::csc::{
    CadenceData, CSC_FEATURE_CRANK, CSC_FEATURE_UUID, CSC_MEASUREMENT_UUID,
    CSC_SENSOR_LOCATION_UUID, CSC_SERVICE_UUID, FLAG_CRANK_REVOLUTION_DATA,
    SENSOR_LOCATION_LEFT_CRANK,
};

pub const DEVICE_NAME: &str = "DIY Cadence Sensor";

const PREFERRED_MTU: u16 = 247;

/// CSC GATT Server — 广播设备名 + 发布 0x1816 服务
pub struct CscServer {
    measurement: Arc<NimbleMutex<BLECharacteristic>>,
    last_data: Arc<NimbleMutex<CadenceData>>,
    pub connected: Arc<AtomicBool>,
    disconnect_count: Arc<AtomicU32>,
}

impl CscServer {
    pub fn begin() -> Result<Arc<Self>, BLEError> {
        let ble_device = BLEDevice::take();
        let ble_advertising = ble_device.get_advertising();
        let server = ble_device.get_server();

        if BLE_USE_BONDING {
            ble_device
                .security()
                .set_auth(AuthReq::Bond)
                .set_io_cap(SecurityIOCap::NoInputNoOutput)
                .resolve_rpa();
            log::info!("CSC: bonding enabled");
        } else {
            // 清除旧绑定，避免华为因残留 bond 信息连接异常
            if let Err(e) = ble_device.delete_all_bonds() {
                log::warn!("CSC: clear bonds: {e:?}");
            } else {
                log::info!("CSC: bonding disabled, cleared stored bonds");
            }
        }

        if let Err(e) = ble_device.set_preferred_mtu(PREFERRED_MTU) {
            log::warn!("CSC: preferred MTU {PREFERRED_MTU}: {e:?}");
        }

        let connected = Arc::new(AtomicBool::new(false));
        let disconnect_count = Arc::new(AtomicU32::new(0));
        let last_data = Arc::new(NimbleMutex::new(CadenceData::new(0, 0, false)));

        let connected_cb = Arc::clone(&connected);
        let disconnect_count_cb = Arc::clone(&disconnect_count);

        server.on_connect(move |server, desc| {
            connected_cb.store(true, Ordering::Release);
            log::info!(
                "CSC: CONNECT {:?} mtu={} bonded={}",
                desc.address(),
                desc.mtu(),
                desc.bonded()
            );
            if let Err(e) = server.update_conn_params(
                desc.conn_handle(),
                BLE_CONN_INTERVAL_MIN,
                BLE_CONN_INTERVAL_MAX,
                BLE_CONN_LATENCY,
                BLE_CONN_SUPERVISION_TIMEOUT,
            ) {
                log::warn!("CSC: conn params update: {e:?}");
            } else {
                log::info!(
                    "CSC: conn params → interval {}-{} latency={} timeout={}ms",
                    BLE_CONN_INTERVAL_MIN * 125 / 100,
                    BLE_CONN_INTERVAL_MAX * 125 / 100,
                    BLE_CONN_LATENCY,
                    BLE_CONN_SUPERVISION_TIMEOUT * 10
                );
            }
            // 单连接踏频器：连接后不再重启广播，避免华为频繁断连
        });

        let connected_disc = Arc::clone(&connected);
        server.on_disconnect(move |desc, reason| {
            connected_disc.store(false, Ordering::Release);
            let n = disconnect_count_cb.fetch_add(1, Ordering::Relaxed) + 1;
            log::info!(
                "CSC: DISCONNECT #{n} {:?} bonded={} ({reason:?})",
                desc.address(),
                desc.bonded()
            );
        });

        if BLE_USE_BONDING {
            server.on_authentication_complete(|_, desc, result| {
                log::info!("CSC: bonding {:?} result={result:?}", desc.address());
            });
        }

        ble_advertising.lock().on_complete(|_| {
            if let Err(e) = ble_advertising.lock().start() {
                log::warn!("CSC: advertising on_complete restart: {e:?}");
            }
        });

        let service = server.create_service(BleUuid::from_uuid16(CSC_SERVICE_UUID));

        // SIG 常见顺序：Measurement → Feature → Sensor Location
        let measurement = service.lock().create_characteristic(
            BleUuid::from_uuid16(CSC_MEASUREMENT_UUID),
            NimbleProperties::READ | NimbleProperties::NOTIFY,
        );
        measurement
            .lock()
            .set_value(&[FLAG_CRANK_REVOLUTION_DATA, 0, 0, 0, 0]);

        let feature = service.lock().create_characteristic(
            BleUuid::from_uuid16(CSC_FEATURE_UUID),
            NimbleProperties::READ,
        );
        feature.lock().set_value(&CSC_FEATURE_CRANK);

        let location = service.lock().create_characteristic(
            BleUuid::from_uuid16(CSC_SENSOR_LOCATION_UUID),
            NimbleProperties::READ,
        );
        location
            .lock()
            .set_value(&[SENSOR_LOCATION_LEFT_CRANK]);

        let measurement_notify = Arc::clone(&measurement);
        let last_data_notify = Arc::clone(&last_data);
        measurement.lock().on_subscribe(move |_chr, desc, sub| {
            if sub.contains(NimbleSub::NOTIFY) {
                log::info!(
                    "CSC: SUBSCRIBED {:?} 0x2A5B mtu={} (watch must show this during workout)",
                    desc.address(),
                    desc.mtu()
                );
                let data = *last_data_notify.lock();
                let payload = data.encode();
                measurement_notify.lock().set_value(&payload).notify();
                log::info!(
                    "CSC: initial notify revs={} time={} flags=0x{:02X}",
                    data.crank_revolutions,
                    data.last_event_time,
                    payload[0]
                );
            } else if sub.is_empty() {
                log::warn!("CSC: UNSUBSCRIBED {:?}", desc.address());
            }
        });

        let mut adv_data = BLEAdvertisementData::new();
        adv_data
            .name(DEVICE_NAME)
            .appearance(BLE_APPEARANCE_CADENCE)
            .add_service_uuid(BleUuid::from_uuid16(CSC_SERVICE_UUID));
        ble_advertising.lock().set_data(&mut adv_data)?;
        ble_advertising.lock().start()?;

        log::info!(
            "CSC server — \"{DEVICE_NAME}\" 0x{CSC_SERVICE_UUID:04X} bonding={BLE_USE_BONDING}"
        );

        Ok(Arc::new(Self {
            measurement,
            last_data,
            connected,
            disconnect_count,
        }))
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    /// 已开启 0x2A5B Notify 的客户端数量（运动中有踏频应为 ≥1）
    pub fn subscriber_count(&self) -> usize {
        self.measurement.lock().subscribed_count()
    }

    pub fn disconnect_count(&self) -> u32 {
        self.disconnect_count.load(Ordering::Relaxed)
    }

    pub fn notify_measurement(&self, data: &CadenceData) {
        *self.last_data.lock() = *data;
        let payload = data.encode();
        self.measurement.lock().set_value(&payload).notify();
    }
}
