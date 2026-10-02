use anyhow::{bail, Context, Result};
use std::{os::windows::process::CommandExt, path::Path, process::Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupStatus {
    Disabled,
    Enabled,
    Stale,
}

/// Task Scheduler is the source of truth. No private flag silently enables startup.
pub fn configure(operation: &str, executable: &Path) -> Result<StartupStatus> {
    if !["status", "enable", "disable"].contains(&operation) {
        bail!("Invalid startup operation");
    }
    let powershell = Path::new(&std::env::var_os("SystemRoot").context("SystemRoot missing")?)
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let output = Command::new(powershell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            include_str!("startup.ps1"),
        ])
        .env("LECOO_STARTUP_OPERATION", operation)
        .env("LECOO_STARTUP_EXE", executable)
        .creation_flags(0x08000000)
        .output()
        .context("无法访问 Windows 任务计划程序")?;
    if !output.status.success() {
        bail!(
            "登录启动设置失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
        "ENABLED" => Ok(StartupStatus::Enabled),
        "DISABLED" => Ok(StartupStatus::Disabled),
        "STALE" => Ok(StartupStatus::Stale),
        value => bail!("Unexpected startup status: {value}"),
    }
}
