#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lecoo_control_center::{core, platform};
mod ui;

use platform::HardwareControl;

#[cfg(target_os = "windows")]
use platform::windows::{ComGuard, WindowsHardwareControl};

fn main() {
    ui::initialize_language();
    if let Err(error) = run() {
        platform::windows::desktop::show_error(&format!(
            "{}: {}",
            lecoo_control_center::localization::text(
                lecoo_control_center::localization::Text::ErrorsAppStart
            ),
            lecoo_control_center::localization::user_error(&format!("{error:#}"))
        ));
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("--fan-worker") {
        if let Err(error) = platform::windows::fan_session::worker() {
            use std::io::Write;
            let _ = writeln!(std::io::stdout(), "ERROR {error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("--startup") {
        let result = std::env::current_exe()
            .map_err(anyhow::Error::from)
            .and_then(|exe| {
                platform::windows::startup::configure(
                    arguments.get(1).map(String::as_str).unwrap_or("status"),
                    &exe,
                )
            });
        match result {
            Ok(status) => println!("{status:?}"),
            Err(error) => {
                eprintln!("{error:#}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    let instance = match platform::windows::instance::Instance::acquire() {
        Ok(Some(instance)) => instance,
        Ok(None) => return Ok(()),
        Err(error) => {
            platform::windows::desktop::show_error(&format!(
                "{}: {}",
                lecoo_control_center::localization::text(
                    lecoo_control_center::localization::Text::ErrorsInstanceInit
                ),
                lecoo_control_center::localization::user_error(&format!("{error:#}"))
            ));
            return Ok(());
        }
    };
    let minimized = arguments.iter().any(|arg| arg == "--minimized");
    let smoke_report = arguments
        .iter()
        .position(|arg| arg == "--smoke-test")
        .and_then(|index| arguments.get(index + 1))
        .map(std::path::PathBuf::from);
    // 初始化 COM（Windows 平台）
    #[cfg(target_os = "windows")]
    let _com = ComGuard::new()?;

    // 创建硬件控制实例
    #[cfg(target_os = "windows")]
    let hw_control: Box<dyn HardwareControl> = Box::new(WindowsHardwareControl::new()?);

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("This application currently only supports Windows");
        std::process::exit(1);
    }

    ui::ControlCenterApp::run(hw_control, instance, minimized, smoke_report)
}
