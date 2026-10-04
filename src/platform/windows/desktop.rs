use super::icons::ModeIcon;
use super::wake::UiWake;
use crate::core::PowerMode;
use crate::localization::{power_mode, text as tr, Text};
use anyhow::Result;
use std::cell::Cell;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};
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

pub struct Desktop {
    icon: TrayIcon,
    modes: Vec<(PowerMode, CheckMenuItem)>,
    receiver: Receiver<Action>,
    registered: Cell<bool>,
    commands: [MenuItem; 3],
    cached_icons: [Icon; 4],
    current_mode: Cell<Option<Option<PowerMode>>>,
    language_dirty: Cell<bool>,
    last_refresh: Cell<Instant>,
}
impl Desktop {
    pub fn new() -> Result<Self> {
        Self::with_wake(None)
    }
    pub fn with_wake(wake: Option<UiWake>) -> Result<Self> {
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
        let open = MenuItem::new(tr(Text::TrayOpenWindow), true, None);
        let hide = MenuItem::new(tr(Text::CommonMinimizeToTray), true, None);
        let exit = MenuItem::new(tr(Text::CommonExit), true, None);
        for (_, item) in &modes {
            menu.append(item)?;
        }
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append_items(&[&open, &hide, &exit])?;
        // Icon clones share their native handle through Arc; no bitmap rendering at runtime.
        let cached_icons = [
            Icon::from_resource(ModeIcon::Quiet as u16, Some((32, 32)))?,
            Icon::from_resource(ModeIcon::Balanced as u16, Some((32, 32)))?,
            Icon::from_resource(ModeIcon::Performance as u16, Some((32, 32)))?,
            Icon::from_resource(ModeIcon::Unknown as u16, Some((32, 32)))?,
        ];
        let icon = TrayIconBuilder::new()
            .with_icon(cached_icons[3].clone())
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
                if let Some(wake) = wake {
                    wake.notify();
                }
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
                if let Some(wake) = wake {
                    wake.notify();
                }
            }
        }));
        Ok(Self {
            icon,
            modes,
            receiver,
            registered: Cell::new(true),
            commands: [open, hide, exit],
            cached_icons,
            current_mode: Cell::new(None),
            language_dirty: Cell::new(true),
            last_refresh: Cell::new(Instant::now()),
        })
    }
    pub fn next_action(&self) -> Option<Action> {
        self.receiver.try_recv().ok()
    }
    pub fn refresh(&self, mode: Option<PowerMode>) {
        let changed = self.current_mode.get() != Some(mode);
        if !changed
            && !self.language_dirty.get()
            && self.registered.get()
            && self.last_refresh.get().elapsed() < Duration::from_secs(5)
        {
            return;
        }
        if changed || !self.registered.get() {
            let index = ModeIcon::from_mode(mode) as usize - 1;
            if self
                .icon
                .set_icon(Some(self.cached_icons[index].clone()))
                .is_err()
            {
                self.registered.set(false);
                return;
            }
            for (candidate, item) in &self.modes {
                item.set_checked(mode == Some(*candidate));
            }
            self.current_mode.set(Some(mode));
        }
        let text = mode
            .map(power_mode)
            .unwrap_or(tr(Text::TrayPowerUnavailable));
        // NIM_MODIFY acknowledges the registered icon even when its overflow rectangle is unavailable.
        self.registered.set(
            self.icon
                .set_tooltip(Some(format!("Lecoo Rust PowerControl — {text}")))
                .is_ok(),
        );
        self.language_dirty.set(false);
        self.last_refresh.set(Instant::now());
    }
    pub fn is_available(&self) -> bool {
        self.registered.get()
    }
    pub fn retranslate(&self) {
        self.language_dirty.set(true);
        for (mode, item) in &self.modes {
            item.set_text(power_mode(*mode));
        }
        for (item, text) in self.commands.iter().zip([
            tr(Text::TrayOpenWindow),
            tr(Text::CommonMinimizeToTray),
            tr(Text::CommonExit),
        ]) {
            item.set_text(text);
        }
    }
}
impl Drop for Desktop {
    fn drop(&mut self) {
        MenuEvent::set_event_handler(None::<fn(MenuEvent)>);
        TrayIconEvent::set_event_handler(None::<fn(TrayIconEvent)>);
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
