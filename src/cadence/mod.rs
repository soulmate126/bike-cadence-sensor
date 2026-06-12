//! 踏频 RPM 计算（占位）

pub struct CadenceCalculator {
    pub rpm: f32,
}

impl CadenceCalculator {
    pub fn new() -> Self {
        Self { rpm: 0.0 }
    }

    pub fn on_pulse(&mut self, _interval_ms: u32) {
        // TODO: 滑动窗口 / 指数平滑
    }
}
