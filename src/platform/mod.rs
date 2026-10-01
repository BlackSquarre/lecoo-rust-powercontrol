pub mod traits;

#[cfg(target_os = "windows")]
pub mod windows;

pub use traits::HardwareControl;

#[cfg(target_os = "windows")]
pub use windows::{ComGuard, WindowsHardwareControl};
