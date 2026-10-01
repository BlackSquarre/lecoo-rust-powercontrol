use eframe::egui;
use crate::core::{SystemMonitor, PowerMode};
use crate::platform::HardwareControl;
use crate::ui::theme::{apply_lecoo_theme, LecooColors};
use crate::ui::widgets::{circular_gauge, disk_gauge, memory_gauge, FanGauge};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Mode,      // 模式设置
    Fan,       // 风扇控制
    Settings,  // 功能设定
    Info,      // 常规信息
}

pub struct ControlCenterApp {
    monitor: SystemMonitor,
    current_page: Page,
    fan_gauge: FanGauge,
    colors: LecooColors,
    last_error: Option<String>,
}

impl ControlCenterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, hw_control: Box<dyn HardwareControl>) -> Self {
        apply_lecoo_theme(&_cc.egui_ctx);

        Self {
            monitor: SystemMonitor::new(hw_control),
            current_page: Page::Mode,
            fan_gauge: FanGauge::new(),
            colors: LecooColors::default(),
            last_error: None,
        }
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // Logo 占位
            ui.heading(
                egui::RichText::new("Lecoo")
                    .size(24.0)
                    .color(self.colors.accent)
            );

            ui.add_space(40.0);

            // 导航按钮
            self.render_nav_button(ui, "模式设置", "🎛", Page::Mode);
            self.render_nav_button(ui, "风扇控制", "🌀", Page::Fan);
            self.render_nav_button(ui, "功能设定", "⚙", Page::Settings);
            self.render_nav_button(ui, "常规信息", "📋", Page::Info);
        });
    }

    fn render_nav_button(&mut self, ui: &mut egui::Ui, label: &str, icon: &str, page: Page) {
        let is_selected = self.current_page == page;

        let button = egui::Button::new(
            egui::RichText::new(format!("{} {}", icon, label))
                .size(16.0)
        )
        .min_size(egui::vec2(120.0, 40.0))
        .selected(is_selected);

        if ui.add(button).clicked() {
            self.current_page = page;
        }

        ui.add_space(8.0);
    }

    fn render_status_panel(&mut self, ui: &mut egui::Ui, snapshot: &crate::core::SystemSnapshot) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // CPU 温度
            if let Some(temp) = snapshot.cpu_temp {
                circular_gauge(ui, "CPU温度", temp as f32, 100.0, "°C", self.colors.primary);
            } else {
                circular_gauge(ui, "CPU温度", 0.0, 100.0, "°C", Color32::GRAY);
                ui.label(egui::RichText::new("不支持").color(Color32::GRAY));
            }

            ui.add_space(30.0);

            // 磁盘使用率
            let disk_used_gb = snapshot.disk_used as f32 / 1_073_741_824.0;
            let disk_total_gb = snapshot.disk_total as f32 / 1_073_741_824.0;
            disk_gauge(ui, disk_used_gb, disk_total_gb, self.colors.primary);

            ui.add_space(30.0);

            // 内存使用率
            let mem_used_gb = snapshot.mem_used as f32 / 1_073_741_824.0;
            let mem_total_gb = snapshot.mem_total as f32 / 1_073_741_824.0;
            memory_gauge(ui, mem_used_gb, mem_total_gb, self.colors.primary);
        });
    }

    fn render_mode_page(&mut self, ui: &mut egui::Ui, snapshot: &crate::core::SystemSnapshot) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // 性能模式按钮
            ui.horizontal(|ui| {
                ui.add_space(50.0);

                let modes = [
                    (PowerMode::Quiet, "安静模式", self.colors.mode_quiet),
                    (PowerMode::Balance, "均衡模式", self.colors.mode_balance),
                    (PowerMode::Performance, "性能模式", self.colors.mode_performance),
                ];

                for (mode, label, color) in modes {
                    let is_current = snapshot.power_mode == Some(mode);

                    let button = egui::Button::new(
                        egui::RichText::new(label)
                            .size(18.0)
                            .color(if is_current { Color32::WHITE } else { Color32::from_gray(180) })
                    )
                    .min_size(egui::vec2(150.0, 50.0))
                    .fill(if is_current { color } else { self.colors.bg_card });

                    if ui.add(button).clicked() {
                        self.set_power_mode(mode);
                    }
                }
            });

            ui.add_space(40.0);

            // 风扇转速仪表
            if let Some(rpm) = snapshot.fan_speed {
                self.fan_gauge.set_rpm(rpm);
            } else {
                self.fan_gauge.set_rpm(0);
            }
            self.fan_gauge.render(ui);

            // 错误提示
            if let Some(error) = self.last_error.as_ref().or(snapshot.power_mode_error.as_ref()) {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new(error)
                        .color(Color32::from_rgb(255, 100, 100))
                );
            }
        });
    }

    fn render_fan_page(&mut self, ui: &mut egui::Ui, snapshot: &crate::core::SystemSnapshot) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            ui.heading("风扇控制页面");
            ui.add_space(20.0);

            if let Some(rpm) = snapshot.fan_speed {
                self.fan_gauge.set_rpm(rpm);
                self.fan_gauge.render(ui);
            } else {
                ui.label("风扇数据不可用");
            }

            ui.add_space(20.0);
            ui.label(egui::RichText::new("风扇手动控制功能待实现").color(Color32::GRAY));
        });
    }

    fn render_settings_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(40.0);
            ui.heading("功能设定");
            ui.add_space(20.0);

            ui.label("热键图标提示");
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("打开功能，按下热键，会提示功能图标；\n关闭功能，按下热键，不会提示功能图标。")
                    .color(Color32::from_gray(180))
            );

            ui.add_space(20.0);

            // 占位开关
            let mut enabled = true;
            ui.checkbox(&mut enabled, "ON");
        });
    }

    fn render_info_page(&mut self, ui: &mut egui::Ui, snapshot: &crate::core::SystemSnapshot) {
        ui.vertical(|ui| {
            ui.add_space(20.0);
            ui.heading("常规信息");
            ui.add_space(20.0);

            egui::Grid::new("info_grid")
                .num_columns(2)
                .spacing([40.0, 16.0])
                .show(ui, |ui| {
                    ui.label("系统");
                    ui.label("Microsoft Windows 10 IoT 企业版 LTSC");
                    ui.end_row();

                    ui.label("磁盘信息");
                    let disk_used = snapshot.disk_used as f64 / 1_073_741_824.0;
                    let disk_total = snapshot.disk_total as f64 / 1_073_741_824.0;
                    ui.label(format!("使用率: {:.0}%, 可用/共享: {:.0}G/{:.0}G",
                        snapshot.disk_usage, disk_total - disk_used, disk_total));
                    ui.end_row();

                    ui.label("内存信息");
                    let mem_used = snapshot.mem_used as f64 / 1_073_741_824.0;
                    let mem_total = snapshot.mem_total as f64 / 1_073_741_824.0;
                    ui.label(format!("使用率: {:.0}%, 已用/共享: {:.1}G/{:.1}G",
                        snapshot.mem_usage, mem_used, mem_total));
                    ui.end_row();

                    if let Some(temp) = snapshot.cpu_temp {
                        ui.label("CPU温度");
                        ui.label(format!("{}°C", temp));
                        ui.end_row();
                    }

                    if let Some(rpm) = snapshot.fan_speed {
                        ui.label("风扇转速");
                        ui.label(format!("{} RPM", rpm));
                        ui.end_row();
                    }
                });
        });
    }

    fn set_power_mode(&mut self, mode: PowerMode) {
        if let Ok(mut hw) = self.monitor.get_hw_control().lock() {
            match hw.set_power_mode(mode) {
                Ok(()) => {
                    self.last_error = None;
                }
                Err(e) if e.to_string().contains("管理员") || e.to_string().contains("权限") => {
                    self.last_error = Some("需要管理员权限才能切换性能模式".to_string());
                }
                Err(e) => {
                    self.last_error = Some(format!("操作失败: {:#}", e));
                }
            }
        }
    }
}

impl eframe::App for ControlCenterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 定期刷新数据
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        // 更新系统快照
        let snapshot = self.monitor.update();

        // 左侧导航栏
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(150.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        // 右侧状态面板
        egui::SidePanel::right("status")
            .resizable(false)
            .exact_width(200.0)
            .show(ctx, |ui| {
                self.render_status_panel(ui, &snapshot);
            });

        // 中央内容区
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Mode => self.render_mode_page(ui, &snapshot),
                Page::Fan => self.render_fan_page(ui, &snapshot),
                Page::Settings => self.render_settings_page(ui),
                Page::Info => self.render_info_page(ui, &snapshot),
            }
        });
    }
}

use egui::Color32;
