//! Embedded, DPI-sized icons. Owned handles are released with their window.
use crate::core::PowerMode;
use anyhow::Result;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HINSTANCE,
        System::LibraryLoader::GetModuleHandleW,
        UI::{HiDpi::GetSystemMetricsForDpi, WindowsAndMessaging::*},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum ModeIcon {
    Quiet = 1,
    Balanced = 2,
    Performance = 3,
    Unknown = 4,
}
impl ModeIcon {
    pub fn from_mode(mode: Option<PowerMode>) -> Self {
        match mode {
            Some(PowerMode::Quiet) => Self::Quiet,
            Some(PowerMode::Balance) => Self::Balanced,
            Some(PowerMode::Performance) => Self::Performance,
            None => Self::Unknown,
        }
    }
}

pub struct OwnedIcon(pub HICON);
impl OwnedIcon {
    fn load(mode: ModeIcon, size: i32) -> Result<Self> {
        unsafe {
            let handle = LoadImageW(
                HINSTANCE(GetModuleHandleW(None)?.0),
                PCWSTR(mode as usize as *const u16),
                IMAGE_ICON,
                size,
                size,
                LR_DEFAULTCOLOR,
            )?;
            Ok(Self(HICON(handle.0)))
        }
    }
}
impl Drop for OwnedIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyIcon(self.0);
        }
    }
}

pub struct WindowIcons {
    pub mode: ModeIcon,
    pub dpi: u32,
    pub small: OwnedIcon,
    pub big: OwnedIcon,
    pub about: Option<OwnedIcon>,
}
impl WindowIcons {
    pub fn load(mode: ModeIcon, dpi: u32, about: bool) -> Result<Self> {
        let small = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) }.max(16);
        let big = unsafe { GetSystemMetricsForDpi(SM_CXICON, dpi) }.max(32);
        Ok(Self {
            mode,
            dpi,
            small: OwnedIcon::load(mode, small)?,
            big: OwnedIcon::load(mode, big)?,
            about: if about {
                Some(OwnedIcon::load(mode, (64 * dpi / 96) as i32)?)
            } else {
                None
            },
        })
    }
}
