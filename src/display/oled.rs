//! SSD1306 显示（128×64）

use core::fmt::Write;

use display_interface_i2c::I2CInterface;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::peripherals::Peripherals;
use heapless::String;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use crate::hardware::i2c::{default_i2c_config, quick_scan, resolve_oled_addr};

use crate::board::config::UI_REFRESH_MS;

type Ssd1306Buffered<'d> = Ssd1306<
    I2CInterface<I2cDriver<'d>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

fn open_display(i2c: I2cDriver<'_>) -> anyhow::Result<Ssd1306Buffered<'_>> {
    let mut i2c = i2c;
    let addr = resolve_oled_addr(&mut i2c)
        .ok_or_else(|| anyhow::anyhow!("OLED not found at 0x3C/0x3D"))?;
    log::info!("OLED at 0x{addr:02X}");

    let interface = I2CDisplayInterface::new_custom_address(i2c, addr);
    let mut display =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
    display.init().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(display)
}

pub struct CadenceOled<'d> {
    display: Ssd1306Buffered<'d>,
    last_drawn_rpm: i32,
    last_drawn_count: u16,
    last_ui_ms: u64,
}

impl<'d> CadenceOled<'d> {
    pub fn new(i2c: I2cDriver<'d>) -> anyhow::Result<Self> {
        Ok(Self {
            display: open_display(i2c)?,
            last_drawn_rpm: -1,
            last_drawn_count: u16::MAX,
            last_ui_ms: 0,
        })
    }

    /// 踏频或转数变化时刷新屏幕（至少每 200ms 刷新一次，保证 RPM 归零可见）。
    pub fn update(&mut self, rpm: f32, count: u16, now_ms: u64) -> anyhow::Result<()> {
        let rpm_round = rpm.round() as i32;
        let stale = now_ms.saturating_sub(self.last_ui_ms) >= UI_REFRESH_MS;
        if rpm_round == self.last_drawn_rpm && count == self.last_drawn_count && !stale {
            return Ok(());
        }

        let label_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let value_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

        self.display.clear_buffer();

        Text::new("CADENCE", Point::new(0, 9), label_style)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let mut rpm_line = String::<16>::new();
        write!(rpm_line, "{rpm_round} RPM").map_err(|_| anyhow::anyhow!("fmt"))?;
        Text::new(&rpm_line, Point::new(0, 30), value_style)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        Text::new("COUNT", Point::new(0, 52), label_style)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let mut count_line = String::<8>::new();
        write!(count_line, "{count}").map_err(|_| anyhow::anyhow!("fmt"))?;
        Text::new(&count_line, Point::new(64, 63), value_style)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        self.display.flush().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        self.last_drawn_rpm = rpm_round;
        self.last_drawn_count = count;
        self.last_ui_ms = now_ms;
        Ok(())
    }
}

/// 示例 3：I2C 扫描并显示 "Hello Bike"（不返回）。
pub fn run_hello_demo() -> ! {
    let peripherals = Peripherals::take().expect("peripherals");
    let i2c_config = default_i2c_config();
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio6,
        &i2c_config,
    )
    .expect("i2c");

    quick_scan(&mut i2c);
    let mut display = open_display(i2c).unwrap_or_else(|e| {
        log::error!("oled hello: {e}");
        loop {
            FreeRtos::delay_ms(1000);
        }
    });

    Text::new(
        "Hello Bike",
        Point::new(0, 16),
        MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
    )
    .draw(&mut display)
    .expect("draw");
    display.flush().expect("flush");
    log::info!("oled hello: displayed Hello Bike");

    loop {
        FreeRtos::delay_ms(1000);
    }
}
