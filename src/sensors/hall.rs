//! KY-003 霍尔 + 踏频算法（下降沿中断 + 轮询更新）

use core::sync::atomic::{AtomicBool, Ordering};

use esp_idf_hal::gpio::{Input, InterruptType, PinDriver};

use crate::cadence::{new_calculator, CadenceCalculator, CadenceSnapshot};
use crate::util::now_ms;

static HALL_EDGE_PENDING: AtomicBool = AtomicBool::new(false);

pub struct HallSensor<'d> {
    pin: PinDriver<'d, Input>,
    calc: CadenceCalculator,
}

impl<'d> HallSensor<'d> {
    pub fn new(mut pin: PinDriver<'d, Input>) -> anyhow::Result<Self> {
        HALL_EDGE_PENDING.store(false, Ordering::Release);
        pin.set_interrupt_type(InterruptType::NegEdge)?;
        unsafe {
            pin.subscribe(|| HALL_EDGE_PENDING.store(true, Ordering::Release))?;
        }
        pin.enable_interrupt()?;

        Ok(Self {
            pin,
            calc: new_calculator(),
        })
    }

    /// 处理中断边沿并更新算法。计入有效转数时返回快照。
    pub fn poll(&mut self) -> Option<CadenceSnapshot> {
        let now = now_ms();
        let contact = self.sensor_contact();
        let mut counted = false;

        if HALL_EDGE_PENDING.swap(false, Ordering::AcqRel) {
            counted = self.calc.on_pulse(now);
        }

        self.calc.update(now);
        let _ = self.pin.enable_interrupt();

        if counted {
            Some(self.calc.snapshot(contact))
        } else {
            None
        }
    }

    pub fn rpm(&self) -> f32 {
        self.calc.rpm()
    }

    pub fn snapshot(&self) -> CadenceSnapshot {
        self.calc.snapshot(self.sensor_contact())
    }

    fn sensor_contact(&self) -> bool {
        self.pin.is_low()
    }
}
