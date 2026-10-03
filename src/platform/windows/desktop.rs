use crate::core::PowerMode;
use crate::localization::{power_mode, text as tr};
use anyhow::Result;
use std::cell::Cell;
use std::sync::mpsc::{self, Receiver};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Mode(PowerMode),
    Open,
    Hide,
    Exit,
}

/// A small original power-button icon shared by the tray and window.
pub fn icon_rgba() -> Vec<u8> {
    icon_rgba_size(32)
}
pub fn icon_rgba_size(size: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 + 0.5) * 32.0 / size as f32 - 16.0;
            let dy = (y as f32 + 0.5) * 32.0 / size as f32 - 16.0;
            let distance = (dx * dx + dy * dy).sqrt();
            let ring = (8.0..=11.0).contains(&distance) && !(dy < -4.0 && dx.abs() < 5.0);
            let stem = dx.abs() < 1.8 && (-12.0..=0.0).contains(&dy);
            let color = if ring || stem {
                [0, 210, 166, 255]
            } else if distance <= 15.0 {
                [18, 25, 33, 255]
            } else {
                [0, 0, 0, 0]
            };
            rgba[(y * size + x) * 4..(y * size + x + 1) * 4].copy_from_slice(&color);
        }
    }
    rgba
}

pub struct Desktop {
    icon: TrayIcon,
    modes: Vec<(PowerMode, CheckMenuItem)>,
    receiver: Receiver<Action>,
    registered: Cell<bool>,
    commands: [MenuItem; 3],
}
impl Desktop {
    pub fn new() -> Result<Self> {
        let menu = Menu::new();
        let modes: Vec<_> = [PowerMode::Quiet, PowerMode::Balance, PowerMode::Performance]
            .into_iter()
            .map(|mode| {
                (
                    mode,
                    CheckMenuItem::new(power_mode(mode), true, false, None),
                )
            })
            .collect();
        let open = MenuItem::new(tr("打开主窗口", "Open window"), true, None);
        let hide = MenuItem::new(tr("最小化到托盘", "Minimize to tray"), true, None);
        let exit = MenuItem::new(tr("退出", "Exit"), true, None);
        for (_, item) in &modes {
            menu.append(item)?;
        }
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append_items(&[&open, &hide, &exit])?;
        let icon = TrayIconBuilder::new()
            .with_icon(Icon::from_rgba(icon_rgba(), 32, 32)?)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip("Lecoo Rust PowerControl")
            .build()?;
        let (sender, receiver) = mpsc::channel();
        let ids: Vec<_> = modes
            .iter()
            .map(|(mode, item)| (item.id().clone(), *mode))
            .collect();
        let open_id = open.id().clone();
        let hide_id = hide.id().clone();
        let exit_id = exit.id().clone();
        let menu_sender = sender.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == open_id {
                Some(Action::Open)
            } else if event.id == hide_id {
                Some(Action::Hide)
            } else if event.id == exit_id {
                Some(Action::Exit)
            } else {
                ids.iter()
                    .find(|(id, _)| *id == event.id)
                    .map(|(_, mode)| Action::Mode(*mode))
            };
            if let Some(action) = action {
                let _ = menu_sender.send(action);
            }
        }));
        TrayIconEvent::set_event_handler(Some(move |event| {
            if matches!(
                event,
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                let _ = sender.send(Action::Open);
            }
        }));
        Ok(Self {
            icon,
            modes,
            receiver,
            registered: Cell::new(true),
            commands: [open, hide, exit],
        })
    }
    pub fn actions(&self) -> Vec<Action> {
        self.receiver.try_iter().collect()
    }
    pub fn refresh(&self, mode: Option<PowerMode>) {
        for (candidate, item) in &self.modes {
            item.set_checked(mode == Some(*candidate));
        }
        let text = mode
            .map(power_mode)
            .unwrap_or(tr("模式读取失败", "Power mode unavailable"));
        // NIM_MODIFY acknowledges the registered icon even when its overflow rectangle is unavailable.
        self.registered.set(
            self.icon
                .set_tooltip(Some(format!("Lecoo Rust PowerControl — {text}")))
                .is_ok(),
        );
    }
    pub fn is_available(&self) -> bool {
        self.registered.get()
    }
    pub fn retranslate(&self) {
        for (mode, item) in &self.modes {
            item.set_text(power_mode(*mode));
        }
        for (item, text) in self.commands.iter().zip([
            tr("打开主窗口", "Open window"),
            tr("最小化到托盘", "Minimize to tray"),
            tr("退出", "Exit"),
        ]) {
            item.set_text(text);
        }
    }
}
pub fn show_error(message: &str) {
    use windows::{
        core::HSTRING,
        Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK},
    };
    unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(message),
            windows::core::w!("Lecoo Rust PowerControl"),
            MB_OK | MB_ICONERROR,
        );
    }
}
