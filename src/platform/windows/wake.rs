use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, WM_APP},
};

pub const WAKE: u32 = WM_APP + 9;

/// A notification target, not a cross-thread owner of an HWND or COM interface.
#[derive(Clone, Copy)]
pub struct UiWake(usize);
impl UiWake {
    pub fn new(hwnd: HWND) -> Self {
        Self(hwnd.0 as usize)
    }
    pub fn notify(self) {
        if self.0 == 0 {
            return;
        }
        unsafe {
            let _ = PostMessageW(HWND(self.0 as *mut _), WAKE, WPARAM(0), LPARAM(0));
        }
    }
}
