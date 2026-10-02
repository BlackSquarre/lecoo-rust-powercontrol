#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lecoo_control_center::{core, platform};
mod ui;

use eframe::egui;
use platform::HardwareControl;

#[cfg(target_os = "windows")]
use platform::windows::{ComGuard, WindowsHardwareControl};

fn main() {
    if let Err(error) = run() {
        platform::windows::desktop::show_error(&format!("应用启动失败: {error}"));
        std::process::exit(1);
    }
}

fn run() -> Result<(), eframe::Error> {
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
            platform::windows::desktop::show_error(&format!("单实例初始化失败: {error:#}"));
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
    let _com = ComGuard::new().map_err(|error| eframe::Error::AppCreation(error.into()))?;

    // 创建硬件控制实例
    #[cfg(target_os = "windows")]
    let hw_control: Box<dyn HardwareControl> = Box::new(
        WindowsHardwareControl::new().map_err(|error| eframe::Error::AppCreation(error.into()))?,
    );

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("This application currently only supports Windows");
        std::process::exit(1);
    }

    // 启动 GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(egui::IconData {
                rgba: platform::windows::desktop::icon_rgba(),
                width: 32,
                height: 32,
            })
            .with_title(concat!("Lecoo Rust PowerControl v", env!("CARGO_PKG_VERSION"))),
        ..Default::default()
    };

    eframe::run_native(
        "Lecoo Rust PowerControl",
        options,
        Box::new(move |cc| {
            Ok(Box::new(ui::ControlCenterApp::new(
                cc,
                hw_control,
                instance,
                minimized,
                smoke_report,
            )?))
        }),
    )
}
