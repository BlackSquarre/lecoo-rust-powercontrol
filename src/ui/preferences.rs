use anyhow::{Context, Result};
use lecoo_control_center::localization::Language;
use std::path::PathBuf;

fn path() -> Result<PathBuf> {
    Ok(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA 不可用")?)
            .join("LecooRustPowerControl")
            .join("preferences.txt"),
    )
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseBehavior {
    Ask,
    Tray,
    Exit,
}
impl CloseBehavior {
    fn value(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Tray => "tray",
            Self::Exit => "exit",
        }
    }
}
fn parse(text: &str) -> CloseBehavior {
    let value = text
        .lines()
        .find(|line| line.starts_with("close_behavior=") || line.starts_with("close_to_tray="))
        .unwrap_or("")
        .trim();
    match value {
        "close_behavior=tray" | "close_to_tray=true" => CloseBehavior::Tray,
        "close_behavior=exit" => CloseBehavior::Exit,
        // The old false value was also the default: it did not represent a remembered choice.
        _ => CloseBehavior::Ask,
    }
}
pub fn load() -> CloseBehavior {
    path()
        .ok()
        .and_then(|file| std::fs::read_to_string(file).ok())
        .map(|text| parse(&text))
        .unwrap_or(CloseBehavior::Ask)
}
pub fn save(behavior: CloseBehavior) -> Result<()> {
    save_all(behavior, load_language())
}
pub fn load_language() -> Language {
    let contents = path()
        .ok()
        .and_then(|file| std::fs::read_to_string(file).ok())
        .unwrap_or_default();
    parse_language(&contents)
}
fn parse_language(contents: &str) -> Language {
    contents
        .lines()
        .find_map(|line| line.strip_prefix("language="))
        .map(Language::parse)
        .unwrap_or(Language::System)
}
pub fn save_all(behavior: CloseBehavior, language: Language) -> Result<()> {
    let file = path()?;
    std::fs::create_dir_all(file.parent().unwrap())?;
    let temporary = file.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(
        &temporary,
        format!(
            "close_behavior={}\nlanguage={}\n",
            behavior.value(),
            language.value()
        ),
    )?;
    std::fs::rename(&temporary, &file).context("无法保存关闭选择")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_default_does_not_skip_prompt() {
        assert_eq!(parse("close_to_tray=false"), CloseBehavior::Ask);
        assert_eq!(parse("garbage"), CloseBehavior::Ask);
    }
    #[test]
    fn remembered_choice_round_trips() {
        for behavior in [CloseBehavior::Ask, CloseBehavior::Tray, CloseBehavior::Exit] {
            assert_eq!(
                parse(&format!("close_behavior={}\n", behavior.value())),
                behavior
            );
        }
    }
    #[test]
    fn explicit_old_tray_choice_is_preserved() {
        assert_eq!(parse("close_to_tray=true\n"), CloseBehavior::Tray);
    }
    #[test]
    fn multilingual_preferences_preserve_both_fields_and_legacy_defaults() {
        for language in Language::ALL {
            for behavior in [CloseBehavior::Ask, CloseBehavior::Tray, CloseBehavior::Exit] {
                let contents = format!(
                    "close_behavior={}\nlanguage={}\n",
                    behavior.value(),
                    language.value()
                );
                assert_eq!(parse(&contents), behavior);
                assert_eq!(parse_language(&contents), language);
            }
        }
        assert_eq!(parse_language("close_to_tray=true\n"), Language::System);
    }
}
