//! 从 `Peripherals` 按板级引脚常量打开外设

use esp_idf_hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::i2c::I2C0;

use super::config;
use crate::hardware::i2c::default_i2c_config;

pub fn hall_pin<'d>(
    gpio: esp_idf_hal::gpio::Gpio4<'d>,
) -> anyhow::Result<PinDriver<'d, Input>> {
    Ok(PinDriver::input(gpio, Pull::Floating)?)
}

pub fn status_led_pin<'d>(
    gpio: esp_idf_hal::gpio::Gpio8<'d>,
) -> anyhow::Result<PinDriver<'d, Output>> {
    let mut led = PinDriver::output(gpio)?;
    // SuperMini 板载 LED 低电平点亮
    led.set_high()?;
    Ok(led)
}

pub fn oled_i2c<'d>(
    i2c0: I2C0<'d>,
    sda: esp_idf_hal::gpio::Gpio5<'d>,
    scl: esp_idf_hal::gpio::Gpio6<'d>,
) -> anyhow::Result<I2cDriver<'d>> {
    Ok(I2cDriver::new(i2c0, sda, scl, &default_i2c_config())?)
}

pub fn cadence_config() -> cadence_core::CadenceConfig {
    cadence_core::CadenceConfig {
        min_interval_ms: config::MIN_INTERVAL_MS,
        stop_timeout_ms: config::STOP_TIMEOUT_MS,
        sample_window: config::SAMPLE_WINDOW,
        min_samples_for_rpm: config::MIN_SAMPLES_FOR_RPM,
    }
}
