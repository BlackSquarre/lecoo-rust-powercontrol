use anyhow::{bail, Result};
use windows::Win32::{Foundation::RPC_E_TOO_LATE, System::Com::*};

/// COM 初始化守卫（RAII）
pub struct ComGuard;

impl ComGuard {
    pub fn new() -> Result<Self> {
        unsafe {
            // Keep UI and background operations in their own COM apartments.
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            if hr.is_err() {
                bail!("COM 初始化失败: {:?}", hr);
            }
            let guard = Self;

            // 设置进程安全级别
            let hr = CoInitializeSecurity(
                None,
                -1, // 自动协商
                None,
                None,
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
                None,
            );

            // Security is process-wide; another initialized apartment may own it.
            if let Err(error) = hr {
                if error.code() != RPC_E_TOO_LATE {
                    bail!("COM 安全初始化失败: {:?}", error);
                }
            }
            Ok(guard)
        }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
