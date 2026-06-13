//! 板载 LED：BLE 连接 / 踏频状态指示

use esp_idf_hal::gpio::{Output, PinDriver};

use super::config::{LED_FAST_BLINK_MS, LED_SLOW_BLINK_MS};

/// 未连接慢闪、已连接常亮、有踏频快闪
pub struct StatusLed<'d> {
    pin: PinDriver<'d, Output>,
    ble_connected: bool,
    active_cadence: bool,
    led_on: bool,
    last_toggle_ms: u64,
}

impl<'d> StatusLed<'d> {
    pub fn new(mut pin: PinDriver<'d, Output>) -> Self {
        let _ = pin.set_high();
        Self {
            pin,
            ble_connected: false,
            active_cadence: false,
            led_on: false,
            last_toggle_ms: 0,
        }
    }

    pub fn set_ble_connected(&mut self, connected: bool) {
        self.ble_connected = connected;
        if connected && !self.active_cadence {
            let _ = self.pin.set_low();
            self.led_on = true;
        } else if !connected {
            let _ = self.pin.set_high();
            self.led_on = false;
            self.last_toggle_ms = 0;
        }
    }

    pub fn tick(&mut self, rpm: f32, now_ms: u64) {
        self.active_cadence = self.ble_connected && rpm > 0.5;

        if !self.ble_connected {
            let period = LED_SLOW_BLINK_MS;
            if now_ms.saturating_sub(self.last_toggle_ms) >= period {
                self.led_on = !self.led_on;
                let _ = if self.led_on {
                    self.pin.set_low()
                } else {
                    self.pin.set_high()
                };
                self.last_toggle_ms = now_ms;
            }
            return;
        }

        if self.active_cadence {
            let period = LED_FAST_BLINK_MS;
            if now_ms.saturating_sub(self.last_toggle_ms) >= period {
                self.led_on = !self.led_on;
                let _ = if self.led_on {
                    self.pin.set_low()
                } else {
                    self.pin.set_high()
                };
                self.last_toggle_ms = now_ms;
            }
        } else {
            let _ = self.pin.set_low();
            self.led_on = true;
        }
    }
}
