//! SSD1306 等外设共用的 I2C 工具

use core::time::Duration;

use esp_idf_hal::delay::TickType;
use esp_idf_hal::i2c::{APBTickType, I2cConfig, I2cDriver};
use esp_idf_hal::units::Hertz;

use crate::board;

const I2C_PROBE_TIMEOUT: u32 = TickType::new_millis(20).ticks();

pub fn default_i2c_config() -> I2cConfig {
    I2cConfig::new()
        .baudrate(Hertz(100_000))
        .sda_enable_pullup(true)
        .scl_enable_pullup(true)
        .timeout(APBTickType::from(Duration::from_millis(100)))
}

/// SSD1306 I2C 首字节为控制位（0x00=命令）。空写部分模块不 ACK。
pub fn probe(i2c: &mut I2cDriver<'_>, addr: u8) -> bool {
    i2c.write(addr, &[0x00], I2C_PROBE_TIMEOUT).is_ok()
}

pub fn quick_scan(i2c: &mut I2cDriver<'_>) {
    log::info!(
        "I2C scan (SDA=GPIO{}, SCL=GPIO{})...",
        board::I2C_SDA_GPIO,
        board::I2C_SCL_GPIO
    );
    let mut found = false;
    for addr in 0x08..0x78 {
        if probe(i2c, addr) {
            log::info!("  device at 0x{addr:02X}");
            found = true;
        }
    }
    if !found {
        log::warn!("  bus empty — check wiring / try swapping SDA-SCL");
    }
}

pub fn resolve_oled_addr(i2c: &mut I2cDriver<'_>) -> Option<u8> {
    [board::OLED_I2C_ADDR, 0x3D]
        .into_iter()
        .find(|&addr| probe(i2c, addr))
}
