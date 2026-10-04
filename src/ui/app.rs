use super::{
    native::{self, ControllerWindow, NativeWindow},
    preferences::{self, CloseBehavior},
};
use crate::core::cooling::{CoolingIo, FanTarget};
use crate::core::{PowerMode, SystemMonitor};
use crate::platform::windows::{
    desktop::{show_error, Action, Desktop},
    fan_session::FanClient,
    instance::Instance,
    startup::{self, StartupStatus},
    WindowsHardwareControl,
};
use crate::platform::HardwareControl;
use anyhow::Result;
use lecoo_control_center::localization::{self, text as tr, Language};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::{
        Controls::{TB_ENDTRACK, TB_THUMBPOSITION, TB_THUMBTRACK},
        WindowsAndMessaging::*,
    },
};

pub struct ControlCenterApp {
    monitor: SystemMonitor,
    last_error: Option<String>,
    desktop: Desktop,
    instance: Instance,
    close_behavior: CloseBehavior,
    exiting: bool,
    startup_status: Option<StartupStatus>,
    startup_pending: Option<Receiver<anyhow::Result<StartupStatus>>>,
    startup_checked: Instant,
    theme_checked: Instant,
    resolved_english: bool,
    slider_pending: Option<(Instant, u8)>,
    pending_fan_target: Option<FanTarget>,
    fan: Option<FanClient>,
    fan_ready: bool,
    fan_started: bool,
    fan_target: u8,
    manual_percent: u8,
    fan_notice: String,
    smoke_report: Option<PathBuf>,
    smoke_started: Instant,
    smoke_phase: u8,
    open_requests: u32,
    windows_created: u32,
    windows_destroyed: u32,
    window: Option<NativeWindow>,
    dialog: Option<NativeWindow>,
    about: Option<NativeWindow>,
    notices: Option<NativeWindow>,
}
impl ControlCenterApp {
    pub fn run(
        hw: Box<dyn HardwareControl>,
        instance: Instance,
        minimized: bool,
        smoke_report: Option<PathBuf>,
    ) -> Result<()> {
        let controller = ControllerWindow::new()?;
        let mut app = Self {
            monitor: SystemMonitor::new(hw),
            last_error: None,
            desktop: Desktop::new()?,
            instance,
            close_behavior: preferences::load(),
            exiting: false,
            startup_status: None,
            startup_pending: None,
            startup_checked: Instant::now(),
            theme_checked: Instant::now(),
            resolved_english: localization::is_english(),
            slider_pending: None,
            pending_fan_target: None,
            fan: None,
            fan_ready: false,
            fan_started: false,
            fan_target: 101,
            manual_percent: 50,
            fan_notice: String::new(),
            smoke_report,
            smoke_started: Instant::now(),
            smoke_phase: 0,
            open_requests: 0,
            windows_created: 0,
            windows_destroyed: 0,
            window: None,
            dialog: None,
            about: None,
            notices: None,
        };
        app.startup_operation("status");
        if !minimized || !app.desktop.is_available() {
            app.open()?;
        }
        app.tick();
        unsafe {
            let mut message = MSG::default();
            while !app.exiting {
                let result = GetMessageW(&mut message, None, 0, 0).0;
                if result == -1 {
                    return Err(windows::core::Error::from_win32().into());
                }
                if result == 0 {
                    break;
                }
                if message.hwnd == controller.0 {
                    match message.message {
                        WM_TIMER => app.tick(),
                        native::COMMAND => app.command((message.wParam.0 & 0xffff) as u16),
                        native::CLOSE => app.close_request(HWND(message.wParam.0 as *mut _)),
                        native::HIDE => app.hide(),
                        native::EXIT => app.exiting = true,
                        native::LAYOUT => {
                            if let Some(ui) = [&app.window, &app.dialog, &app.about, &app.notices]
                                .into_iter()
                                .flatten()
                                .find(|ui| ui.hwnd.0 as usize == message.wParam.0)
                            {
                                ui.layout();
                            }
                        }
                        native::DPI => {
                            for ui in [&app.window, &app.dialog, &app.about, &app.notices]
                                .into_iter()
                                .flatten()
                                .filter(|ui| ui.hwnd.0 as usize == message.wParam.0)
                            {
                                if let Err(error) = ui.update_font() {
                                    app.last_error = Some(format!("{error:#}"));
                                }
                                ui.layout();
                            }
                        }
                        native::THEME => app.refresh_theme(),
                        native::SLIDER => app.slider(
                            (message.wParam.0 & 0xffff) as u32,
                            HWND(message.lParam.0 as *mut _),
                        ),
                        _ => {
                            let _ = TranslateMessage(&message);
                            DispatchMessageW(&message);
                        }
                    }
                } else {
                    let active = app
                        .notices
                        .as_ref()
                        .or(app.about.as_ref())
                        .or(app.dialog.as_ref())
                        .or(app.window.as_ref());
                    if !active.is_some_and(|ui| IsDialogMessageW(ui.hwnd, &message).as_bool()) {
                        let _ = TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            }
        }
        app.stop_fan();
        app.dialog.take();
        app.window.take();
        Ok(())
    }
    fn open(&mut self) -> Result<()> {
        if self.window.is_none() {
            self.window = Some(NativeWindow::new(self.manual_percent)?);
            self.windows_created += 1;
        }
        if let Some(ui) = &self.window {
            ui.show();
        }
        if let Some(active) = self
            .notices
            .as_ref()
            .or(self.about.as_ref())
            .or(self.dialog.as_ref())
        {
            active.show();
        }
        self.startup_operation("status");
        self.refresh_theme();
        self.render();
        Ok(())
    }
    fn hide(&mut self) {
        if self.desktop.is_available() {
            self.notices.take();
            self.about.take();
            self.dialog.take();
            if self.window.take().is_some() {
                self.windows_destroyed += 1;
            }
        } else {
            self.last_error = Some("托盘图标不可用，保留主窗口".to_owned());
            let _ = self.open();
        }
    }
    fn close_request(&mut self, hwnd: HWND) {
        if self.notices.as_ref().is_some_and(|ui| ui.hwnd == hwnd) {
            self.notices.take();
            return;
        }
        if self.about.as_ref().is_some_and(|ui| ui.hwnd == hwnd) {
            self.notices.take();
            self.about.take();
            return;
        }
        if self
            .dialog
            .as_ref()
            .is_some_and(|dialog| dialog.hwnd == hwnd)
        {
            self.cancel_dialog();
            return;
        }
        if !self.window.as_ref().is_some_and(|ui| ui.hwnd == hwnd) {
            return;
        }
        match self.close_behavior {
            CloseBehavior::Tray => self.hide(),
            CloseBehavior::Exit => self.exiting = true,
            CloseBehavior::Ask => {
                if self.dialog.is_none() {
                    match NativeWindow::close_dialog(hwnd) {
                        Ok(dialog) => self.dialog = Some(dialog),
                        Err(error) => {
                            self.last_error = Some(format!("关闭窗口对话框失败: {error:#}"))
                        }
                    }
                }
            }
        }
    }
    fn cancel_dialog(&mut self) {
        if self.notices.take().is_some() {
            return;
        }
        if self.about.take().is_some() {
            return;
        }
        self.dialog.take();
        if let Some(ui) = &self.window {
            ui.show();
        }
    }
    fn dialog_choice(&mut self, behavior: CloseBehavior) {
        let remember = self
            .dialog
            .as_ref()
            .is_some_and(|ui| ui.checked(native::REMEMBER));
        if remember {
            match preferences::save(behavior) {
                Ok(()) => self.close_behavior = behavior,
                Err(error) => self.last_error = Some(format!("保存关闭选择失败: {error:#}")),
            }
        }
        self.dialog.take();
        match behavior {
            CloseBehavior::Tray => self.hide(),
            CloseBehavior::Exit => self.exiting = true,
            CloseBehavior::Ask => {}
        }
    }
    fn command(&mut self, id: u16) {
        match id {
            native::ABOUT => {
                if self.about.is_none() {
                    if let Some(owner) = self.dialog.as_ref().filter(|ui| ui.is_settings()) {
                        match NativeWindow::about_dialog(owner.hwnd, false) {
                            Ok(ui) => self.about = Some(ui),
                            Err(error) => self.last_error = Some(format!("{error:#}")),
                        }
                    }
                }
            }
            native::NOTICES => {
                if self.notices.is_none() {
                    if let Some(owner) = &self.about {
                        match NativeWindow::about_dialog(owner.hwnd, true) {
                            Ok(ui) => self.notices = Some(ui),
                            Err(error) => self.last_error = Some(format!("{error:#}")),
                        }
                    }
                }
            }
            native::BILIBILI | native::PROJECT => {
                let url = if id == native::BILIBILI {
                    "https://space.bilibili.com/404899?spm_id_from=333.1365.0.0"
                } else {
                    "https://github.com/BlackSquarre/lecoo-rust-powercontrol"
                };
                unsafe {
                    let owner = self.about.as_ref().map(|ui| ui.hwnd).unwrap_or_default();
                    let result = windows::Win32::UI::Shell::ShellExecuteW(
                        owner,
                        windows::core::w!("open"),
                        &windows::core::HSTRING::from(url),
                        None,
                        None,
                        SW_SHOWNORMAL,
                    );
                    if result.0 as isize <= 32 {
                        show_error(tr(
                            "无法打开链接，请检查默认浏览器设置",
                            "Unable to open the link. Check your default browser settings.",
                        ));
                    }
                }
            }
            native::SETTINGS => {
                if self.dialog.is_none() {
                    if let Some(ui) = &self.window {
                        match NativeWindow::settings_dialog(ui.hwnd) {
                            Ok(dialog) => self.dialog = Some(dialog),
                            Err(error) => {
                                self.last_error = Some(format!(
                                    "{}: {error:#}",
                                    tr("无法打开设置", "Unable to open settings")
                                ))
                            }
                        }
                    }
                    self.startup_operation("status");
                }
            }
            native::LANGUAGE => {
                if let Some(dialog) = self.dialog.as_ref().filter(|ui| ui.is_settings()) {
                    let language = Language::from_index(dialog.combo_index(native::LANGUAGE));
                    match preferences::save_all(self.close_behavior, language) {
                        Ok(()) => {
                            localization::set_language(language);
                            self.resolved_english = localization::is_english();
                            for ui in [&self.window, &self.dialog, &self.about, &self.notices]
                                .into_iter()
                                .flatten()
                            {
                                ui.retranslate();
                            }
                            self.desktop.retranslate();
                        }
                        Err(error) => {
                            self.last_error = Some(format!(
                                "{}: {error:#}",
                                tr("无法保存语言设置", "Unable to save language preference")
                            ))
                        }
                    }
                }
            }
            native::CLOSE_BEHAVIOR => {
                if let Some(dialog) = self.dialog.as_ref().filter(|ui| ui.is_settings()) {
                    let behavior = match dialog.combo_index(native::CLOSE_BEHAVIOR) {
                        1 => CloseBehavior::Tray,
                        2 => CloseBehavior::Exit,
                        _ => CloseBehavior::Ask,
                    };
                    match preferences::save(behavior) {
                        Ok(()) => self.close_behavior = behavior,
                        Err(error) => {
                            self.last_error = Some(format!(
                                "{}: {error:#}",
                                tr("无法保存关闭选择", "Unable to save close preference")
                            ))
                        }
                    }
                }
            }
            native::QUIET => self.set_power_mode(PowerMode::Quiet),
            native::BALANCE => self.set_power_mode(PowerMode::Balance),
            native::PERFORMANCE => self.set_power_mode(PowerMode::Performance),
            native::AUTO => {
                self.slider_pending = None;
                self.pending_fan_target = None;
                self.request_fan(FanTarget::Auto);
            }
            native::MAXIMUM => {
                self.slider_pending = None;
                self.submit_manual(100);
            }
            native::MANUAL => self.submit_manual(self.manual_percent),
            native::RECONNECT => {
                self.slider_pending = None;
                self.stop_fan();
                self.fan_started = false;
                self.request_fan(FanTarget::Auto);
            }
            native::STARTUP if self.startup_pending.is_none() && self.startup_status.is_some() => {
                self.startup_operation(if self.startup_status == Some(StartupStatus::Enabled) {
                    "disable"
                } else {
                    "enable"
                });
            }
            native::REMEMBER => {
                if let Some(ui) = &self.dialog {
                    ui.check(native::REMEMBER, !ui.checked(native::REMEMBER));
                }
            }
            native::DIALOG_TRAY => self.dialog_choice(CloseBehavior::Tray),
            native::DIALOG_EXIT => self.dialog_choice(CloseBehavior::Exit),
            1 => {
                // IsDialogMessage routes Enter to IDOK when there is no dialog template.
                let id = unsafe {
                    GetDlgCtrlID(windows::Win32::UI::Input::KeyboardAndMouse::GetFocus())
                } as u16;
                if self.dialog.as_ref().is_some_and(|ui| !ui.is_settings()) {
                    self.dialog_choice(if id == native::DIALOG_EXIT {
                        CloseBehavior::Exit
                    } else {
                        CloseBehavior::Tray
                    });
                } else if [
                    native::QUIET,
                    native::BALANCE,
                    native::PERFORMANCE,
                    native::AUTO,
                    native::MANUAL,
                    native::MAXIMUM,
                    native::STARTUP,
                    native::RECONNECT,
                    native::SETTINGS,
                    native::ABOUT,
                    native::BILIBILI,
                    native::PROJECT,
                    native::NOTICES,
                ]
                .contains(&id)
                {
                    self.command(id);
                }
            }
            native::DIALOG_CANCEL | 2 => self.cancel_dialog(),
            _ => {}
        }
        self.render();
    }
    fn slider(&mut self, code: u32, source: HWND) {
        let Some(ui) = self
            .window
            .as_ref()
            .filter(|ui| ui.handle(native::TARGET) == source)
        else {
            return;
        };
        self.manual_percent = ui.percent();
        if code == TB_THUMBTRACK {
            self.slider_pending = None;
        } else if code == TB_ENDTRACK || code == TB_THUMBPOSITION {
            self.slider_pending = None;
            self.submit_manual(self.manual_percent);
        } else {
            self.slider_pending = Some((Instant::now(), self.manual_percent));
        }
        self.render();
    }
    fn submit_manual(&mut self, percent: u8) {
        if percent != 100 {
            self.last_error =
                Some("ACPI 热区尚未验证为 CPU 保护温度，手动风扇目标暂不可用".to_owned());
            return;
        }
        match FanTarget::percent(percent) {
            Ok(target) => {
                if !self.fan_started
                    || self.fan_target != percent
                    || self.pending_fan_target.is_some()
                {
                    self.request_fan(target);
                }
            }
            Err(error) => self.last_error = Some(format!("{error:#}")),
        }
    }
    fn desktop_action(&mut self, action: Action) {
        match action {
            Action::Mode(mode) => {
                self.set_power_mode(mode);
                if let Some(error) = &self.last_error {
                    show_error(&localization::user_error(error));
                }
            }
            Action::Open => {
                if let Err(error) = self.open() {
                    show_error(&localization::user_error(&format!(
                        "打开窗口失败: {error:#}"
                    )));
                }
            }
            Action::Hide => self.hide(),
            Action::Exit => self.exiting = true,
        }
    }
    fn refresh_theme(&self) {
        for ui in [&self.window, &self.dialog, &self.about, &self.notices]
            .into_iter()
            .flatten()
        {
            ui.refresh_theme();
        }
    }
    fn tick(&mut self) {
        if let Some(result) = self
            .startup_pending
            .as_ref()
            .and_then(|pending| pending.try_recv().ok())
        {
            self.startup_pending = None;
            self.startup_checked = Instant::now();
            match result {
                Ok(status) => self.startup_status = Some(status),
                Err(error) => self.last_error = Some(format!("登录启动操作失败: {error:#}")),
            }
        }
        if self.window.is_some() && self.startup_checked.elapsed() >= Duration::from_secs(30) {
            self.startup_operation("status");
            self.startup_checked = Instant::now();
        }
        if self.theme_checked.elapsed() >= Duration::from_secs(2) {
            self.refresh_theme();
            if localization::is_english() != self.resolved_english {
                self.resolved_english = localization::is_english();
                for ui in [&self.window, &self.dialog, &self.about, &self.notices]
                    .into_iter()
                    .flatten()
                {
                    ui.retranslate();
                }
                self.desktop.retranslate();
            }
            self.theme_checked = Instant::now();
        }
        if self.instance.take_open_request() {
            self.open_requests += 1;
            self.desktop_action(Action::Open);
        }
        for action in self.desktop.actions() {
            self.desktop_action(action);
        }
        if self
            .slider_pending
            .is_some_and(|(at, _)| at.elapsed() >= Duration::from_millis(350))
        {
            let (_, value) = self.slider_pending.take().unwrap();
            self.submit_manual(value);
        }
        self.fan_tick();
        let snapshot = self.monitor.update();
        self.desktop.refresh(snapshot.power_mode);
        if self.window.is_none() && !self.desktop.is_available() {
            self.last_error = Some("托盘连接失效，已重新打开主窗口".to_owned());
            self.desktop_action(Action::Open);
        }
        self.smoke_tick();
        self.render();
    }
    fn render(&mut self) {
        if let Some(ui) = &self.about {
            ui.refresh_year();
        }
        let snapshot = self.monitor.update();
        self.desktop.refresh(snapshot.power_mode);
        for ui in [&self.window, &self.dialog, &self.about, &self.notices]
            .into_iter()
            .flatten()
        {
            if let Err(error) = ui.icon_mode(snapshot.power_mode) {
                self.last_error = Some(format!("{error:#}"));
            }
        }
        let Some(ui) = &self.window else {
            return;
        };
        ui.text(
            native::THERMAL_ZONE,
            snapshot
                .thermal_zone_temp
                .map(|v| format!("{v:.1} °C"))
                .unwrap_or_else(|| tr("不可用", "Unavailable").into()),
        );
        ui.text(
            native::RPM,
            snapshot
                .fan_speed
                .map(|v| format!("{v} RPM"))
                .unwrap_or_else(|| tr("不可用", "Unavailable").into()),
        );
        let gb = |v: u64| v as f64 / 1073741824.0;
        ui.text(
            native::MEMORY,
            format!(
                "{:.0}% · {:.1}/{:.1} GiB",
                snapshot.mem_usage,
                gb(snapshot.mem_used),
                gb(snapshot.mem_total)
            ),
        );
        ui.text(
            native::DISK,
            format!(
                "{:.0}% · {:.0}/{:.0} GiB",
                snapshot.disk_usage,
                gb(snapshot.disk_used),
                gb(snapshot.disk_total)
            ),
        );
        ui.progress(native::MEMORY_BAR, snapshot.mem_usage);
        ui.progress(native::DISK_BAR, snapshot.disk_usage);
        ui.mode(snapshot.power_mode.map(|m| m as u8));
        for (id, mode) in [
            (native::QUIET, PowerMode::Quiet),
            (native::BALANCE, PowerMode::Balance),
            (native::PERFORMANCE, PowerMode::Performance),
        ] {
            ui.check(id, snapshot.power_mode == Some(mode));
        }
        ui.check(native::AUTO, self.fan_target == 101);
        ui.check(native::MAXIMUM, self.fan_target == 100);
        ui.check(native::MANUAL, (35..100).contains(&self.fan_target));
        let adjustable = !self.fan_started || self.fan_ready;
        ui.enable(native::MANUAL, false);
        ui.enable(native::MAXIMUM, adjustable);
        ui.enable(native::TARGET, false);
        ui.text(native::PERCENT, "—".to_owned());
        ui.fan_state(self.fan_target, self.fan_ready);
        if let Some(dialog) = self.dialog.as_ref().filter(|ui| ui.is_settings()) {
            dialog.check(
                native::STARTUP,
                self.startup_status == Some(StartupStatus::Enabled),
            );
            dialog.enable(
                native::STARTUP,
                self.startup_pending.is_none() && self.startup_status.is_some(),
            );
            dialog.select(
                native::CLOSE_BEHAVIOR,
                match self.close_behavior {
                    CloseBehavior::Ask => 0,
                    CloseBehavior::Tray => 1,
                    CloseBehavior::Exit => 2,
                },
            );
            dialog.select(native::LANGUAGE, localization::language().index());
        }
        let error = self
            .last_error
            .as_ref()
            .or(snapshot.power_mode_error.as_ref());
        let message = error
            .map(|error| localization::user_error(error))
            .unwrap_or_default();
        if let Some(dialog) = self.dialog.as_ref().filter(|ui| ui.is_settings()) {
            dialog.text(native::ERROR, message.clone());
        }
        ui.text(native::ERROR, message);
    }
    fn smoke_tick(&mut self) {
        if self.smoke_report.is_none() {
            return;
        }
        let seconds = self.smoke_started.elapsed().as_secs();
        if self.smoke_phase == 0 && seconds >= 1 {
            self.smoke_phase = 1;
            self.close_behavior = CloseBehavior::Tray;
            if let Some(ui) = &self.window {
                unsafe {
                    let _ = PostMessageW(ui.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                }
            }
        }
        if self.smoke_phase == 1 && seconds >= 4 {
            self.smoke_phase = 2;
            self.desktop_action(Action::Open);
        }
        if self.smoke_phase == 2 && seconds >= 5 {
            self.smoke_phase = 3;
            self.command(native::MAXIMUM);
        }
        if self.smoke_phase == 3 && seconds >= 6 && self.fan_ready {
            self.smoke_phase = 4;
        }
        if self.smoke_phase == 4 && seconds >= 8 {
            self.smoke_phase = 5;
            self.desktop_action(Action::Hide);
        }
        let visible = self
            .window
            .as_ref()
            .is_some_and(|ui| unsafe { IsWindowVisible(ui.hwnd).as_bool() });
        let mode = self
            .monitor
            .update()
            .power_mode
            .map(|m| m as u8)
            .unwrap_or(255);
        let report=format!("{{\"Backend\":\"Win32\",\"TrayRegistered\":{},\"Hidden\":{},\"WindowVisible\":{},\"OpenRequests\":{},\"FanReady\":{},\"FanTarget\":{},\"Phase\":{},\"Seconds\":{},\"PowerMode\":{},\"WindowsCreated\":{},\"WindowsDestroyed\":{},\"ControlCount\":{},\"Dark\":{}}}",
            self.desktop.is_available(),self.window.is_none(),visible,self.open_requests,self.fan_ready,self.fan_target,self.smoke_phase,seconds,mode,self.windows_created,self.windows_destroyed,
            self.window.as_ref().map(|ui|ui.count()).unwrap_or(0),self.window.as_ref().is_some_and(|ui|ui.dark()));
        let _ = std::fs::write(self.smoke_report.as_ref().unwrap(), report);
        if seconds >= 13 {
            self.exiting = true;
        }
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
        if target == FanTarget::Auto && self.fan_started && !self.fan_ready {
            self.stop_fan();
            self.fan_started = false;
        }
        if self.fan.is_none() && !self.fan_started {
            self.fan_started = true;
            match FanClient::start() {
                Ok(fan) => self.fan = Some(fan),
                Err(error) => {
                    self.last_error = Some(format!("风扇连接失败: {error:#}"));
                    return;
                }
            }
        }
        if let Some(fan) = &mut self.fan {
            if self.fan_ready || target == FanTarget::Auto {
                self.pending_fan_target = None;
                if let Err(error) = fan.send(target) {
                    self.last_error = Some(format!("风扇请求失败: {error:#}"));
                }
            } else {
                self.pending_fan_target = Some(target);
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
        self.pending_fan_target = None;
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
    fn fan_tick(&mut self) {
        let result = self.fan.as_mut().map(FanClient::tick);
        match result {
            Some(Ok(lines)) => {
                for line in lines {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    match fields.first().copied() {
                        Some("READY") => {
                            self.fan_ready = true;
                            self.fan_notice = "连接正常".to_owned();
                            if let Some(target) = self.pending_fan_target.take() {
                                self.request_fan(target);
                            }
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
                            self.pending_fan_target = None;
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
}
impl Drop for ControlCenterApp {
    fn drop(&mut self) {
        self.stop_fan();
        self.notices.take();
        self.about.take();
        self.dialog.take();
        self.window.take();
    }
}
