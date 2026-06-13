//! 板级可调参数（后续可扩展为 NVS 配置）

/// 主循环轮询间隔（毫秒）
pub const LOOP_DELAY_MS: u32 = 5;

/// 踏频去抖：两次有效脉冲最小间隔（毫秒）
pub const MIN_INTERVAL_MS: u64 = 200;

/// 无脉冲超过此时间则 RPM 归零（毫秒）
pub const STOP_TIMEOUT_MS: u64 = 3_000;

/// RPM 滑动平均窗口
pub const SAMPLE_WINDOW: usize = 5;

/// 至少 N 次间隔后才输出 RPM
pub const MIN_SAMPLES_FOR_RPM: usize = 3;

/// OLED UI 刷新间隔（毫秒）
pub const UI_REFRESH_MS: u64 = 200;

/// 状态 LED：未连接时慢闪周期（毫秒）
pub const LED_SLOW_BLINK_MS: u64 = 1000;

/// 状态 LED：有踏频时快闪周期（毫秒）
pub const LED_FAST_BLINK_MS: u64 = 200;

/// BLE Bonding（华为 GT5 对 DIY CSC 常不兼容，默认关闭）
pub const BLE_USE_BONDING: bool = false;

/// CSC flags 是否带接触位（华为部分固件只认 0x02）
pub const CSC_INCLUDE_CONTACT_FLAGS: bool = false;

/// 广播 Appearance：Cycling Cadence Sensor
pub const BLE_APPEARANCE_CADENCE: u16 = 0x0483;

/// 已连接但无订阅时的诊断日志间隔（毫秒）
pub const BLE_DIAG_INTERVAL_MS: u64 = 5_000;

/// 连接间隔（单位 1.25ms）：32≈40ms，64≈80ms
pub const BLE_CONN_INTERVAL_MIN: u16 = 32;
pub const BLE_CONN_INTERVAL_MAX: u16 = 64;
/// 可跳过的连接事件数（降低射频占用、提升稳定性）
pub const BLE_CONN_LATENCY: u16 = 4;
/// 监督超时（单位 10ms）：400 = 4s（原 60=600ms 过短，易断连）
pub const BLE_CONN_SUPERVISION_TIMEOUT: u16 = 400;
