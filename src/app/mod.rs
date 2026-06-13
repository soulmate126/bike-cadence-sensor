//! 应用主循环（霍尔 + OLED + BLE 及其子集）

use std::sync::Arc;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;

use crate::ble::csc::CadenceData;
use crate::ble::server::CscServer;
use crate::board::config::{BLE_DIAG_INTERVAL_MS, LOOP_DELAY_MS};
use crate::board::pins::{hall_pin, oled_i2c, status_led_pin};
use crate::board::status_led::StatusLed;
use crate::board;
use crate::cadence::CadenceSnapshot;
use crate::display::CadenceOled;
use crate::sensors::HallSensor;
use crate::util::now_ms;

struct AppParts<'d> {
    hall: HallSensor<'d>,
    oled: Option<CadenceOled<'d>>,
    ble: Option<Arc<CscServer>>,
    led: Option<StatusLed<'d>>,
}

struct LoopOptions {
    log_pulses: bool,
}

/// 主固件：霍尔 + OLED（可选）+ BLE CSC（不返回）。
pub fn run_full() -> ! {
    run_fatal("app", || {
        let peripherals = Peripherals::take()?;
        let hall = HallSensor::new(hall_pin(peripherals.pins.gpio4)?)?;
        let oled = match oled_i2c(peripherals.i2c0, peripherals.pins.gpio5, peripherals.pins.gpio6)
        {
            Ok(i2c) => match CadenceOled::new(i2c) {
                Ok(o) => Some(o),
                Err(e) => {
                    log::warn!("app: OLED init failed ({e:?}), continuing without display");
                    None
                }
            },
            Err(e) => {
                log::warn!("app: I2C init failed ({e:?}), continuing without display");
                None
            }
        };
        let ble = Some(CscServer::begin()?);
        let led = match status_led_pin(peripherals.pins.gpio8) {
            Ok(p) => Some(StatusLed::new(p)),
            Err(e) => {
                log::warn!("app: status LED unavailable ({e:?})");
                None
            }
        };

        log::info!(
            "app: hall GPIO{} | OLED {} | BLE CSC | LED status",
            board::HALL_GPIO,
            if oled.is_some() { "on" } else { "off" }
        );

        Ok(run_loop(
            AppParts {
                hall,
                oled,
                ble,
                led,
            },
            LoopOptions { log_pulses: false },
        ))
    })
}

/// 示例 5：霍尔 + BLE CSC（不返回）。
pub fn run_hall_ble() -> ! {
    run_fatal("hall_ble", || {
        let peripherals = Peripherals::take()?;
        let hall = HallSensor::new(hall_pin(peripherals.pins.gpio4)?)?;
        let ble = Some(CscServer::begin()?);
        let led = status_led_pin(peripherals.pins.gpio8)
            .ok()
            .map(StatusLed::new);

        log::info!("hall_ble: GPIO{} + BLE CSC", board::HALL_GPIO);

        Ok(run_loop(
            AppParts {
                hall,
                oled: None,
                ble,
                led,
            },
            LoopOptions { log_pulses: false },
        ))
    })
}

/// 示例 6：霍尔 + OLED（不返回）。
pub fn run_hall_oled() -> ! {
    run_fatal("hall_oled", || {
        let peripherals = Peripherals::take()?;
        let hall = HallSensor::new(hall_pin(peripherals.pins.gpio4)?)?;
        let i2c = oled_i2c(peripherals.i2c0, peripherals.pins.gpio5, peripherals.pins.gpio6)?;
        let oled = Some(CadenceOled::new(i2c)?);

        log::info!(
            "hall_oled: GPIO{} + OLED GPIO{}/{}",
            board::HALL_GPIO,
            board::I2C_SDA_GPIO,
            board::I2C_SCL_GPIO
        );

        Ok(run_loop(
            AppParts {
                hall,
                oled,
                ble: None,
                led: None,
            },
            LoopOptions { log_pulses: true },
        ))
    })
}

fn run_fatal(name: &str, setup: impl FnOnce() -> anyhow::Result<std::convert::Infallible>) -> ! {
    match setup() {
        Ok(infallible) => match infallible {},
        Err(e) => {
            log::error!("{name} fatal: {e:?}");
            loop {
                FreeRtos::delay_ms(1000);
            }
        }
    }
}

fn run_loop(
    mut parts: AppParts<'_>,
    opts: LoopOptions,
) -> std::convert::Infallible {
    let mut last_ble_diag_ms = 0u64;

    loop {
        let now = now_ms();

        if let Some(ref ble) = parts.ble {
            let connected = ble.is_connected();
            if let Some(ref mut led) = parts.led {
                led.set_ble_connected(connected);
            }

            if now.saturating_sub(last_ble_diag_ms) >= BLE_DIAG_INTERVAL_MS {
                last_ble_diag_ms = now;
                let subs = ble.subscriber_count();
                if connected && subs == 0 {
                    log::warn!(
                        "CSC: connected but NO 0x2A5B subscription — \
                         start 户外骑行 on watch and spin crank; \
                         if adding device only, this is normal"
                    );
                } else if connected {
                    log::info!("CSC: connected, subscribers={subs}");
                }
            }
        }

        if let Some(snapshot) = parts.hall.poll() {
            if opts.log_pulses {
                log::info!(
                    "pulse revs={} rpm={:.0}",
                    snapshot.cumulative_revolutions,
                    snapshot.rpm
                );
            }
            if let Some(ref ble) = parts.ble {
                notify_csc(ble, &snapshot);
            }
        }

        if let Some(ref mut oled) = parts.oled {
            let snap = parts.hall.snapshot();
            let _ = oled.update(snap.rpm, snap.cumulative_revolutions, now);
        }

        if let Some(ref mut led) = parts.led {
            led.tick(parts.hall.rpm(), now);
        }

        FreeRtos::delay_ms(LOOP_DELAY_MS);
    }
}

fn notify_csc(server: &Arc<CscServer>, snapshot: &CadenceSnapshot) {
    let data = CadenceData::from_snapshot(snapshot);
    server.notify_measurement(&data);
    log::info!(
        "CSC notify: revs={} time={} contact={} rpm={:.0}",
        data.crank_revolutions,
        data.last_event_time,
        data.sensor_contact,
        snapshot.rpm
    );
}
