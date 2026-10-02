use crate::core::cooling::{CoolingIo, FanTarget};
use crate::core::{PowerMode, SystemMonitor};
use crate::platform::windows::gui_timer::GuiTimer;
use crate::platform::windows::{
    desktop::{show_error, Action, Desktop},
    fan_session::FanClient,
    instance::Instance,
    startup::{self, StartupStatus},
    WindowsHardwareControl,
};
use crate::platform::HardwareControl;
use crate::ui::theme::{apply_lecoo_theme, LecooColors};
use crate::ui::widgets::{circular_gauge, disk_gauge, memory_gauge, FanGauge};
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{cell::RefCell, rc::Rc};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    time::{Duration, Instant},
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        IsWindowVisible, PostMessageW, SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE,
        SW_SHOW, WM_CLOSE,
    },
};

pub struct NativeApp {
    _timer: GuiTimer,
    inner: Rc<RefCell<ControlCenterApp>>,
}
impl eframe::App for NativeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(mut app) = self.inner.try_borrow_mut() {
            app.update(ctx, frame);
        } else {
            ctx.request_repaint();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Mode,     // 模式设置
    Fan,      // 风扇控制
    Settings, // 功能设定
    Info,     // 常规信息
}

pub struct ControlCenterApp {
    monitor: SystemMonitor,
    current_page: Page,
    fan_gauge: FanGauge,
    colors: LecooColors,
    last_error: Option<String>,
    desktop: Desktop,
    instance: Instance,
    close_to_tray: bool,
    hidden: bool,
    exiting: bool,
    initial_minimized: bool,
    startup_status: Option<StartupStatus>,
    startup_pending: Option<Receiver<anyhow::Result<StartupStatus>>>,
    fan: Option<FanClient>,
    fan_ready: bool,
    fan_started: bool,
    fan_target: u8,
    manual_percent: u8,
    fan_notice: String,
    ticker_stop: Arc<AtomicBool>,
    smoke_report: Option<PathBuf>,
    smoke_started: Instant,
    smoke_phase: u8,
    open_requests: u32,
    window: HWND,
}

impl ControlCenterApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        hw_control: Box<dyn HardwareControl>,
        instance: Instance,
        minimized: bool,
        smoke_report: Option<PathBuf>,
    ) -> anyhow::Result<NativeApp> {
        apply_lecoo_theme(&_cc.egui_ctx);
        let desktop = Desktop::new(&_cc.egui_ctx)?;
        let window = match _cc.window_handle()?.as_raw() {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as *mut _),
            _ => anyhow::bail!("需要 Windows 主窗口"),
        };
        let ticker_stop = Arc::new(AtomicBool::new(false));
        let stop = ticker_stop.clone();
        let ctx = _cc.egui_ctx.clone();
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                ctx.request_repaint();
                std::thread::sleep(Duration::from_millis(500));
            }
        });
        let mut app = Self {
            monitor: SystemMonitor::new(hw_control),
            current_page: Page::Mode,
            fan_gauge: FanGauge::new(),
            colors: LecooColors::default(),
            last_error: None,
            desktop,
            instance,
            close_to_tray: super::preferences::close_to_tray(),
            hidden: false,
            exiting: false,
            initial_minimized: minimized,
            startup_status: None,
            startup_pending: None,
            fan: None,
            fan_ready: false,
            fan_started: false,
            fan_target: 101,
            manual_percent: 50,
            fan_notice: "尚未启动控制会话".to_owned(),
            ticker_stop,
            smoke_report,
            smoke_started: Instant::now(),
            smoke_phase: 0,
            open_requests: 0,
            window,
        };
        app.startup_operation("status");
        let inner = Rc::new(RefCell::new(app));
        let weak = Rc::downgrade(&inner);
        let ctx = _cc.egui_ctx.clone();
        let timer = GuiTimer::new(move || {
            if let Some(inner) = weak.upgrade() {
                if let Ok(mut app) = inner.try_borrow_mut() {
                    app.background_tick(&ctx);
                }
            }
        })?;
        Ok(NativeApp {
            _timer: timer,
            inner,
        })
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // Logo 占位
            ui.heading(
                egui::RichText::new("Lecoo")
                    .size(24.0)
                    .color(self.colors.accent),
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

        let button =
            egui::Button::new(egui::RichText::new(format!("{} {}", icon, label)).size(16.0))
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
                ui.heading("CPU温度");
                ui.label(egui::RichText::new("不可用").color(Color32::GRAY));
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
                    (
                        PowerMode::Performance,
                        "性能模式",
                        self.colors.mode_performance,
                    ),
                ];

                for (mode, label, color) in modes {
                    let is_current = snapshot.power_mode == Some(mode);

                    let button = egui::Button::new(egui::RichText::new(label).size(18.0).color(
                        if is_current {
                            Color32::WHITE
                        } else {
                            Color32::from_gray(180)
                        },
                    ))
                    .min_size(egui::vec2(150.0, 50.0))
                    .fill(if is_current {
                        color
                    } else {
                        self.colors.bg_card
                    });

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
            if let Some(error) = self
                .last_error
                .as_ref()
                .or(snapshot.power_mode_error.as_ref())
            {
                ui.add_space(20.0);
                ui.label(egui::RichText::new(error).color(Color32::from_rgb(255, 100, 100)));
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
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.fan.is_some(), egui::Button::new("自动"))
                    .clicked()
                {
                    self.request_fan(FanTarget::Auto);
                }
                if ui
                    .add_enabled(self.fan_ready, egui::Button::new("最大"))
                    .clicked()
                {
                    self.request_fan(FanTarget::Percent(100));
                }
            });
            ui.add(egui::Slider::new(&mut self.manual_percent, 35..=100).text("目标 %"));
            if ui
                .add_enabled(self.fan_ready, egui::Button::new("应用手动目标"))
                .clicked()
            {
                self.request_fan(FanTarget::Percent(self.manual_percent));
            }
            ui.label(if self.fan_target == 101 {
                "本会话请求：自动".to_owned()
            } else {
                format!("本会话请求：{}%", self.fan_target)
            });
            ui.label(&self.fan_notice);
            ui.label("固件只返回 RPM，无法读回实际占空比或控制模式。");
            ui.label("手动控制需要有效 CPU 温度；达到 85°C 或心跳中断会恢复自动。");
            if !self.fan_ready && ui.button("重新连接").clicked() {
                self.stop_fan();
                self.fan_started = false;
            }
        });
    }

    fn render_settings_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(40.0);
            ui.heading("功能设定");
            ui.add_space(20.0);

            let mut enabled = self.startup_status == Some(StartupStatus::Enabled);
            if ui
                .add_enabled(
                    self.startup_pending.is_none() && self.startup_status.is_some(),
                    egui::Checkbox::new(&mut enabled, "当前用户登录后启动（管理员权限，进入托盘）"),
                )
                .changed()
            {
                self.startup_operation(if enabled { "enable" } else { "disable" });
            }
            ui.label(match self.startup_status {
                None => "正在查询登录启动状态…",
                Some(StartupStatus::Disabled) => "登录启动：已关闭",
                Some(StartupStatus::Enabled) => "登录启动：已启用",
                Some(StartupStatus::Stale) => "登录启动：条目失效，请重新启用或关闭",
            });
            if ui
                .add_enabled(
                    self.startup_pending.is_none(),
                    egui::Button::new("刷新启动状态"),
                )
                .clicked()
            {
                self.startup_operation("status");
            }
            ui.label("使用当前用户的计划任务，不保存密码；应用首次启动会显示 Windows 管理员提示。");
            ui.add_space(20.0);
            if ui
                .checkbox(&mut self.close_to_tray, "关闭主窗口时缩到托盘")
                .changed()
            {
                if let Err(error) = super::preferences::save(self.close_to_tray) {
                    self.last_error = Some(format!("保存设置失败: {error:#}"));
                }
            }
            ui.label("默认关闭窗口即退出；托盘菜单始终可以退出。双击图标可打开主窗口。");
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
                    ui.label(format!(
                        "使用率: {:.0}%, 可用/共享: {:.0}G/{:.0}G",
                        snapshot.disk_usage,
                        disk_total - disk_used,
                        disk_total
                    ));
                    ui.end_row();

                    ui.label("内存信息");
                    let mem_used = snapshot.mem_used as f64 / 1_073_741_824.0;
                    let mem_total = snapshot.mem_total as f64 / 1_073_741_824.0;
                    ui.label(format!(
                        "使用率: {:.0}%, 已用/共享: {:.1}G/{:.1}G",
                        snapshot.mem_usage, mem_used, mem_total
                    ));
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
                Err(e) if e.to_string().contains("管理员") || e.to_string().contains("权限") =>
                {
                    self.last_error = Some("需要管理员权限才能切换性能模式".to_string());
                }
                Err(e) => {
                    self.last_error = Some(format!("操作失败: {:#}", e));
                }
            }
        }
        self.monitor.invalidate();
    }

    fn startup_operation(&mut self, operation: &'static str) {
        if self.startup_pending.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.startup_pending = Some(receiver);
        std::thread::spawn(move || {
            let result = std::env::current_exe()
                .map_err(anyhow::Error::from)
                .and_then(|exe| startup::configure(operation, &exe));
            let _ = sender.send(result);
        });
    }
    fn request_fan(&mut self, target: FanTarget) {
        if let Some(fan) = &mut self.fan {
            if let Err(error) = fan.send(target) {
                self.last_error = Some(format!("风扇请求失败: {error:#}"));
            }
        }
    }
    fn stop_fan(&mut self) {
        if let Some(mut fan) = self.fan.take() {
            match fan.stop() {
                Ok(()) => self.fan_target = 101,
                Err(error) => self.recover_fan(&format!("{error:#}")),
            }
        }
        self.fan_ready = false;
    }
    fn recover_fan(&mut self, reason: &str) {
        // A worker crash closes the pipe; the GUI makes an independent best-effort recovery.
        let recovery =
            WindowsHardwareControl::new().and_then(|mut hw| hw.write_fan(FanTarget::Auto));
        self.last_error = Some(format!("{reason}；备用自动恢复: {recovery:?}"));
        self.fan_ready = false;
        if recovery.is_ok() {
            self.fan_target = 101;
        }
    }
    fn fan_tick(&mut self, ctx: &egui::Context) {
        if self.current_page == Page::Fan && !self.fan_started {
            self.fan_started = true;
            match FanClient::start(ctx) {
                Ok(fan) => self.fan = Some(fan),
                Err(error) => self.last_error = Some(format!("{error:#}")),
            }
        }
        let result = self.fan.as_mut().map(FanClient::tick);
        match result {
            Some(Ok(lines)) => {
                for line in lines {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    match fields.first().copied() {
                        Some("READY") => {
                            self.fan_ready = true;
                            self.fan_notice = "已连接温度采样与风扇固件".to_owned();
                        }
                        Some("DATA") => {
                            if let Some(target) = fields.get(3).and_then(|v| v.parse().ok()) {
                                self.fan_target = target;
                            }
                        }
                        Some("OK") => {
                            self.fan_target =
                                fields.get(1).and_then(|v| v.parse().ok()).unwrap_or(101);
                            self.fan_notice = format!(
                                "固件响应 {}；实测 {} RPM",
                                fields.get(2).unwrap_or(&"?"),
                                fields.get(3).unwrap_or(&"?")
                            );
                            self.last_error = None;
                        }
                        Some("AUTO") => self.fan_target = 101,
                        Some("UNAVAILABLE") => {
                            self.fan_ready = false;
                            self.fan_notice = line;
                        }
                        Some("RECOVERY") => {
                            self.fan_target =
                                fields.get(1).and_then(|v| v.parse().ok()).unwrap_or(101);
                            self.last_error = Some(line);
                        }
                        Some("ERROR") => self.last_error = Some(line),
                        Some("DIED") => {
                            self.fan.take();
                            self.recover_fan(&line);
                        }
                        _ => {}
                    }
                }
            }
            Some(Err(error)) => {
                self.stop_fan();
                self.recover_fan(&format!("风扇通信中断: {error:#}"));
            }
            None => {}
        }
    }
    fn desktop_action(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::Mode(mode) => {
                self.set_power_mode(mode);
                if let Some(error) = &self.last_error {
                    show_error(error);
                }
                self.desktop.refresh(self.monitor.update().power_mode);
            }
            Action::Open => {
                self.hidden = false;
                // Native visibility wakes the event loop even when no egui frame can run.
                unsafe {
                    let _ = ShowWindow(self.window, SW_SHOW);
                    let _ = ShowWindow(self.window, SW_RESTORE);
                    let _ = SetForegroundWindow(self.window);
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            Action::Hide => {
                if self.desktop.is_available() {
                    self.hidden = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    unsafe {
                        let _ = ShowWindow(self.window, SW_HIDE);
                    }
                } else {
                    self.last_error = Some("托盘图标不可用，保留主窗口".to_owned());
                }
            }
            Action::Exit => {
                self.exiting = true;
                self.stop_fan();
                // eframe must process one final frame to close a hidden viewport.
                self.hidden = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                unsafe {
                    let _ = ShowWindow(self.window, SW_SHOW);
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                unsafe {
                    let _ = PostMessageW(self.window, WM_CLOSE, WPARAM(0), LPARAM(0));
                }
            }
        }
    }
    fn smoke_tick(&mut self, ctx: &egui::Context) {
        if self.smoke_report.is_none() {
            return;
        }
        let seconds = self.smoke_started.elapsed().as_secs();
        if self.smoke_phase == 0 && seconds >= 1 {
            self.smoke_phase = 1;
            // Exercise the real window-close event without persisting a test preference.
            self.close_to_tray = true;
            unsafe {
                let _ = PostMessageW(self.window, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
        if self.smoke_phase == 1 && seconds >= 4 {
            self.smoke_phase = 2;
            self.desktop_action(ctx, Action::Open);
        }
        if self.smoke_phase == 2 && seconds >= 5 {
            self.smoke_phase = 3;
            self.current_page = Page::Fan;
        }
        if self.smoke_phase == 3 && seconds >= 6 && self.fan_ready {
            self.smoke_phase = 4;
            self.request_fan(FanTarget::Percent(50));
        }
        if self.smoke_phase == 4 && seconds >= 8 {
            self.smoke_phase = 5;
            self.desktop_action(ctx, Action::Hide);
        }
        let snapshot = self.monitor.update();
        let report = format!("{{\"TrayRegistered\":{},\"Hidden\":{},\"WindowVisible\":{},\"OpenRequests\":{},\"FanReady\":{},\"FanTarget\":{},\"Phase\":{},\"Seconds\":{},\"PowerMode\":{:?}}}",
            self.desktop.is_available(), self.hidden, unsafe { IsWindowVisible(self.window).as_bool() }, self.open_requests, self.fan_ready, self.fan_target, self.smoke_phase, seconds, snapshot.power_mode.map(|m| m as u8).unwrap_or(255));
        let _ = std::fs::write(self.smoke_report.as_ref().unwrap(), report);
        if seconds >= 13 {
            self.desktop_action(ctx, Action::Exit);
        }
    }
    fn background_tick(&mut self, ctx: &egui::Context) {
        if let Some(result) = self
            .startup_pending
            .as_ref()
            .and_then(|pending| pending.try_recv().ok())
        {
            self.startup_pending = None;
            match result {
                Ok(status) => self.startup_status = Some(status),
                Err(error) => self.last_error = Some(format!("登录启动操作失败: {error:#}")),
            }
        }
        if self.initial_minimized {
            self.initial_minimized = false;
            self.desktop_action(ctx, Action::Hide);
        }
        if self.instance.take_open_request() {
            self.open_requests += 1;
            self.desktop_action(ctx, Action::Open);
        }
        for action in self.desktop.actions() {
            self.desktop_action(ctx, action);
        }
        self.fan_tick(ctx);
        self.smoke_tick(ctx);
        let snapshot = self.monitor.update();
        self.desktop.refresh(snapshot.power_mode);
        if self.hidden && !self.desktop.is_available() {
            self.desktop_action(ctx, Action::Open);
            self.last_error = Some("托盘连接失效，已重新打开主窗口".to_owned());
        }
    }
}

impl eframe::App for ControlCenterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.background_tick(ctx);
        if ctx.input(|input| input.viewport().close_requested()) && !self.exiting {
            if self.close_to_tray && self.desktop.is_available() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.desktop_action(ctx, Action::Hide);
            } else {
                self.exiting = true;
                self.stop_fan();
            }
        }
        // 定期刷新数据
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        // 更新系统快照
        let snapshot = self.monitor.update();
        self.desktop.refresh(snapshot.power_mode);
        if self.hidden && !self.desktop.is_available() {
            self.desktop_action(ctx, Action::Open);
            self.last_error = Some("托盘连接失效，已重新打开主窗口".to_owned());
        }

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
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.current_page != Page::Mode {
                    if let Some(error) = &self.last_error {
                        ui.colored_label(Color32::from_rgb(255, 100, 100), error);
                    }
                }
                match self.current_page {
                    Page::Mode => self.render_mode_page(ui, &snapshot),
                    Page::Fan => self.render_fan_page(ui, &snapshot),
                    Page::Settings => self.render_settings_page(ui),
                    Page::Info => self.render_info_page(ui, &snapshot),
                }
            });
        });
    }
}

impl Drop for ControlCenterApp {
    fn drop(&mut self) {
        self.ticker_stop.store(true, Ordering::Relaxed);
        self.stop_fan();
    }
}

use egui::Color32;
