//! Allocation-free language resources shared by the native UI and tray.
mod resources {
    include!(concat!(env!("OUT_DIR"), "/localization_resources.rs"));
}
pub use resources::Text;

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Language {
    System,
    Chinese,
    English,
    TraditionalChinese,
    Japanese,
    Korean,
    Spanish,
    French,
    German,
}

static ACTIVE: AtomicU8 = AtomicU8::new(0);

impl Language {
    /// Preserve the existing preference values and dropdown indices.
    pub const ALL: [Self; 9] = [
        Self::System,
        Self::Chinese,
        Self::English,
        Self::TraditionalChinese,
        Self::Japanese,
        Self::Korean,
        Self::Spanish,
        Self::French,
        Self::German,
    ];

    pub fn value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Chinese => "zh-CN",
            Self::English => "en",
            Self::TraditionalChinese => "zh-TW",
            Self::Japanese => "ja",
            Self::Korean => "ko",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::German => "de",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "zh-CN" => Self::Chinese,
            "en" => Self::English,
            "zh-TW" => Self::TraditionalChinese,
            "ja" => Self::Japanese,
            "ko" => Self::Korean,
            "es" => Self::Spanish,
            "fr" => Self::French,
            "de" => Self::German,
            _ => Self::System,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(Self::System)
    }

    pub fn name(self) -> &'static str {
        if self == Self::System {
            text(Text::SettingsSystemLanguage)
        } else {
            resources::LANGUAGE_NAMES[self.index() - 1]
        }
    }

    fn from_windows_lang_id(id: u16) -> Self {
        match id & 0x3ff {
            // Taiwan, Hong Kong, Macao and the Traditional Chinese neutral ID.
            0x04 if matches!(id >> 10, 1 | 3 | 5 | 0x1f) => Self::TraditionalChinese,
            0x04 => Self::Chinese,
            0x11 => Self::Japanese,
            0x12 => Self::Korean,
            0x0a => Self::Spanish,
            0x0c => Self::French,
            0x07 => Self::German,
            _ => Self::English,
        }
    }
}

pub fn language_options() -> [&'static str; Language::ALL.len()] {
    Language::ALL.map(Language::name)
}

pub fn set_language(language: Language) {
    ACTIVE.store(language.index() as u8, Ordering::Relaxed);
}

pub fn language() -> Language {
    Language::from_index(ACTIVE.load(Ordering::Relaxed) as usize)
}

pub fn resolved_language() -> Language {
    match language() {
        Language::System => system_language(),
        explicit => explicit,
    }
}

fn system_language() -> Language {
    #[cfg(windows)]
    {
        Language::from_windows_lang_id(unsafe {
            windows::Win32::Globalization::GetUserDefaultUILanguage()
        })
    }
    #[cfg(not(windows))]
    {
        Language::English
    }
}

pub fn is_english() -> bool {
    resolved_language() == Language::English
}

pub fn text(key: Text) -> &'static str {
    text_for(resolved_language(), key)
}

fn text_for(language: Language, key: Text) -> &'static str {
    let resolved = if language == Language::System {
        system_language()
    } else {
        language
    };
    resources::MESSAGES[resolved.index() - 1][key as usize]
}

pub fn power_mode(mode: crate::core::PowerMode) -> &'static str {
    match mode {
        crate::core::PowerMode::Quiet => text(Text::PowerQuietMode),
        crate::core::PowerMode::Balance => text(Text::PowerBalancedMode),
        crate::core::PowerMode::Performance => text(Text::PowerPerformanceMode),
    }
}

/// Compact failures never present a recovery request as confirmed recovery.
pub fn user_error(message: &str) -> String {
    user_error_for(resolved_language(), message)
}

fn user_error_for(language: Language, message: &str) -> String {
    text_for(language, error_key(message)).to_owned()
}

fn error_key(message: &str) -> Text {
    if message.contains("恢复失败")
        || message.contains("无法确认恢复")
        || message.contains("恢复也失败")
    {
        Text::ErrorsRecoveryUnconfirmed
    } else if message.contains("RECOVERY") {
        Text::ErrorsAutomaticRequested
    } else if message.contains("手动风扇目标暂不可用") {
        Text::ErrorsManualSensorValidation
    } else if message.contains("拒绝手动控制") || message.contains("温度过高") {
        Text::ErrorsHighTemperature
    } else if message.contains("采样无效")
        || message.contains("采样失败")
        || message.contains("有效 CPU 温度")
    {
        Text::ErrorsInvalidSamples
    } else if message.contains("另一个风扇控制会话") {
        Text::ErrorsFanSessionActive
    } else if message.contains("ResultStatus=255") {
        Text::ErrorsFirmwareRejected
    } else if message.contains("管理员") || message.contains("权限") {
        Text::ErrorsAdminRequired
    } else if message.contains("登录启动") {
        Text::ErrorsStartupUpdate
    } else if message.contains("保存") {
        Text::ErrorsSavePreferences
    } else if message.contains("打开") {
        Text::ErrorsOpenWindow
    } else {
        Text::ErrorsOperationFailed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_values_round_trip_without_guessing_unknown_codes() {
        for language in Language::ALL {
            assert_eq!(Language::parse(language.value()), language);
            assert_eq!(Language::from_index(language.index()), language);
        }
        assert_eq!(Language::parse("garbage"), Language::System);
        assert_eq!(Language::from_index(usize::MAX), Language::System);
        assert_eq!(Language::Chinese.index(), 1);
        assert_eq!(Language::English.index(), 2);
    }

    #[test]
    fn windows_display_languages_resolve_script_and_regional_variants() {
        for id in [0x0404, 0x0c04, 0x1404, 0x7c04] {
            assert_eq!(
                Language::from_windows_lang_id(id),
                Language::TraditionalChinese
            );
        }
        for id in [0x0004, 0x0804, 0x1004, 0x7804] {
            assert_eq!(Language::from_windows_lang_id(id), Language::Chinese);
        }
        for (id, expected) in [
            (0x0411, Language::Japanese),
            (0x0412, Language::Korean),
            (0x040a, Language::Spanish),
            (0x080a, Language::Spanish),
            (0x0c0a, Language::Spanish),
            (0x040c, Language::French),
            (0x0c0c, Language::French),
            (0x0407, Language::German),
            (0x0807, Language::German),
            (0x0409, Language::English),
            (0x0419, Language::English),
        ] {
            assert_eq!(Language::from_windows_lang_id(id), expected);
        }
    }

    #[test]
    fn all_languages_use_resource_text_including_simplified_chinese_and_english() {
        for (language, expected) in [
            (Language::Chinese, "设置"),
            (Language::English, "Settings"),
            (Language::TraditionalChinese, "設定"),
            (Language::Japanese, "設定"),
            (Language::Korean, "설정"),
            (Language::Spanish, "Ajustes"),
            (Language::French, "Paramètres"),
            (Language::German, "Einstellungen"),
        ] {
            assert_eq!(text_for(language, Text::SettingsTitle), expected);
        }
    }

    #[test]
    fn generated_resources_cover_every_key_and_language_name() {
        for language in Language::ALL.into_iter().skip(1) {
            assert!(!language.name().is_empty());
            for key in Text::ALL {
                assert!(
                    !text_for(language, key).trim().is_empty(),
                    "{}: {}",
                    language.value(),
                    key.key()
                );
            }
        }
    }

    #[test]
    fn recovery_failure_translation_does_not_claim_recovery_succeeded() {
        let failure = "RECOVERY 无法确认恢复自动控制";
        let request = "RECOVERY 主程序心跳超时，已提交自动恢复并读回转速";
        assert_eq!(
            user_error_for(Language::English, failure),
            "Automatic fan recovery could not be confirmed"
        );
        assert_eq!(
            user_error_for(Language::English, request),
            "Automatic fan control requested by the safety monitor"
        );
        assert_eq!(
            user_error_for(Language::English, "CPU 温度达到 90.0°C，已拒绝手动控制"),
            "CPU temperature is too high for manual fan control"
        );
        for language in Language::ALL.into_iter().skip(1) {
            let translated = user_error_for(language, failure);
            assert_ne!(translated, user_error_for(language, request));
            assert_ne!(translated, failure);
        }
    }
}
