//! 踏频 RPM 计算（封装 cadence-core，注入板级配置）

pub use cadence_core::{
    calculate_rpm, ms_to_csc_time, CadenceCalculator, CadenceConfig, CadenceSnapshot,
};

use crate::board::pins::cadence_config;

pub fn new_calculator() -> CadenceCalculator {
    CadenceCalculator::with_config(cadence_config())
}
