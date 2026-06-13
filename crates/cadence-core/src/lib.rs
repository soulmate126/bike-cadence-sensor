//! 踏频 RPM 计算（纯逻辑，无硬件依赖，可在主机上单元测试）

/// 算法可调参数
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CadenceConfig {
    /// 两次有效脉冲之间的最小间隔（去抖）
    pub min_interval_ms: u64,
    /// 超过此时间无新脉冲则判定为停止骑行
    pub stop_timeout_ms: u64,
    /// 滑动平均窗口大小
    pub sample_window: usize,
    /// 至少积累这么多次间隔后才输出 RPM
    pub min_samples_for_rpm: usize,
}

impl Default for CadenceConfig {
    fn default() -> Self {
        Self {
            min_interval_ms: 200,
            stop_timeout_ms: 3_000,
            sample_window: 5,
            min_samples_for_rpm: 3,
        }
    }
}

/// 由两次脉冲间隔计算瞬时 RPM
pub fn calculate_rpm(delta_ms: u64) -> f32 {
    if delta_ms == 0 {
        return 0.0;
    }
    60_000.0 / delta_ms as f32
}

/// 毫秒 → CSC 事件时间（1/1024 秒，u16 自然回绕）
pub fn ms_to_csc_time(ms: u64) -> u16 {
    ((ms * 1024) / 1000) as u16
}

/// 霍尔 → 算法 / 显示 / BLE 的共享快照
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CadenceSnapshot {
    pub rpm: f32,
    /// 累计曲柄转数（u16 循环，与 CSC 0x2A5B 一致）
    pub cumulative_revolutions: u16,
    /// 最后一次有效脉冲的 CSC 事件时间（1/1024 秒，u16 循环）
    pub last_event_time: u16,
    /// 磁铁是否贴近霍尔（KY-003：S 脚为低表示有接触）
    pub sensor_contact: bool,
}

pub struct CadenceCalculator {
    config: CadenceConfig,
    rpm: f32,
    cumulative_revolutions: u16,
    last_event_time: u16,
    last_pulse_ms: Option<u64>,
    intervals: [u64; 8],
    interval_count: usize,
}

impl CadenceCalculator {
    pub fn new() -> Self {
        Self::with_config(CadenceConfig::default())
    }

    pub fn with_config(config: CadenceConfig) -> Self {
        Self {
            config,
            rpm: 0.0,
            cumulative_revolutions: 0,
            last_event_time: 0,
            last_pulse_ms: None,
            intervals: [0; 8],
            interval_count: 0,
        }
    }

    pub fn rpm(&self) -> f32 {
        self.rpm
    }

    pub fn cumulative_revolutions(&self) -> u16 {
        self.cumulative_revolutions
    }

    pub fn snapshot(&self, sensor_contact: bool) -> CadenceSnapshot {
        CadenceSnapshot {
            rpm: self.rpm,
            cumulative_revolutions: self.cumulative_revolutions,
            last_event_time: self.last_event_time,
            sensor_contact,
        }
    }

    /// 霍尔下降沿时调用。返回 `true` 表示计入了 1 次有效转数（可发 BLE 通知）。
    pub fn on_pulse(&mut self, timestamp_ms: u64) -> bool {
        let Some(prev) = self.last_pulse_ms else {
            self.last_pulse_ms = Some(timestamp_ms);
            self.last_event_time = ms_to_csc_time(timestamp_ms);
            return false;
        };

        let delta = timestamp_ms.saturating_sub(prev);
        if delta < self.config.min_interval_ms {
            return false;
        }

        self.last_pulse_ms = Some(timestamp_ms);
        self.last_event_time = ms_to_csc_time(timestamp_ms);
        self.cumulative_revolutions = self.cumulative_revolutions.wrapping_add(1);
        self.push_interval(delta);
        self.rpm = self.average_rpm();
        true
    }

    /// 主循环定期调用；超过 `stop_timeout_ms` 无脉冲则将 RPM 归零。
    pub fn update(&mut self, now_ms: u64) {
        let Some(last) = self.last_pulse_ms else {
            return;
        };
        if now_ms.saturating_sub(last) >= self.config.stop_timeout_ms {
            self.rpm = 0.0;
            self.interval_count = 0;
        }
    }

    fn push_interval(&mut self, delta_ms: u64) {
        let window = self.config.sample_window.min(self.intervals.len());
        if self.interval_count < window {
            self.intervals[self.interval_count] = delta_ms;
            self.interval_count += 1;
        } else if window > 0 {
            self.intervals.copy_within(1..window, 0);
            self.intervals[window - 1] = delta_ms;
        }
    }

    fn average_rpm(&self) -> f32 {
        if self.interval_count < self.config.min_samples_for_rpm {
            return 0.0;
        }
        let n = self.interval_count;
        let sum: u64 = self.intervals[..n].iter().sum();
        let avg_delta = sum / n as u64;
        calculate_rpm(avg_delta)
    }
}

impl Default for CadenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CFG: CadenceConfig = CadenceConfig {
        min_interval_ms: 200,
        stop_timeout_ms: 3_000,
        sample_window: 5,
        min_samples_for_rpm: 3,
    };

    #[test]
    fn calculate_rpm_reference_values() {
        assert!((calculate_rpm(1000) - 60.0).abs() < f32::EPSILON);
        assert!((calculate_rpm(750) - 80.0).abs() < f32::EPSILON);
        assert!((calculate_rpm(600) - 100.0).abs() < f32::EPSILON);
        assert!((calculate_rpm(500) - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ms_to_csc_time_one_second() {
        assert_eq!(ms_to_csc_time(1000), 1024);
    }

    #[test]
    fn ignores_short_intervals() {
        let mut calc = CadenceCalculator::with_config(TEST_CFG);
        assert!(!calc.on_pulse(0));
        assert!(!calc.on_pulse(100));
        assert_eq!(calc.cumulative_revolutions(), 0);
        assert_eq!(calc.rpm(), 0.0);
    }

    #[test]
    fn rpm_zero_until_three_intervals() {
        let mut calc = CadenceCalculator::with_config(TEST_CFG);
        calc.on_pulse(0);
        calc.on_pulse(750);
        assert_eq!(calc.rpm(), 0.0);
        calc.on_pulse(1500);
        assert_eq!(calc.rpm(), 0.0);
        calc.on_pulse(2250);
        assert!((calc.rpm() - 80.0).abs() < 0.1);
    }

    #[test]
    fn stop_detection() {
        let mut calc = CadenceCalculator::with_config(TEST_CFG);
        calc.on_pulse(0);
        for i in 1..=4 {
            calc.on_pulse(i * 750);
        }
        assert!((calc.rpm() - 80.0).abs() < 0.1);

        calc.update(4 * 750 + TEST_CFG.stop_timeout_ms);
        assert_eq!(calc.rpm(), 0.0);
        assert_eq!(calc.cumulative_revolutions(), 4);
    }
}
