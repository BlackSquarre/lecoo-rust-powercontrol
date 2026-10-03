//! Independent native sampling experiment; no GUI or vendor DLL is loaded.
use anyhow::{bail, Result};
use lecoo_control_center::platform::{
    windows::sensors::{cpu_brand, ExistingWinRing0},
    ComGuard, HardwareControl, WindowsHardwareControl,
};

fn run() -> Result<()> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let power = arguments.as_slice() == ["--power"];
    if !arguments.is_empty() && !power {
        bail!("Usage: sensor_probe.exe [--power]");
    }
    println!("CPU={}", cpu_brand());
    let driver = ExistingWinRing0::open()?;
    println!("DRIVER_VERSION=0x{:08x}", driver.version());
    if power {
        let mut previous = driver.read_package_energy()?;
        println!(
            "POWER_SOURCE=AMD_PACKAGE_ENERGY_MSR UNIT_RAW=0x{:016x} ENERGY_UNIT_J={:.12}",
            previous.raw_unit_register,
            2f64.powi(-(((previous.raw_unit_register >> 8) & 0x1f) as i32))
        );
        for index in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let current = driver.read_package_energy()?;
            let watts = current.watts_since(&previous)?;
            let temperature = driver.read_package_temperature()?;
            println!("POWER_SAMPLE={index} COUNTER_BEFORE={} COUNTER_AFTER={} ELAPSED_S={:.6} CPU_PACKAGE_W={watts:.3} CPU_PACKAGE_C={:.3}",
                previous.counter, current.counter,
                current.sampled_at.duration_since(previous.sampled_at).as_secs_f64(), temperature.celsius);
            previous = current;
        }
        return Ok(());
    }
    let _com = ComGuard::new()?;
    let hardware = WindowsHardwareControl::new()?;
    println!("MODE={}", hardware.get_power_mode()? as u32);
    println!(
        "FAN_COUNT={:?}",
        hardware.get_feature_value(lecoo_control_center::core::FeatureKey::FanCount)?
    );
    for index in 0..5 {
        let temperature = driver.read_package_temperature()?;
        println!(
            "SAMPLE={index} RAW=0x{:08x} CPU_PACKAGE_C={:.3} WMI_TEMP={:?} FAN={:?}",
            temperature.raw_register,
            temperature.celsius,
            hardware.get_hw_temp(1)?,
            hardware.get_fan_speed(1)?
        );
        if index < 4 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ERROR: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
