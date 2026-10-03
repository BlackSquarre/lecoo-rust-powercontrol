//! Small, allocation-free bilingual resources shared by the UI and tray.
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    System,
    Chinese,
    English,
}
static ACTIVE: AtomicU8 = AtomicU8::new(0);
impl Language {
    pub fn value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Chinese => "zh-CN",
            Self::English => "en",
        }
    }
    pub fn parse(value: &str) -> Self {
        match value {
            "zh-CN" => Self::Chinese,
            "en" => Self::English,
            _ => Self::System,
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::System => 0,
            Self::Chinese => 1,
            Self::English => 2,
        }
    }
    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Chinese,
            2 => Self::English,
            _ => Self::System,
        }
    }
}
pub fn set_language(language: Language) {
    ACTIVE.store(language.index() as u8, Ordering::Relaxed);
}
pub fn language() -> Language {
    Language::from_index(ACTIVE.load(Ordering::Relaxed) as usize)
}
pub fn is_english() -> bool {
    match language() {
        Language::English => true,
        Language::Chinese => false,
        Language::System => {
            #[cfg(windows)]
            {
                unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() & 0x3ff != 4 }
            }
            #[cfg(not(windows))]
            {
                true
            }
        }
    }
}
pub fn text<'a>(chinese: &'a str, english: &'a str) -> &'a str {
    if is_english() {
        english
    } else {
        chinese
    }
}
pub fn power_mode(mode: crate::core::PowerMode) -> &'static str {
    match mode {
        crate::core::PowerMode::Quiet => text("安静模式", "Quiet mode"),
        crate::core::PowerMode::Balance => text("均衡模式", "Balanced mode"),
        crate::core::PowerMode::Performance => text("性能模式", "Performance mode"),
    }
}

/// Keep compact user-facing failures meaningful without exposing untranslated
/// firmware/worker diagnostics. Recovery requests are never called confirmed recovery.
pub fn user_error(message: &str) -> String {
    if !is_english() {
        return message.to_owned();
    }
    english_error(message).to_owned()
}
fn english_error(message: &str) -> &'static str {
    if message.contains("恢复失败")
        || message.contains("无法确认恢复")
        || message.contains("恢复也失败")
    {
        "Automatic fan recovery could not be confirmed"
    } else if message.contains("RECOVERY") {
        "Automatic fan control requested by the safety monitor"
    } else if message.contains("拒绝手动控制") || message.contains("温度过高") {
        "CPU temperature is too high for manual fan control"
    } else if message.contains("采样无效")
        || message.contains("采样失败")
        || message.contains("有效 CPU 温度")
    {
        "A valid CPU temperature and fan speed are required"
    } else if message.contains("另一个风扇控制会话") {
        "Another fan control session is already running"
    } else if message.contains("ResultStatus=255") {
        "The firmware rejected the requested operation"
    } else if message.contains("管理员") || message.contains("权限") {
        "Administrator privileges are required"
    } else if message.contains("登录启动") {
        "Unable to update sign-in startup"
    } else if message.contains("保存") {
        "Unable to save preferences"
    } else if message.contains("打开") {
        "Unable to open the window"
    } else {
        "The requested operation could not be completed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn language_values_round_trip_without_guessing_unknown_codes() {
        for language in [Language::System, Language::Chinese, Language::English] {
            assert_eq!(Language::parse(language.value()), language);
            assert_eq!(Language::from_index(language.index()), language);
        }
        assert_eq!(Language::parse("garbage"), Language::System);
    }
    #[test]
    fn recovery_failure_translation_does_not_claim_recovery_succeeded() {
        assert_eq!(
            english_error("RECOVERY 无法确认恢复自动控制"),
            "Automatic fan recovery could not be confirmed"
        );
        assert_eq!(
            english_error("RECOVERY 主程序心跳超时，已提交自动恢复并读回转速"),
            "Automatic fan control requested by the safety monitor"
        );
        assert_eq!(
            english_error("CPU 温度达到 90.0°C，已拒绝手动控制"),
            "CPU temperature is too high for manual fan control"
        );
    }
}
