use crate::core::types::SystemSnapshot;
use crate::platform::HardwareControl;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};

pub struct SystemMonitor {
    hw_control: Arc<Mutex<Box<dyn HardwareControl>>>,
    sys_info: System,
    disks: Disks,
    last_update: Instant,
    update_interval: Duration,
    snapshot: SystemSnapshot,
}

impl SystemMonitor {
    pub fn new(hw_control: Box<dyn HardwareControl>) -> Self {
        Self {
            hw_control: Arc::new(Mutex::new(hw_control)),
            // Only memory is consumed here; avoid retaining an unused process/CPU inventory.
            sys_info: System::new(),
            disks: Disks::new_with_refreshed_list(),
            last_update: Instant::now() - Duration::from_secs(1),
            update_interval: Duration::from_secs(1),
            snapshot: SystemSnapshot::default(),
        }
    }

    pub fn update(&mut self) -> SystemSnapshot {
        let now = Instant::now();

        // 只有超过更新间隔才真正刷新
        if now.duration_since(self.last_update) < self.update_interval {
            return self.snapshot.clone();
        }

        self.last_update = now;

        // 刷新系统信息
        self.sys_info.refresh_memory();
        self.disks.refresh(true);

        self.snapshot = self.get_current_snapshot();
        self.snapshot.clone()
    }

    fn get_current_snapshot(&self) -> SystemSnapshot {
        let mut snapshot = SystemSnapshot {
            timestamp: Instant::now(),
            ..Default::default()
        };

        // 获取硬件控制数据
        if let Ok(mut hw) = self.hw_control.lock() {
            match hw.get_power_mode() {
                Ok(mode) => snapshot.power_mode = Some(mode),
                Err(error) => {
                    snapshot.power_mode_error = Some(format!("读取当前模式失败: {:#}", error))
                }
            }
            snapshot.fan_speed = hw.get_fan_speed(1).ok().flatten();
            snapshot.cpu_temp = hw.get_cpu_temperature().ok().flatten();
            snapshot.cpu_package_power = hw.get_cpu_package_power().ok().flatten();
        }

        // 内存信息
        snapshot.mem_total = self.sys_info.total_memory();
        snapshot.mem_used = self.sys_info.used_memory();
        snapshot.mem_usage = if snapshot.mem_total > 0 {
            (snapshot.mem_used as f32 / snapshot.mem_total as f32) * 100.0
        } else {
            0.0
        };

        // Prefer the Windows system drive; enumeration order can otherwise show another volume.
        let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
        let system_disk = self
            .disks
            .list()
            .iter()
            .find(|disk| {
                disk.mount_point()
                    .to_string_lossy()
                    .trim_end_matches(['\\', '/'])
                    .eq_ignore_ascii_case(system_drive.trim_end_matches(['\\', '/']))
            })
            .or_else(|| self.disks.list().first());
        if let Some(disk) = system_disk {
            snapshot.disk_total = disk.total_space();
            snapshot.disk_used = snapshot.disk_total - disk.available_space();
            snapshot.disk_usage = if snapshot.disk_total > 0 {
                (snapshot.disk_used as f32 / snapshot.disk_total as f32) * 100.0
            } else {
                0.0
            };
        }

        snapshot
    }

    pub fn get_hw_control(&self) -> Arc<Mutex<Box<dyn HardwareControl>>> {
        self.hw_control.clone()
    }

    pub fn invalidate(&mut self) {
        self.last_update = Instant::now() - self.update_interval;
    }
}
