use anyhow::{Context, Result};
use windows::{
    core::w,
    Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE},
        System::Threading::{CreateEventW, CreateMutexW, SetEvent},
    },
};

pub struct Instance {
    mutex: HANDLE,
    event: HANDLE,
}
impl Instance {
    /// Local namespace scopes this guard to the interactive Windows session.
    pub fn acquire() -> Result<Option<Self>> {
        unsafe {
            // Create the event first, so a second launch cannot race its creation.
            let event = CreateEventW(None, false, false, w!("Local\\LecooRustPowerControl.Open"))?;
            let mutex = match CreateMutexW(None, false, w!("Local\\LecooRustPowerControl.Instance"))
            {
                Ok(value) => value,
                Err(error) => {
                    let _ = CloseHandle(event);
                    return Err(error).context("单实例锁创建失败");
                }
            };
            let existing = GetLastError() == ERROR_ALREADY_EXISTS;
            let guard = Self { mutex, event };
            if existing {
                SetEvent(event).context("无法唤醒已运行的应用")?;
                return Ok(None);
            }
            Ok(Some(guard))
        }
    }
    pub fn open_event(&self) -> HANDLE {
        self.event
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.event);
            let _ = CloseHandle(self.mutex);
        }
    }
}
