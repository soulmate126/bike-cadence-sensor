//! 踏频 RPM 计算（纯逻辑，无硬件依赖）

/// 两次有效脉冲之间的最小间隔（去抖 / 过滤异常触发）
pub const MIN_INTERVAL_MS: u64 = 200;

/// 超过此时间无新脉冲则判定为停止骑行
pub const STOP_TIMEOUT_MS: u64 = 3_000;

/// 滑动平均窗口大小
pub const SAMPLE_WINDOW: usize = 5;

/// 由两次脉冲间隔计算瞬时 RPM
pub fn calculate_rpm(delta_ms: u64) -> f32 {
    if delta_ms == 0 {
        return 0.0;
    }
    60_000.0 / delta_ms as f32
}

/// 霍尔 → 算法 / 显示 / BLE 的共享快照
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CadenceSnapshot {
    pub rpm: f32,
    pub cumulative_revolutions: u32,
    pub last_event_time_ms: u64,
}

pub struct CadenceCalculator {
    rpm: f32,
    cumulative_revolutions: u32,
    last_pulse_ms: Option<u64>,
    intervals: [u64; SAMPLE_WINDOW],
    interval_count: usize,
}

impl CadenceCalculator {
    pub fn new() -> Self {
        Self {
            rpm: 0.0,
            cumulative_revolutions: 0,
            last_pulse_ms: None,
            intervals: [0; SAMPLE_WINDOW],
            interval_count: 0,
        }
    }

    pub fn rpm(&self) -> f32 {
        self.rpm
    }

    pub fn cumulative_revolutions(&self) -> u32 {
        self.cumulative_revolutions
    }

    pub fn snapshot(&self) -> CadenceSnapshot {
        CadenceSnapshot {
            rpm: self.rpm,
            cumulative_revolutions: self.cumulative_revolutions,
            last_event_time_ms: self.last_pulse_ms.unwrap_or(0),
        }
    }

    /// 霍尔触发时调用，传入当前时间戳（毫秒）
    pub fn on_pulse(&mut self, timestamp_ms: u64) {
        let Some(prev) = self.last_pulse_ms else {
            self.last_pulse_ms = Some(timestamp_ms);
            return;
        };

        let delta = timestamp_ms.saturating_sub(prev);
        if delta < MIN_INTERVAL_MS {
            return;
        }

        self.last_pulse_ms = Some(timestamp_ms);
        self.cumulative_revolutions = self.cumulative_revolutions.saturating_add(1);
        self.push_interval(delta);
        self.rpm = self.average_rpm();
    }

    /// 主循环定期调用，检测停止骑行
    pub fn update(&mut self, now_ms: u64) {
        let Some(last) = self.last_pulse_ms else {
            return;
        };
        if now_ms.saturating_sub(last) >= STOP_TIMEOUT_MS {
            self.rpm = 0.0;
            self.interval_count = 0;
        }
    }

    fn push_interval(&mut self, delta_ms: u64) {
        if self.interval_count < SAMPLE_WINDOW {
            self.intervals[self.interval_count] = delta_ms;
            self.interval_count += 1;
        } else {
            self.intervals.copy_within(1.., 0);
            self.intervals[SAMPLE_WINDOW - 1] = delta_ms;
        }
    }

    fn average_rpm(&self) -> f32 {
        if self.interval_count == 0 {
            return 0.0;
        }
        let sum: u64 = self.intervals[..self.interval_count].iter().sum();
        let avg_delta = sum / self.interval_count as u64;
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

    #[test]
    fn calculate_rpm_reference_values() {
        assert!((calculate_rpm(1000) - 60.0).abs() < f32::EPSILON);
        assert!((calculate_rpm(750) - 80.0).abs() < f32::EPSILON);
        assert!((calculate_rpm(600) - 100.0).abs() < f32::EPSILON);
        assert!((calculate_rpm(500) - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ignores_short_intervals() {
        let mut calc = CadenceCalculator::new();
        calc.on_pulse(0);
        calc.on_pulse(100); // < 200ms，忽略
        assert_eq!(calc.cumulative_revolutions(), 0);
        assert_eq!(calc.rpm(), 0.0);
    }

    #[test]
    fn five_sample_moving_average() {
        let mut calc = CadenceCalculator::new();
        calc.on_pulse(0);
        for i in 1..=5 {
            calc.on_pulse(i * 750); // 每次间隔 750ms → 80 RPM
        }
        assert_eq!(calc.cumulative_revolutions(), 5);
        assert!((calc.rpm() - 80.0).abs() < 0.1);
    }

    #[test]
    fn stop_detection() {
        let mut calc = CadenceCalculator::new();
        calc.on_pulse(0);
        calc.on_pulse(750);
        assert!(calc.rpm() > 0.0);

        calc.update(750 + STOP_TIMEOUT_MS);
        assert_eq!(calc.rpm(), 0.0);
        // 累计转数保留，供 BLE CSC 使用
        assert_eq!(calc.cumulative_revolutions(), 1);
    }
}
