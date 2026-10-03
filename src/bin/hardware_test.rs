use anyhow::{bail, Context, Result};
use lecoo_control_center::{
    core::{FeatureKey, PowerMode},
    platform::{ComGuard, HardwareControl, WindowsHardwareControl},
};

fn run() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    // Validate arguments before creating a connection or attempting any write.
    let requested_mode = match arguments.as_slice() {
        [] => None,
        [operation] if operation == "--thermal-zone" => None,
        [operation, value] if operation == "--set-mode" => Some(
            PowerMode::from_u32(value.parse().context("Mode must be 0, 1 or 2")?)
                .context("Mode must be 0, 1 or 2")?,
        ),
        _ => bail!("Usage: hardware_test.exe [--thermal-zone|--set-mode 0|1|2]"),
    };
    let _com = ComGuard::new()?;
    let mut hardware = WindowsHardwareControl::new()?;
    if arguments.as_slice() == ["--thermal-zone"] {
        println!(
            "THERMAL_ZONE_C={:?}",
            hardware.get_thermal_zone_temperature()?
        );
        return Ok(());
    }
    if !hardware.is_elevated() {
        bail!("Administrator privileges are required on this device");
    }
    if let Some(mode) = requested_mode {
        // Exactly the same library method used by the GUI.
        hardware.set_power_mode(mode)?;
        println!("SET_VERIFIED={}", mode as u32);
    }
    println!("MODE={}", hardware.get_power_mode()? as u32);
    println!("FAN={:?}", hardware.get_fan_speed(1)?);
    println!("TEMP={:?}", hardware.get_hw_temp(1)?);
    println!(
        "MODE_COUNT={:?}",
        hardware.get_feature_value(FeatureKey::ModeCount)?
    );
    println!(
        "FAN_COUNT={:?}",
        hardware.get_feature_value(FeatureKey::FanCount)?
    );
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
