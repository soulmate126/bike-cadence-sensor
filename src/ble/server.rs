//! BLE CSC GATT Server 骨架
//!
//! 基于 esp-idf-svc `bt_gatt_server` 示例，发布标准 CSC 服务 (0x1816)。

use std::sync::{Arc, Mutex};

use enumset::enum_set;
use esp_idf_svc::bt::ble::gap::{AdvConfiguration, AppearanceCategory, BleGapEvent, EspBleGap};
use esp_idf_svc::bt::ble::gatt::server::{ConnectionId, EspGatts, GattsEvent, TransferId};
use esp_idf_svc::bt::ble::gatt::{
    AutoResponse, GattCharacteristic, GattDescriptor, GattId, GattInterface, GattResponse,
    GattServiceId, GattStatus, Handle, Permission, Property,
};
use esp_idf_svc::bt::{BdAddr, Ble, BtDriver, BtStatus, BtUuid};
use esp_idf_svc::sys::{EspError, ESP_FAIL};

use super::csc::{
    CadenceData, CSC_FEATURE_CRANK, CSC_FEATURE_UUID, CSC_MEASUREMENT_UUID, CSC_SERVICE_UUID,
    CCCD_UUID,
};

pub const DEVICE_NAME: &str = "DIY Cadence Sensor";
const APP_ID: u16 = 0;
const MAX_CONNECTIONS: usize = 2;
const SERVICE_NUM_HANDLES: u16 = 8;

type ExBtDriver = BtDriver<'static, Ble>;
type ExEspBleGap = Arc<EspBleGap<'static, Ble, Arc<ExBtDriver>>>;
type ExEspGatts = Arc<EspGatts<'static, Ble, Arc<ExBtDriver>>>;

#[derive(Debug, Clone)]
struct Connection {
    peer: BdAddr,
    conn_id: Handle,
    subscribed: bool,
}

#[derive(Default)]
struct State {
    gatt_if: Option<GattInterface>,
    service_handle: Option<Handle>,
    measurement_handle: Option<Handle>,
    feature_handle: Option<Handle>,
    cccd_handle: Option<Handle>,
    connections: heapless::Vec<Connection, MAX_CONNECTIONS>,
    response: GattResponse,
}

/// CSC GATT Server — 广播设备名 + 发布 0x1816 服务
#[derive(Clone)]
pub struct CscServer {
    gap: ExEspBleGap,
    gatts: ExEspGatts,
    state: Arc<Mutex<State>>,
}

impl CscServer {
    pub fn new(gap: ExEspBleGap, gatts: ExEspGatts) -> Arc<Self> {
        Arc::new(Self {
            gap,
            gatts,
            state: Arc::new(Mutex::new(State::default())),
        })
    }

    /// 注册 GAP/GATTS 回调并启动 GATT 应用
    pub fn begin(self: &Arc<Self>) -> Result<(), EspError> {
        let gap_server = Arc::clone(self);
        self.gap.subscribe(move |event| {
            if let Err(e) = gap_server.on_gap_event(event) {
                log::warn!("GAP event error: {e:?}");
            }
        })?;

        let gatts_server = Arc::clone(self);
        self.gatts.subscribe(move |(gatt_if, event)| {
            if let Err(e) = gatts_server.on_gatts_event(gatt_if, event) {
                log::warn!("GATTS event error: {e:?}");
            }
        })?;

        self.gatts.register_app(APP_ID)?;
        log::info!("CSC GATT server registering (app_id={APP_ID})");
        Ok(())
    }

    /// 向已订阅的客户端发送 CSC Measurement 通知
    pub fn notify_measurement(&self, data: &CadenceData) -> Result<(), EspError> {
        let payload = data.encode();
        let state = self.state.lock().unwrap();

        let Some(gatt_if) = state.gatt_if else {
            return Ok(());
        };
        let Some(measurement_handle) = state.measurement_handle else {
            return Ok(());
        };

        for conn in &state.connections {
            if conn.subscribed {
                self.gatts
                    .notify(gatt_if, conn.conn_id, measurement_handle, &payload)?;
                log::debug!(
                    "CSC notify → {}: revs={} time={}",
                    conn.peer,
                    data.crank_revolutions,
                    data.last_event_time
                );
            }
        }

        Ok(())
    }

    fn on_gap_event(&self, event: BleGapEvent) -> Result<(), EspError> {
        if let BleGapEvent::AdvertisingConfigured(status) = event {
            self.check_bt_status(status)?;
            self.gap.start_advertising()?;
            log::info!("BLE advertising started — scan for \"{DEVICE_NAME}\"");
        }
        Ok(())
    }

    fn on_gatts_event(
        &self,
        gatt_if: GattInterface,
        event: GattsEvent,
    ) -> Result<(), EspError> {
        match event {
            GattsEvent::ServiceRegistered { status, app_id } => {
                self.check_gatt_status(status)?;
                if app_id == APP_ID {
                    self.create_service(gatt_if)?;
                }
            }
            GattsEvent::ServiceCreated {
                status,
                service_handle,
                ..
            } => {
                self.check_gatt_status(status)?;
                self.start_service(service_handle)?;
            }
            GattsEvent::CharacteristicAdded {
                status,
                attr_handle,
                service_handle,
                char_uuid,
            } => {
                self.check_gatt_status(status)?;
                self.on_characteristic_added(service_handle, attr_handle, char_uuid)?;
            }
            GattsEvent::DescriptorAdded {
                status,
                attr_handle,
                service_handle,
                descr_uuid,
            } => {
                self.check_gatt_status(status)?;
                self.on_descriptor_added(service_handle, attr_handle, descr_uuid)?;
            }
            GattsEvent::PeerConnected { conn_id, addr, .. } => {
                self.on_peer_connected(conn_id, addr)?;
            }
            GattsEvent::PeerDisconnected { addr, .. } => {
                self.on_peer_disconnected(addr)?;
            }
            GattsEvent::Write {
                conn_id,
                trans_id,
                handle,
                offset,
                need_rsp,
                is_prep,
                value,
                ..
            } => {
                let handled = self.on_write(conn_id, handle, offset, value)?;
                if handled && need_rsp {
                    self.send_write_response(gatt_if, conn_id, trans_id, handle, offset, is_prep, value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn create_service(&self, gatt_if: GattInterface) -> Result<(), EspError> {
        self.state.lock().unwrap().gatt_if = Some(gatt_if);

        self.gap.set_device_name(DEVICE_NAME)?;
        self.gap.set_adv_conf(&AdvConfiguration {
            include_name: true,
            include_txpower: true,
            appearance: AppearanceCategory::Cycling,
            service_uuid: Some(BtUuid::uuid16(CSC_SERVICE_UUID)),
            ..Default::default()
        })?;

        self.gatts.create_service(
            gatt_if,
            &GattServiceId {
                id: GattId {
                    uuid: BtUuid::uuid16(CSC_SERVICE_UUID),
                    inst_id: 0,
                },
                is_primary: true,
            },
            SERVICE_NUM_HANDLES,
        )
    }

    fn start_service(&self, service_handle: Handle) -> Result<(), EspError> {
        self.state.lock().unwrap().service_handle = Some(service_handle);
        self.gatts.start_service(service_handle)?;
        self.add_characteristics(service_handle)
    }

    fn add_characteristics(&self, service_handle: Handle) -> Result<(), EspError> {
        // CSC Feature (0x2A5C) — 只读，告知客户端支持曲柄转数
        self.gatts.add_characteristic(
            service_handle,
            &GattCharacteristic {
                uuid: BtUuid::uuid16(CSC_FEATURE_UUID),
                permissions: enum_set!(Permission::Read),
                properties: enum_set!(Property::Read),
                max_len: CSC_FEATURE_CRANK.len(),
                auto_rsp: AutoResponse::ByGatt,
            },
            &[],
        )?;

        // CSC Measurement (0x2A5B) — Notify，发送累计转数 + 事件时间
        self.gatts.add_characteristic(
            service_handle,
            &GattCharacteristic {
                uuid: BtUuid::uuid16(CSC_MEASUREMENT_UUID),
                permissions: enum_set!(Permission::Read),
                properties: enum_set!(Property::Notify),
                max_len: 5,
                auto_rsp: AutoResponse::ByApp,
            },
            &[],
        )?;

        Ok(())
    }

    fn on_characteristic_added(
        &self,
        service_handle: Handle,
        attr_handle: Handle,
        char_uuid: BtUuid,
    ) -> Result<(), EspError> {
        let is_measurement = {
            let mut state = self.state.lock().unwrap();
            if state.service_handle != Some(service_handle) {
                return Ok(());
            }

            if char_uuid == BtUuid::uuid16(CSC_FEATURE_UUID) {
                state.feature_handle = Some(attr_handle);
                false
            } else if char_uuid == BtUuid::uuid16(CSC_MEASUREMENT_UUID) {
                state.measurement_handle = Some(attr_handle);
                true
            } else {
                false
            }
        };

        if !is_measurement {
            if let Some(feature_handle) = self.state.lock().unwrap().feature_handle {
                self.gatts
                    .set_attr(feature_handle, &CSC_FEATURE_CRANK)?;
                log::info!("CSC Feature characteristic ready");
            }
            return Ok(());
        }

        self.gatts.add_descriptor(
            service_handle,
            &GattDescriptor {
                uuid: BtUuid::uuid16(CCCD_UUID),
                permissions: enum_set!(Permission::Read | Permission::Write),
            },
        )?;

        Ok(())
    }

    fn on_descriptor_added(
        &self,
        service_handle: Handle,
        attr_handle: Handle,
        descr_uuid: BtUuid,
    ) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();
        if descr_uuid == BtUuid::uuid16(CCCD_UUID) && state.service_handle == Some(service_handle)
        {
            state.cccd_handle = Some(attr_handle);
            log::info!("CSC Measurement + CCCD ready — waiting for client subscribe");
        }
        Ok(())
    }

    fn on_peer_connected(&self, conn_id: ConnectionId, addr: BdAddr) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();
        if state.connections.len() < MAX_CONNECTIONS {
            state
                .connections
                .push(Connection {
                    peer: addr,
                    conn_id,
                    subscribed: false,
                })
                .map_err(|_| EspError::from_infallible::<ESP_FAIL>())?;
            log::info!("Client connected: {addr}");
        }
        drop(state);
        self.restart_advertising()
    }

    fn on_peer_disconnected(&self, addr: BdAddr) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();
        if let Some(index) = state
            .connections
            .iter()
            .position(|c| c.peer == addr)
        {
            state.connections.remove(index);
            log::info!("Client disconnected: {addr}");
        }
        drop(state);
        self.restart_advertising()
    }

    fn restart_advertising(&self) -> Result<(), EspError> {
        self.gap.set_adv_conf(&AdvConfiguration {
            include_name: true,
            include_txpower: true,
            appearance: AppearanceCategory::Cycling,
            service_uuid: Some(BtUuid::uuid16(CSC_SERVICE_UUID)),
            ..Default::default()
        })
    }

    fn on_write(
        &self,
        conn_id: ConnectionId,
        handle: Handle,
        offset: u16,
        value: &[u8],
    ) -> Result<bool, EspError> {
        let mut state = self.state.lock().unwrap();
        let cccd_handle = state.cccd_handle;

        let Some(conn) = state
            .connections
            .iter_mut()
            .find(|c| c.conn_id == conn_id)
        else {
            return Ok(false);
        };

        if Some(handle) != cccd_handle {
            return Ok(false);
        }

        if offset == 0 && value.len() == 2 {
            let flags = u16::from_le_bytes([value[0], value[1]]);
            if flags & 0x0001 != 0 {
                if !conn.subscribed {
                    conn.subscribed = true;
                    log::info!("Client {} subscribed to CSC notifications", conn.peer);
                }
            } else if conn.subscribed {
                conn.subscribed = false;
                log::info!("Client {} unsubscribed", conn.peer);
            }
        }

        Ok(true)
    }

    fn send_write_response(
        &self,
        gatt_if: GattInterface,
        conn_id: ConnectionId,
        trans_id: TransferId,
        handle: Handle,
        offset: u16,
        is_prep: bool,
        value: &[u8],
    ) -> Result<(), EspError> {
        if is_prep {
            let mut state = self.state.lock().unwrap();
            state
                .response
                .attr_handle(handle)
                .auth_req(0)
                .offset(offset)
                .value(value)
                .map_err(|_| EspError::from_infallible::<ESP_FAIL>())?;
            self.gatts.send_response(
                gatt_if,
                conn_id,
                trans_id,
                GattStatus::Ok,
                Some(&state.response),
            )
        } else {
            self.gatts
                .send_response(gatt_if, conn_id, trans_id, GattStatus::Ok, None)
        }
    }

    fn check_bt_status(&self, status: BtStatus) -> Result<(), EspError> {
        if !matches!(status, BtStatus::Success) {
            Err(EspError::from_infallible::<ESP_FAIL>())
        } else {
            Ok(())
        }
    }

    fn check_gatt_status(&self, status: GattStatus) -> Result<(), EspError> {
        if !matches!(status, GattStatus::Ok) {
            Err(EspError::from_infallible::<ESP_FAIL>())
        } else {
            Ok(())
        }
    }
}
