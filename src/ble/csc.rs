//! BLE Cycling Speed and Cadence (CSC) 服务
//!
//! Service UUID: 0x1816
//! Measurement Characteristic UUID: 0x2A5B

use crate::cadence::CadenceSnapshot;

/// CSC Service UUID
pub const CSC_SERVICE_UUID: u16 = 0x1816;

/// CSC Feature Characteristic UUID
pub const CSC_FEATURE_UUID: u16 = 0x2A5C;

/// CSC Sensor Location Characteristic UUID
pub const CSC_SENSOR_LOCATION_UUID: u16 = 0x2A5D;

/// CCCD Descriptor UUID
pub const CCCD_UUID: u16 = 0x2902;

/// CSC Feature 值：支持曲柄转数（bit 1）
pub const CSC_FEATURE_CRANK: [u8; 2] = [0x02, 0x00];

/// CSC Measurement Characteristic UUID
pub const CSC_MEASUREMENT_UUID: u16 = 0x2A5B;

/// bit 1: Crank Revolution Data Present
pub const FLAG_CRANK_REVOLUTION_DATA: u8 = 0x02;
/// bit 2: Sensor Contact Status Supported
pub const FLAG_SENSOR_CONTACT_SUPPORTED: u8 = 0x04;
/// bit 3: Sensor Contact Status (磁铁贴近时为 1)
pub const FLAG_SENSOR_CONTACT_DETECTED: u8 = 0x08;

/// Bluetooth SIG：Left Crank = 5
pub const SENSOR_LOCATION_LEFT_CRANK: u8 = 5;

/// 映射到 CSC Measurement Characteristic 的数据
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CadenceData {
    pub crank_revolutions: u16,
    pub last_event_time: u16,
    pub sensor_contact: bool,
}

impl CadenceData {
    pub fn new(crank_revolutions: u16, last_event_time: u16, sensor_contact: bool) -> Self {
        Self {
            crank_revolutions,
            last_event_time,
            sensor_contact,
        }
    }

    pub fn from_snapshot(snapshot: &CadenceSnapshot) -> Self {
        Self {
            crank_revolutions: snapshot.cumulative_revolutions,
            last_event_time: snapshot.last_event_time,
            sensor_contact: snapshot.sensor_contact,
        }
    }

    pub fn encode(&self) -> [u8; 5] {
        let flags = if crate::board::config::CSC_INCLUDE_CONTACT_FLAGS {
            let mut f = FLAG_CRANK_REVOLUTION_DATA | FLAG_SENSOR_CONTACT_SUPPORTED;
            if self.sensor_contact {
                f |= FLAG_SENSOR_CONTACT_DETECTED;
            }
            f
        } else {
            FLAG_CRANK_REVOLUTION_DATA
        };
        let mut buf = [0u8; 5];
        buf[0] = flags;
        buf[1..3].copy_from_slice(&self.crank_revolutions.to_le_bytes());
        buf[3..5].copy_from_slice(&self.last_event_time.to_le_bytes());
        buf
    }
}

pub use crate::cadence::ms_to_csc_time;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cadence::CadenceSnapshot;

    #[test]
    fn encode_with_contact() {
        let data = CadenceData::new(42, 1024, true);
        let bytes = data.encode();
        // CSC_INCLUDE_CONTACT_FLAGS=false 时恒为 0x02（华为兼容）
        assert_eq!(bytes[0], 0x02);
        assert_eq!(bytes[1..3], [42, 0]);
    }

    #[test]
    fn encode_without_contact() {
        let data = CadenceData::new(42, 1024, false);
        assert_eq!(data.encode()[0], 0x02);
    }

    #[test]
    fn from_snapshot() {
        let snapshot = CadenceSnapshot {
            rpm: 80.0,
            cumulative_revolutions: 100,
            last_event_time: 1024,
            sensor_contact: true,
        };
        let data = CadenceData::from_snapshot(&snapshot);
        assert_eq!(data.crank_revolutions, 100);
        assert_eq!(data.sensor_contact, true);
    }
}
