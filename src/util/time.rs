use esp_idf_hal::sys::esp_timer_get_time;

/// 单调时钟（毫秒，自启动起）
pub fn now_ms() -> u64 {
    (unsafe { esp_timer_get_time() } / 1000) as u64
}
