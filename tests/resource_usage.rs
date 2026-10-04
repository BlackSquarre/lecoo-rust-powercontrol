//! Isolated monitor comparison. Uses simulated hardware and never changes settings.
use anyhow::Result;
use lecoo_control_center::{
    core::{types::FeatureKey, PowerMode, SystemMonitor},
    platform::HardwareControl,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use windows::Win32::{
    Foundation::FILETIME,
    System::{
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX},
        Threading::{GetCurrentProcess, GetProcessTimes},
    },
};

#[derive(Default)]
struct Reads {
    mode: Cell<u32>,
    fan: Cell<u32>,
    thermal: Cell<u32>,
}
struct FakeHardware(Rc<Reads>);
impl HardwareControl for FakeHardware {
    fn get_power_mode(&self) -> Result<PowerMode> {
        self.0.mode.set(self.0.mode.get() + 1);
        Ok(PowerMode::Balance)
    }
    fn set_power_mode(&mut self, _: PowerMode) -> Result<()> {
        panic!("resource probe must never write hardware");
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

fn counters(stage: &str) -> u64 {
    unsafe {
        let process = GetCurrentProcess();
        let mut memory = PROCESS_MEMORY_COUNTERS_EX::default();
        GetProcessMemoryInfo(
            process,
            (&mut memory as *mut PROCESS_MEMORY_COUNTERS_EX).cast(),
            std::mem::size_of_val(&memory) as u32,
        )
        .unwrap();
        let (mut created, mut exited, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user).unwrap();
        let ticks = |v: FILETIME| (u64::from(v.dwHighDateTime) << 32) | u64::from(v.dwLowDateTime);
        let cpu = ticks(kernel) + ticks(user);
        println!("{{\"stage\":\"{stage}\",\"private_commit_bytes\":{},\"working_set_bytes\":{},\"cpu_100ns\":{cpu}}}", memory.PrivateUsage, memory.WorkingSetSize);
        cpu
    }
}

#[test]
#[ignore = "15-second isolated resource measurement; no hardware access"]
fn monitor_resource_usage() {
    probe(false);
}
#[test]
#[ignore = "15-second isolated resource measurement; no hardware access"]
fn monitor_hidden_resource_usage() {
    probe(true);
}

fn probe(hidden: bool) {
    let calls = Rc::new(Reads::default());
    counters("before_init");
    let mut monitor = SystemMonitor::new(Box::new(FakeHardware(calls.clone())));
    counters("after_init");
    monitor.refresh(!hidden);
    let initial_cpu = counters("after_first_sample");
    for _ in 0..60 {
        std::thread::sleep(Duration::from_millis(250));
        // Mirrors the old controller's tick + render path, including cached updates.
        monitor.refresh(!hidden);
        monitor.refresh(!hidden);
    }
    let final_cpu = counters("steady_end");
    println!(
        "{{\"sample_seconds\":15,\"hidden\":{hidden},\"mode_reads\":{},\"fan_reads\":{},\"thermal_reads\":{},\"cpu_ms\":{:.4},\"hardware_writes\":false}}",
        calls.mode.get(), calls.fan.get(), calls.thermal.get(),
        (final_cpu - initial_cpu) as f64 / 10000.0
    );
}
