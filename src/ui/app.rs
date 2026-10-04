use super::{
    native::{self, ControllerWindow, NativeWindow},
    preferences::{self, CloseBehavior},
};
use crate::core::cooling::{CoolingController, FanTarget};
use crate::core::{PowerMode, SystemMonitor};
use crate::platform::windows::{
    desktop::{show_error, Action, Desktop},
    fan_session::FanClient,
    instance::Instance,
    startup::{self, StartupStatus},
    wake::{UiWake, WAKE},
    WindowsHardwareControl,
};
use crate::platform::HardwareControl;
use anyhow::Result;
use lecoo_control_center::localization::{self, text as tr, Language, Text};
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
    wake: UiWake,
    render_dirty: bool,
    rendered_revision: u64,
    resolved_language: Language,
    slider_pending: Option<(Instant, u8)>,
    pending_fan_target: Option<FanTarget>,
    fan: Option<FanClient>,
    fan_ready: bool,
    fan_stopping: bool,
    fan_restart: bool,
    fan_target: u8,
    manual_percent: u8,
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
        let wake = UiWake::new(controller.0);
        let mut app = Self {
            monitor: SystemMonitor::new(hw),
            last_error: None,
            desktop: Desktop::with_wake(Some(wake))?,
            instance,
            close_behavior: preferences::load(),
            exiting: false,
            startup_status: None,
            startup_pending: None,
            wake,
            render_dirty: true,
            rendered_revision: 0,
            resolved_language: localization::resolved_language(),
            slider_pending: None,
            pending_fan_target: None,
            fan: None,
            fan_ready: false,
            fan_stopping: false,
            fan_restart: false,
            fan_target: 101,
            manual_percent: 50,
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
        if !minimized || !app.desktop.is_available() {
            app.open()?;
        }
        app.tick();
        unsafe {
            let mut message = MSG::default();
            while !app.exiting {
                controller.set_interval(app.timer_interval())?;
                if native::wait_for_activity(app.instance.open_event())? {
                    app.open_requests += 1;
                    app.desktop_action(Action::Open);
                    continue;
                }
                if !PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                    continue;
                }
                if message.message == WM_QUIT {
                    break;
                }
                if message.hwnd == controller.0 {
                    match message.message {
                        WM_TIMER | WAKE => app.tick(),
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
                            app.render_dirty = true;
                            app.render();
                        }
                        native::THEME => {
                            app.refresh_theme();
                            app.render();
                        }
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
        self.monitor.refresh(true);
        self.desktop.refresh(self.monitor.snapshot().power_mode);
        self.render_dirty = true;
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
            self.monitor.refresh(false);
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
                        show_error(tr(Text::ErrorsOpenLink));
                    }
                }
            }
            native::SETTINGS => {
                if self.dialog.is_none() {
                    if let Some(ui) = &self.window {
                        match NativeWindow::settings_dialog(ui.hwnd) {
                            Ok(dialog) => self.dialog = Some(dialog),
                            Err(error) => {
                                self.last_error =
                                    Some(format!("{}: {error:#}", tr(Text::ErrorsOpenSettings)))
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
                            self.resolved_language = localization::resolved_language();
                            for ui in [&self.window, &self.dialog, &self.about, &self.notices]
                                .into_iter()
                                .flatten()
                            {
                                ui.retranslate();
                            }
                            self.desktop.retranslate();
                        }
                        Err(error) => {
                            self.last_error =
                                Some(format!("{}: {error:#}", tr(Text::ErrorsSaveLanguage)))
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
                            self.last_error =
                                Some(format!("{}: {error:#}", tr(Text::ErrorsSaveClose)))
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
                if self.fan.is_some() {
                    self.fan_restart = true;
                    self.begin_fan_stop();
                } else {
                    self.request_fan(FanTarget::Auto);
                }
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
        self.render_dirty = true;
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
        self.render_dirty = true;
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
                if self.fan.is_none()
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
    fn refresh_theme(&mut self) {
        for ui in [&self.window, &self.dialog, &self.about, &self.notices]
            .into_iter()
            .flatten()
        {
            ui.refresh_theme();
        }
        if localization::resolved_language() != self.resolved_language {
            self.resolved_language = localization::resolved_language();
            for ui in [&self.window, &self.dialog, &self.about, &self.notices]
                .into_iter()
                .flatten()
            {
                ui.retranslate();
            }
            self.desktop.retranslate();
        }
        self.render_dirty = true;
    }
    fn timer_interval(&self) -> u32 {
        if self.fan.is_some() || self.slider_pending.is_some() || self.smoke_report.is_some() {
            250
        } else if self.window.is_some() {
            1000
        } else {
            5000
        }
    }
    fn tick(&mut self) {
        if let Some(result) =
            self.startup_pending
                .as_ref()
                .and_then(|pending| match pending.try_recv() {
                    Ok(result) => Some(result),
                    Err(mpsc::TryRecvError::Empty) => None,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Some(Err(anyhow::anyhow!("启动状态查询意外中断")))
                    }
                })
        {
            self.startup_pending = None;
            self.render_dirty = true;
            match result {
                Ok(status) => self.startup_status = Some(status),
                Err(error) => self.last_error = Some(format!("登录启动操作失败: {error:#}")),
            }
        }
        while let Some(action) = self.desktop.next_action() {
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
        self.monitor.refresh(self.window.is_some());
        self.desktop.refresh(self.monitor.snapshot().power_mode);
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
        if !self.render_dirty && self.rendered_revision == self.monitor.revision() {
            return;
        }
        self.render_dirty = false;
        self.rendered_revision = self.monitor.revision();
        let snapshot = self.monitor.snapshot();
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
                .unwrap_or_else(|| tr(Text::CommonUnavailable).into()),
        );
        ui.text(
            native::RPM,
            snapshot
                .fan_speed
                .map(|v| format!("{v} RPM"))
                .unwrap_or_else(|| tr(Text::CommonUnavailable).into()),
        );
        let gb = |v: u64| v as f64 / 1073741824.0;
        if snapshot.mem_total == 0 {
            ui.text(native::MEMORY, tr(Text::CommonUnavailable));
        } else {
            ui.text(
                native::MEMORY,
                format!(
                    "{:.0}% · {:.1}/{:.1} GiB",
                    snapshot.mem_usage,
                    gb(snapshot.mem_used),
                    gb(snapshot.mem_total)
                ),
            );
        }
        if snapshot.disk_total == 0 {
            ui.text(native::DISK, tr(Text::CommonUnavailable));
        } else {
            ui.text(
                native::DISK,
                format!(
                    "{:.0}% · {:.0}/{:.0} GiB",
                    snapshot.disk_usage,
                    gb(snapshot.disk_used),
                    gb(snapshot.disk_total)
                ),
            );
        }
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
        let adjustable = !self.fan_stopping && (self.fan.is_none() || self.fan_ready);
        ui.enable(native::MANUAL, false);
        ui.enable(native::MAXIMUM, adjustable);
        ui.enable(native::TARGET, false);
        ui.text(native::PERCENT, "—");
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
            dialog.text(native::ERROR, &message);
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
            .snapshot()
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
        match self.monitor.hardware_mut().set_power_mode(mode) {
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
        self.monitor.invalidate();
        self.render_dirty = true;
        self.monitor.refresh(self.window.is_some());
        self.desktop.refresh(self.monitor.snapshot().power_mode);
    }

    fn startup_operation(&mut self, operation: &'static str) {
        if self.startup_pending.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.startup_pending = Some(receiver);
        let wake = self.wake;
        std::thread::spawn(move || {
            let result = std::env::current_exe()
                .map_err(anyhow::Error::from)
                .and_then(|exe| startup::configure(operation, &exe));
            let _ = sender.send(result);
            wake.notify();
        });
    }
    fn request_fan(&mut self, target: FanTarget) {
        if self.fan_stopping {
            self.last_error = Some("风扇恢复仍在后台进行，请稍后重试".to_owned());
            return;
        }
        if self.fan.is_none() {
            match FanClient::start(self.wake) {
                Ok(fan) => self.fan = Some(fan),
                Err(error) => {
                    self.last_error = Some(format!("风扇连接失败: {error:#}"));
                    return;
                }
            }
        }
        if let Some(fan) = &mut self.fan {
            if self.fan_ready {
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
        self.fan_stopping = false;
        self.pending_fan_target = None;
    }
    fn begin_fan_stop(&mut self) {
        self.pending_fan_target = None;
        self.fan_stopping = self.fan.is_some();
        self.fan_ready = false;
        if let Some(fan) = &mut self.fan {
            if let Err(error) = fan.begin_stop() {
                self.last_error = Some(format!("风扇恢复请求失败: {error:#}"));
            }
        }
        self.render_dirty = true;
    }
    fn recover_fan(&mut self, reason: &str) {
        // A worker crash closes the pipe; the GUI makes an independent best-effort recovery.
        let recovery =
            WindowsHardwareControl::new().and_then(|hw| CoolingController::new(hw).restore_auto());
        self.last_error = Some(format!("{reason}；备用自动恢复: {recovery:?}"));
        self.fan_ready = false;
        if recovery.is_ok() {
            self.fan_target = 101;
        }
    }
    fn fan_tick(&mut self) {
        // Drain acknowledgements before heartbeat writes: a successful AUTO can
        // already have started shutdown, so a closed pipe is then expected.
        while let Some(line) = self.fan.as_ref().and_then(FanClient::next_message) {
            let mut fields = line.split_whitespace();
            match fields.next() {
                Some("READY") => {
                    self.fan_ready = true;
                    if let Some(target) = self.pending_fan_target.take() {
                        self.request_fan(target);
                    }
                }
                Some("DATA") => {
                    if let Some(target) = fields.nth(2).and_then(|v| v.parse().ok()) {
                        self.fan_target = target;
                    }
                }
                Some("OK") => {
                    self.fan_target = fields.next().and_then(|v| v.parse().ok()).unwrap_or(101);
                    self.last_error = None;
                    if self.fan_target == 101 {
                        self.begin_fan_stop();
                    }
                }
                Some("AUTO") => self.fan_target = 101,
                Some("UNAVAILABLE") => {
                    self.fan_ready = false;
                    self.pending_fan_target = None;
                    self.last_error = Some(line);
                }
                Some("RECOVERY") => {
                    self.fan_target = fields.next().and_then(|v| v.parse().ok()).unwrap_or(101);
                    self.last_error = Some(line);
                    if self.fan_target == 101 {
                        self.begin_fan_stop();
                    }
                }
                Some("ERROR") => self.last_error = Some(line),
                Some("DIED") if !self.fan_stopping => {
                    self.fan.take();
                    self.recover_fan(&line);
                }
                _ => {}
            }
            self.render_dirty = true;
        }
        if self.fan_stopping {
            match self.fan.as_mut().map(FanClient::poll_stop) {
                Some(Ok(true)) => {
                    self.fan.take();
                    self.fan_stopping = false;
                    self.fan_target = 101;
                    self.render_dirty = true;
                    if std::mem::take(&mut self.fan_restart) {
                        self.request_fan(FanTarget::Auto);
                    }
                }
                Some(Err(error)) => {
                    self.fan.take();
                    self.fan_stopping = false;
                    self.fan_restart = false;
                    self.recover_fan(&format!("{error:#}"));
                    self.render_dirty = true;
                }
                _ => {}
            }
            if self.fan.as_mut().is_some_and(FanClient::take_stop_timeout) {
                self.last_error = Some("风扇恢复仍在后台进行，请检查散热状态".to_owned());
                self.render_dirty = true;
            }
        } else if let Some(Err(error)) = self.fan.as_mut().map(FanClient::tick) {
            self.last_error = Some(format!("风扇通信中断: {error:#}"));
            self.begin_fan_stop();
            self.render_dirty = true;
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
