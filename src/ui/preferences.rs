use anyhow::{Context, Result};
use std::path::PathBuf;

fn path() -> Result<PathBuf> {
    Ok(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA 不可用")?)
            .join("LecooRustPowerControl")
            .join("preferences.txt"),
    )
}
pub fn close_to_tray() -> bool {
    path()
        .ok()
        .and_then(|file| std::fs::read_to_string(file).ok())
        .map(|text| text.trim() == "close_to_tray=true")
        .unwrap_or(false)
}
pub fn save(close_to_tray: bool) -> Result<()> {
    let file = path()?;
    std::fs::create_dir_all(file.parent().unwrap())?;
    std::fs::write(file, format!("close_to_tray={close_to_tray}\n"))?;
    Ok(())
}
