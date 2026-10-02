use anyhow::{bail, Result};
use std::time::Duration;

pub const MAX_MANUAL_TEMP: f32 = 85.0;
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanTarget {
    Auto,
    Percent(u8),
}
impl FanTarget {
    pub fn percent(value: u8) -> Result<Self> {
        if !(35..=100).contains(&value) {
            bail!("风扇目标必须在 35–100% 内");
        }
        Ok(Self::Percent(value))
    }
    pub fn validate(self) -> Result<()> {
        if let Self::Percent(value) = self {
            Self::percent(value)?;
        }
        Ok(())
    }
    pub fn encoded(self) -> u8 {
        match self {
            Self::Auto => 101,
            Self::Percent(value) => value,
        }
    }
}

pub trait CoolingIo {
    fn write_fan(&mut self, target: FanTarget) -> Result<u32>;
    fn measured_rpm(&self) -> Result<u32>;
    fn package_temperature(&self) -> Result<f32>;
}

#[derive(Debug, Clone, Copy)]
pub struct FanReceipt {
    pub target: FanTarget,
    pub status: u32,
    pub rpm: u32,
}

/// Owns a manual session. RPM is telemetry, never presented as duty/mode readback.
pub struct CoolingController<T: CoolingIo> {
    io: T,
    armed: bool,
    target: FanTarget,
}
impl<T: CoolingIo> CoolingController<T> {
    pub fn new(io: T) -> Self {
        Self {
            io,
            armed: false,
            target: FanTarget::Auto,
        }
    }
    pub fn target(&self) -> FanTarget {
        self.target
    }
    pub fn sample(&self) -> Result<(f32, u32)> {
        let temp = self.io.package_temperature()?;
        let rpm = self.io.measured_rpm()?;
        if !temp.is_finite() || !(0.0..=125.0).contains(&temp) || rpm == 0 {
            bail!("温度或风扇采样无效");
        }
        Ok((temp, rpm))
    }
    pub fn apply(&mut self, target: FanTarget) -> Result<FanReceipt> {
        target.validate()?; // Reject invalid values before all hardware calls.
        if target == FanTarget::Auto {
            return self.restore_auto();
        }
        let (temp, _) = match self.sample() {
            Ok(value) => value,
            Err(error) => {
                if self.armed {
                    if let Err(restore) = self.restore_auto() {
                        bail!("采样失败: {error:#}；恢复失败: {restore:#}");
                    }
                }
                return Err(error);
            }
        };
        if temp >= MAX_MANUAL_TEMP {
            if self.armed {
                self.restore_auto()?;
            }
            bail!("CPU 温度达到 {temp:.1}°C，已拒绝手动控制");
        }
        // Arm before writing: an error can mean the firmware changed despite the transport error.
        self.armed = true;
        let result = (|| {
            let status = self.io.write_fan(target)?;
            if status == 255 {
                bail!("固件拒绝风扇请求，ResultStatus=255");
            }
            let (temp, rpm) = self.sample()?;
            if temp >= MAX_MANUAL_TEMP {
                bail!("写入后温度过高: {temp:.1}°C");
            }
            Ok(FanReceipt {
                target,
                status,
                rpm,
            })
        })();
        match result {
            Ok(receipt) => {
                self.target = target;
                Ok(receipt)
            }
            Err(error) => match self.restore_auto() {
                Ok(_) => Err(error),
                Err(restore) => bail!("风扇请求失败: {error:#}；恢复也失败: {restore:#}"),
            },
        }
    }
    pub fn restore_auto(&mut self) -> Result<FanReceipt> {
        // Mark the attempt as armed until both the firmware response and RPM read succeed.
        self.armed = true;
        let mut last_error = None;
        for _ in 0..3 {
            match (|| {
                let status = self.io.write_fan(FanTarget::Auto)?;
                if status == 255 {
                    bail!("固件拒绝自动控制");
                }
                let rpm = self.io.measured_rpm()?;
                if rpm == 0 {
                    bail!("自动恢复后风扇转速无效");
                }
                Ok(FanReceipt {
                    target: FanTarget::Auto,
                    status,
                    rpm,
                })
            })() {
                Ok(receipt) => {
                    self.armed = false;
                    self.target = FanTarget::Auto;
                    return Ok(receipt);
                }
                Err(error) => {
                    last_error = Some(error);
                }
            }
        }
        // Best-effort full cooling when automatic restoration cannot be acknowledged.
        let fallback = self.io.write_fan(FanTarget::Percent(100));
        bail!(
            "无法确认恢复自动控制: {:#}; 最大风量备用请求: {:?}",
            last_error.unwrap(),
            fallback
        );
    }
    pub fn guard(&mut self, heartbeat_age: Duration) -> Result<()> {
        if !self.armed {
            return Ok(());
        }
        let sample = self.sample();
        let reason = if heartbeat_age >= HEARTBEAT_TIMEOUT {
            Some("主程序心跳超时")
        } else if sample
            .as_ref()
            .map(|(temp, _)| *temp >= MAX_MANUAL_TEMP)
            .unwrap_or(true)
        {
            Some("过温或采样失败")
        } else {
            None
        };
        if let Some(reason) = reason {
            self.restore_auto()?;
            bail!("{reason}，已提交自动恢复并读回转速");
        }
        Ok(())
    }
}
impl<T: CoolingIo> Drop for CoolingController<T> {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.restore_auto();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    struct Fake {
        writes: Rc<RefCell<Vec<FanTarget>>>,
        temp: f32,
        fail_manual: bool,
    }
    impl CoolingIo for Fake {
        fn write_fan(&mut self, target: FanTarget) -> Result<u32> {
            self.writes.borrow_mut().push(target);
            if self.fail_manual && target != FanTarget::Auto {
                bail!("injected communication failure");
            }
            Ok(0)
        }
        fn measured_rpm(&self) -> Result<u32> {
            Ok(2000)
        }
        fn package_temperature(&self) -> Result<f32> {
            Ok(self.temp)
        }
    }
    fn controller(temp: f32, fail: bool) -> (CoolingController<Fake>, Rc<RefCell<Vec<FanTarget>>>) {
        let writes = Rc::new(RefCell::new(Vec::new()));
        (
            CoolingController::new(Fake {
                writes: writes.clone(),
                temp,
                fail_manual: fail,
            }),
            writes,
        )
    }
    #[test]
    fn invalid_and_hot_requests_never_write() {
        let (mut ctrl, writes) = controller(90.0, false);
        assert!(ctrl.apply(FanTarget::Percent(34)).is_err());
        assert!(ctrl.apply(FanTarget::Percent(50)).is_err());
        assert!(writes.borrow().is_empty());
    }
    #[test]
    fn write_failure_attempts_auto() {
        let (mut ctrl, writes) = controller(50.0, true);
        assert!(ctrl.apply(FanTarget::Percent(50)).is_err());
        assert_eq!(
            *writes.borrow(),
            vec![FanTarget::Percent(50), FanTarget::Auto]
        );
    }
    #[test]
    fn parent_timeout_restores_auto() {
        let (mut ctrl, writes) = controller(50.0, false);
        ctrl.apply(FanTarget::Percent(50)).unwrap();
        assert!(ctrl.guard(HEARTBEAT_TIMEOUT).is_err());
        assert_eq!(writes.borrow().last(), Some(&FanTarget::Auto));
        assert_eq!(ctrl.target(), FanTarget::Auto);
    }
    #[test]
    fn temperature_failure_and_exit_restore_auto() {
        for temp in [90.0, f32::NAN] {
            let (mut ctrl, writes) = controller(50.0, false);
            ctrl.apply(FanTarget::Percent(50)).unwrap();
            ctrl.io.temp = temp;
            assert!(ctrl.guard(Duration::ZERO).is_err());
            assert_eq!(writes.borrow().last(), Some(&FanTarget::Auto));
        }
        let (mut ctrl, writes) = controller(50.0, false);
        ctrl.apply(FanTarget::Percent(50)).unwrap();
        drop(ctrl);
        assert_eq!(writes.borrow().last(), Some(&FanTarget::Auto));
    }
    #[test]
    fn range_and_encoding() {
        for value in 0..=255 {
            assert_eq!(
                FanTarget::percent(value).is_ok(),
                (35..=100).contains(&value)
            );
        }
        assert_eq!(FanTarget::Auto.encoded(), 101);
    }
    struct Faulty {
        writes: Rc<RefCell<Vec<FanTarget>>>,
        post_temp: f32,
        reject_manual: bool,
        fail_auto: bool,
    }
    impl CoolingIo for Faulty {
        fn write_fan(&mut self, target: FanTarget) -> Result<u32> {
            self.writes.borrow_mut().push(target);
            if self.fail_auto && target == FanTarget::Auto {
                bail!("auto transport failed");
            }
            Ok(if self.reject_manual && target != FanTarget::Auto {
                255
            } else {
                0
            })
        }
        fn measured_rpm(&self) -> Result<u32> {
            Ok(2000)
        }
        fn package_temperature(&self) -> Result<f32> {
            Ok(
                if self
                    .writes
                    .borrow()
                    .last()
                    .is_some_and(|v| *v != FanTarget::Auto)
                {
                    self.post_temp
                } else {
                    50.0
                },
            )
        }
    }
    #[test]
    fn firmware_rejection_or_post_write_bad_sample_restores_auto() {
        for (temp, reject) in [(50.0, true), (90.0, false), (f32::NAN, false)] {
            let writes = Rc::new(RefCell::new(Vec::new()));
            let mut ctrl = CoolingController::new(Faulty {
                writes: writes.clone(),
                post_temp: temp,
                reject_manual: reject,
                fail_auto: false,
            });
            assert!(ctrl.apply(FanTarget::Percent(50)).is_err());
            assert_eq!(
                *writes.borrow(),
                vec![FanTarget::Percent(50), FanTarget::Auto]
            );
            assert_eq!(ctrl.target(), FanTarget::Auto);
        }
    }
    #[test]
    fn recovery_failure_retries_then_requests_full_cooling_and_remains_armed() {
        let writes = Rc::new(RefCell::new(Vec::new()));
        let mut ctrl = CoolingController::new(Faulty {
            writes: writes.clone(),
            post_temp: 50.0,
            reject_manual: false,
            fail_auto: true,
        });
        ctrl.apply(FanTarget::Percent(50)).unwrap();
        assert!(ctrl.restore_auto().is_err());
        assert_eq!(
            &writes.borrow()[1..],
            &[
                FanTarget::Auto,
                FanTarget::Auto,
                FanTarget::Auto,
                FanTarget::Percent(100)
            ]
        );
        assert!(ctrl.armed);
    }
}
