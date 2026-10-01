#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lecoo_control_center::{core, platform};
mod ui;

use eframe::egui;
use platform::HardwareControl;

#[cfg(target_os = "windows")]
use platform::windows::{ComGuard, WindowsHardwareControl};

fn main() -> Result<(), eframe::Error> {
    // 初始化 COM（Windows 平台）
    #[cfg(target_os = "windows")]
    let _com = ComGuard::new().expect("Failed to initialize COM");

    // 创建硬件控制实例
    #[cfg(target_os = "windows")]
    let hw_control: Box<dyn HardwareControl> = Box::new(
        WindowsHardwareControl::new().expect("Failed to initialize hardware control")
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
            .with_title("Lecoo Mini Pro 控制中心"),
        ..Default::default()
    };

    eframe::run_native(
        "Lecoo Control Center",
        options,
        Box::new(|cc| Ok(Box::new(ui::ControlCenterApp::new(cc, hw_control)))),
    )
}
