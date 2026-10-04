use crate::core::types::SystemSnapshot;
use crate::platform::HardwareControl;
use std::time::{Duration, Instant};
use windows::{
    core::HSTRING,
    Win32::{
        Storage::FileSystem::GetDiskFreeSpaceExW,
        System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
    },
};

const SENSOR_INTERVAL: Duration = Duration::from_secs(1);
const TRAY_MODE_INTERVAL: Duration = Duration::from_secs(5);
const MEMORY_INTERVAL: Duration = Duration::from_secs(2);
const DISK_INTERVAL: Duration = Duration::from_secs(30);

/// Lives entirely in the UI's COM apartment. Own only the data the UI consumes.
pub struct SystemMonitor {
    hw_control: Box<dyn HardwareControl>,
    system_drive: HSTRING,
    visible: bool,
    mode_at: Option<Instant>,
    sensors_at: Option<Instant>,
    memory_at: Option<Instant>,
    disk_at: Option<Instant>,
    revision: u64,
    snapshot: SystemSnapshot,
}

fn due(last: Option<Instant>, now: Instant, interval: Duration) -> bool {
    last.is_none_or(|last| now.saturating_duration_since(last) >= interval)
}
fn usage(used: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64 * 100.0) as f32
    }
}

impl SystemMonitor {
    pub fn new(hw_control: Box<dyn HardwareControl>) -> Self {
        let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
        Self {
            hw_control,
            system_drive: HSTRING::from(format!("{}\\", drive.trim_end_matches(['\\', '/']))),
            visible: false,
            mode_at: None,
            sensors_at: None,
            memory_at: None,
            disk_at: None,
            revision: 0,
            snapshot: SystemSnapshot::default(),
        }
    }

    /// Compatibility entry point for isolated diagnostics; no snapshot allocation.
    pub fn update(&mut self) -> &SystemSnapshot {
        self.refresh(true);
        self.snapshot()
    }
    pub fn snapshot(&self) -> &SystemSnapshot {
        &self.snapshot
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn hardware_mut(&mut self) -> &mut dyn HardwareControl {
        &mut *self.hw_control
    }

    pub fn refresh(&mut self, visible: bool) -> bool {
        self.refresh_at(visible, Instant::now())
    }

    fn refresh_at(&mut self, visible: bool, now: Instant) -> bool {
        if visible && !self.visible {
            // Reopening shows fresh data even if the previous window closed just now.
            self.mode_at = None;
            self.sensors_at = None;
            self.memory_at = None;
            self.disk_at = None;
        }
        self.visible = visible;
        let mut changed = false;
        let mut sampled = false;
        let mode_interval = if visible {
            SENSOR_INTERVAL
        } else {
            TRAY_MODE_INTERVAL
        };
        if due(self.mode_at, now, mode_interval) {
            sampled = true;
            self.mode_at = Some(now);
            match self.hw_control.get_power_mode() {
                Ok(mode) => {
                    changed |= self.snapshot.power_mode != Some(mode)
                        || self.snapshot.power_mode_error.is_some();
                    self.snapshot.power_mode = Some(mode);
                    self.snapshot.power_mode_error = None;
                }
                Err(error) => {
                    let error = format!("读取当前模式失败: {error:#}");
                    changed |= self.snapshot.power_mode.is_some()
                        || self.snapshot.power_mode_error.as_ref() != Some(&error);
                    self.snapshot.power_mode = None;
                    self.snapshot.power_mode_error = Some(error);
                }
            }
        }
        if visible && due(self.sensors_at, now, SENSOR_INTERVAL) {
            sampled = true;
            self.sensors_at = Some(now);
            let fan_speed = self.hw_control.get_fan_speed(1).ok().flatten();
            let temperature = self
                .hw_control
                .get_thermal_zone_temperature()
                .ok()
                .flatten();
            changed |= (self.snapshot.fan_speed, self.snapshot.thermal_zone_temp)
                != (fan_speed, temperature);
            self.snapshot.fan_speed = fan_speed;
            self.snapshot.thermal_zone_temp = temperature;
        }
        if visible && due(self.memory_at, now, MEMORY_INTERVAL) {
            sampled = true;
            self.memory_at = Some(now);
            let mut memory = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                ..Default::default()
            };
            let (total, used) = if unsafe { GlobalMemoryStatusEx(&mut memory) }.is_ok() {
                (
                    memory.ullTotalPhys,
                    memory.ullTotalPhys.saturating_sub(memory.ullAvailPhys),
                )
            } else {
                (0, 0)
            };
            changed |= (self.snapshot.mem_total, self.snapshot.mem_used) != (total, used);
            self.snapshot.mem_total = total;
            self.snapshot.mem_used = used;
            self.snapshot.mem_usage = usage(used, total);
        }
        if visible && due(self.disk_at, now, DISK_INTERVAL) {
            sampled = true;
            self.disk_at = Some(now);
            let (mut total, mut free) = (0, 0);
            if unsafe {
                GetDiskFreeSpaceExW(&self.system_drive, Some(&mut free), Some(&mut total), None)
            }
            .is_err()
            {
                total = 0;
                free = 0;
            }
            let used = total.saturating_sub(free);
            changed |= (self.snapshot.disk_total, self.snapshot.disk_used) != (total, used);
            self.snapshot.disk_total = total;
            self.snapshot.disk_used = used;
            self.snapshot.disk_usage = usage(self.snapshot.disk_used, total);
        }
        if sampled {
            self.snapshot.timestamp = now;
        }
        if changed {
            self.revision = self.revision.wrapping_add(1);
        }
        changed
    }

    /// A mode write requires immediate readback, without rescanning unrelated metrics.
    pub fn invalidate(&mut self) {
        self.mode_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{types::FeatureKey, PowerMode};
    use anyhow::Result;
    use std::{cell::Cell, rc::Rc};

    #[derive(Default)]
    struct Reads {
        mode: Cell<u32>,
        fan: Cell<u32>,
        thermal: Cell<u32>,
    }
    struct Fake(Rc<Reads>);
    impl HardwareControl for Fake {
        fn get_power_mode(&self) -> Result<PowerMode> {
            self.0.mode.set(self.0.mode.get() + 1);
            Ok(PowerMode::Balance)
        }
        fn set_power_mode(&mut self, _: PowerMode) -> Result<()> {
            panic!("monitor must not write hardware")
        }
        fn get_fan_speed(&self, _: u8) -> Result<Option<u32>> {
            self.0.fan.set(self.0.fan.get() + 1);
            Ok(Some(2000))
        }
        fn get_hw_temp(&self, _: u8) -> Result<Option<u32>> {
            Ok(None)
        }
        fn get_thermal_zone_temperature(&self) -> Result<Option<f32>> {
            self.0.thermal.set(self.0.thermal.get() + 1);
            Ok(Some(40.0))
        }
        fn get_feature_value(&self, _: FeatureKey) -> Result<Option<u32>> {
            Ok(None)
        }
        fn is_elevated(&self) -> bool {
            false
        }
    }
    #[test]
    fn hidden_skips_ui_metrics_and_reopening_refreshes_immediately() {
        let reads = Rc::new(Reads::default());
        let mut monitor = SystemMonitor::new(Box::new(Fake(reads.clone())));
        let now = Instant::now();
        monitor.refresh_at(false, now);
        for seconds in 1..=4 {
            assert!(!monitor.refresh_at(false, now + Duration::from_secs(seconds)));
        }
        assert_eq!(
            (reads.mode.get(), reads.fan.get(), reads.thermal.get()),
            (1, 0, 0)
        );
        assert_eq!(monitor.snapshot().mem_total, 0);
        let revision = monitor.revision();
        assert!(!monitor.refresh_at(false, now + Duration::from_secs(5)));
        assert_eq!(
            monitor.revision(),
            revision,
            "unchanged mode must not schedule rendering"
        );
        assert!(monitor.refresh_at(true, now + Duration::from_secs(5)));
        assert_eq!(
            (reads.mode.get(), reads.fan.get(), reads.thermal.get()),
            (3, 1, 1)
        );
        assert!(!monitor.refresh_at(true, now + Duration::from_millis(5500)));
        monitor.invalidate();
        assert!(!monitor.refresh_at(true, now + Duration::from_millis(5500)));
        assert_eq!(
            (reads.mode.get(), reads.fan.get(), reads.thermal.get()),
            (4, 1, 1)
        );
        monitor.refresh_at(false, now + Duration::from_secs(6));
        monitor.refresh_at(true, now + Duration::from_secs(6));
        assert_eq!(reads.fan.get(), 2);
    }
    #[test]
    fn visible_metrics_use_independent_cadences() {
        let reads = Rc::new(Reads::default());
        let mut monitor = SystemMonitor::new(Box::new(Fake(reads.clone())));
        let now = Instant::now();
        monitor.refresh_at(true, now);
        let disk_at = monitor.disk_at;
        let memory_at = monitor.memory_at;
        monitor.refresh_at(true, now + Duration::from_secs(1));
        assert_eq!(monitor.disk_at, disk_at);
        assert_eq!(monitor.memory_at, memory_at);
        monitor.refresh_at(true, now + Duration::from_secs(2));
        assert_ne!(monitor.memory_at, memory_at);
        assert_eq!(monitor.disk_at, disk_at);
        monitor.refresh_at(true, now + Duration::from_secs(30));
        assert_ne!(monitor.disk_at, disk_at);
        assert_eq!(reads.fan.get(), 4);
    }
}
