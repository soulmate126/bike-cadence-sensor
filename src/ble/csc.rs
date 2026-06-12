//! BLE Cycling Speed and Cadence (CSC) 服务
//!
//! Service UUID: 0x1816
//! Measurement Characteristic UUID: 0x2A5B

use crate::cadence::CadenceSnapshot;

/// CSC Service UUID
pub const CSC_SERVICE_UUID: u16 = 0x1816;

/// CSC Feature Characteristic UUID
pub const CSC_FEATURE_UUID: u16 = 0x2A5C;

/// CCCD Descriptor UUID
pub const CCCD_UUID: u16 = 0x2902;

/// CSC Feature 值：支持曲柄转数（bit 1）
pub const CSC_FEATURE_CRANK: [u8; 2] = [0x02, 0x00];

/// CSC Measurement Characteristic UUID
pub const CSC_MEASUREMENT_UUID: u16 = 0x2A5B;

/// CSC Measurement flags — bit 1: Crank Revolution Data Present
pub const FLAG_CRANK_REVOLUTION_DATA: u8 = 0x02;

/// 映射到 CSC Measurement Characteristic 的数据
///
/// 注意：CSC 协议发送的是**累计曲柄转数 + 事件时间**，不是 RPM。
/// 接收端（手表 / 手机）自行计算 RPM。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CadenceData {
    pub crank_revolutions: u16,
    pub last_event_time: u16,
}

impl CadenceData {
    pub fn new(crank_revolutions: u16, last_event_time: u16) -> Self {
        Self {
            crank_revolutions,
            last_event_time,
        }
    }

    /// 从算法快照转换，时间戳转为 CSC 1/1024 秒单位
    pub fn from_snapshot(snapshot: &CadenceSnapshot) -> Self {
        Self {
            crank_revolutions: snapshot.cumulative_revolutions as u16,
            last_event_time: ms_to_csc_time(snapshot.last_event_time_ms),
        }
    }

    /// 编码为 CSC Measurement 字节流（仅踏频，5 字节）
    pub fn encode(&self) -> [u8; 5] {
        let mut buf = [0u8; 5];
        buf[0] = FLAG_CRANK_REVOLUTION_DATA;
        buf[1..3].copy_from_slice(&self.crank_revolutions.to_le_bytes());
        buf[3..5].copy_from_slice(&self.last_event_time.to_le_bytes());
        buf
    }
}

/// 毫秒 → CSC 事件时间（1/1024 秒，u16 回绕）
pub fn ms_to_csc_time(ms: u64) -> u16 {
    ((ms * 1024) / 1000) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cadence::CadenceSnapshot;

    #[test]
    fn ms_to_csc_time_one_second() {
        assert_eq!(ms_to_csc_time(1000), 1024);
    }

    #[test]
    fn encode_cadence_only() {
        let data = CadenceData::new(42, 1024);
        let bytes = data.encode();
        assert_eq!(bytes, [0x02, 42, 0, 0, 4]); // 1024 LE = 0x0400
    }

    #[test]
    fn from_snapshot() {
        let snapshot = CadenceSnapshot {
            rpm: 80.0,
            cumulative_revolutions: 100,
            last_event_time_ms: 1000,
        };
        let data = CadenceData::from_snapshot(&snapshot);
        assert_eq!(data.crank_revolutions, 100);
        assert_eq!(data.last_event_time, 1024);
    }
}
