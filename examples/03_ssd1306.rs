//! 示例 3：SSD1306 OLED 显示 "Hello Bike"

use bike_cadence_sensor::{board, init};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::Hertz;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

fn main() -> anyhow::Result<()> {
    init();

    let peripherals = Peripherals::take().unwrap();
    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio6,
        &i2c_config,
    )?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Text::new("Hello Bike", Point::new(0, 16), MonoTextStyle::new(&FONT_6X10, BinaryColor::On))
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    display.flush().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    log::info!("03_ssd1306: displayed Hello Bike on I2C 0x{:02X}", board::OLED_I2C_ADDR);

    loop {
        esp_idf_hal::delay::FreeRtos::delay_ms(1000);
    }
}
