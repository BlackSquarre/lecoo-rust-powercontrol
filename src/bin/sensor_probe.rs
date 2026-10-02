//! Independent native sampling experiment; no GUI or vendor DLL is loaded.
use anyhow::{bail, Result};
use lecoo_control_center::platform::{
    windows::sensors::{cpu_brand, ExistingWinRing0},
    ComGuard, HardwareControl, WindowsHardwareControl,
};

fn run() -> Result<()> {
    if std::env::args().len() != 1 {
        bail!("Usage: sensor_probe.exe (no arguments)");
    }
    println!("CPU={}", cpu_brand());
    let driver = ExistingWinRing0::open()?;
    println!("DRIVER_VERSION=0x{:08x}", driver.version());
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
