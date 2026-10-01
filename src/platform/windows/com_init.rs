use windows::Win32::System::Com::*;
use anyhow::{Result, bail};

/// COM 初始化守卫（RAII）
pub struct ComGuard;

impl ComGuard {
    pub fn new() -> Result<Self> {
        unsafe {
            // 使用单线程模式以兼容 winit/egui
            let hr = CoInitializeEx(
                None,
                COINIT_APARTMENTTHREADED,
            );

            if hr.is_err() {
                bail!("COM 初始化失败: {:?}", hr);
            }

            // 设置进程安全级别
            let hr = CoInitializeSecurity(
                None,
                -1,  // 自动协商
                None,
                None,
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
                None,
            );

            if hr.is_err() {
                bail!("COM 安全初始化失败: {:?}", hr);
            }
        }

        Ok(ComGuard)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
