use serde::{Deserialize, Serialize};

const LANG_ZH: &str = "zh";
const LANG_ZH_HANT: &str = "zh-hant";
const LANG_ZH_TW: &str = "zh-tw";
const LANG_ZH_HK: &str = "zh-hk";
const LANG_JA: &str = "ja";
const LANG_KO: &str = "ko";
const LANG_FR: &str = "fr";
const LANG_ES: &str = "es";
const LANG_RU: &str = "ru";
const LANG_AR: &str = "ar";
const LANG_DE: &str = "de";
const LANG_PT: &str = "pt";

/// All supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    /// Simplified Chinese
    #[serde(rename = "zh-Hans")]
    #[default]
    ZHS,
    /// Traditional Chinese
    #[serde(rename = "zh-Hant")]
    ZHT,
    /// English
    #[serde(rename = "en")]
    En,
    /// Japanese
    #[serde(rename = "ja")]
    Ja,
    /// Korean
    #[serde(rename = "ko")]
    Ko,
    /// French
    #[serde(rename = "fr")]
    Fr,
    /// Spanish
    #[serde(rename = "es")]
    Es,
    /// Russian
    #[serde(rename = "ru")]
    Ru,
    /// Arabic
    #[serde(rename = "ar")]
    Ar,
    /// German
    #[serde(rename = "de")]
    De,
    /// Portuguese
    #[serde(rename = "pt")]
    Pt,
}

impl Language {
    /// Language code used in doc directories (for locating language folders under docs/agents)
    pub fn doc_code(&self) -> &'static str {
        match self {
            Language::ZHS => "zh-Hans",
            Language::ZHT => "zh-Hant",
            Language::En => "en",
            Language::Ja => "ja",
            Language::Ko => "ko",
            Language::Fr => "fr",
            Language::Es => "es",
            Language::Ru => "ru",
            Language::Ar => "ar",
            Language::De => "de",
            Language::Pt => "pt",
        }
    }

    /// Language code used in i18n locale directories
    pub fn locale_code(&self) -> &'static str {
        self.doc_code()
    }

    /// Key name used in entrypoint TOML
    pub fn entry_point_lang_key(&self) -> &'static str {
        match self {
            Language::ZHS => "zh-Hans",
            Language::ZHT => "zh-Hant",
            Language::En => "en",
            Language::Ja => "ja",
            Language::Ko => "ko",
            Language::Fr => "fr",
            Language::Es => "es",
            Language::Ru => "ru",
            Language::Ar => "ar",
            Language::De => "de",
            Language::Pt => "pt",
        }
    }

    /// Native language name
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::ZHS => "简体中文",
            Language::ZHT => "繁體中文",
            Language::En => "English",
            Language::Ja => "日本語",
            Language::Ko => "한국어",
            Language::Fr => "Français",
            Language::Es => "Español",
            Language::Ru => "Русский",
            Language::Ar => "العربية",
            Language::De => "Deutsch",
            Language::Pt => "Português",
        }
    }

    /// Parse a language from its code
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "zh-Hans" | "zhs" | "ZHS" => Some(Language::ZHS),
            "zh-Hant" | "zht" | "ZHT" => Some(Language::ZHT),
            "en" => Some(Language::En),
            "ja" => Some(Language::Ja),
            "ko" => Some(Language::Ko),
            "fr" => Some(Language::Fr),
            "es" => Some(Language::Es),
            "ru" => Some(Language::Ru),
            "ar" => Some(Language::Ar),
            "de" => Some(Language::De),
            "pt" => Some(Language::Pt),
            _ => None,
        }
    }

    /// Parse a language from its code, defaulting to Default (ZHS)
    pub fn from_code_or_default(code: &str) -> Self {
        Self::from_code(code).unwrap_or_default()
    }

    /// Detect the user's preferred language from system locale settings
    ///
    /// Returns `Some(language)` on successfully detecting a supported language,
    /// or `None` if no locale could be detected, or only English was detected.
    ///
    /// English is treated as a special case that requires explicit selection:
    /// if the system language is English, returns `None` so the caller can
    /// still show a language selection dialog.
    pub fn from_system_locale() -> Option<Self> {
        let locale = sys_locale::get_locale()?;
        Self::parse_bcp47(&locale)
    }

    /// Parse a BCP 47 language tag into a `Language`
    ///
    /// Returns `None` when:
    /// - The tag does not match any supported language
    /// - The tag matches English (English must be selected via dialog, so the caller handles it)
    fn parse_bcp47(tag: &str) -> Option<Self> {
        let lang_tag = tag.to_lowercase();
        if lang_tag.starts_with(LANG_ZH) {
            if lang_tag.starts_with(LANG_ZH_HANT)
                || lang_tag.starts_with(LANG_ZH_TW)
                || lang_tag.starts_with(LANG_ZH_HK)
            {
                Some(Language::ZHT)
            } else {
                Some(Language::ZHS)
            }
        } else if lang_tag.starts_with(LANG_JA) {
            Some(Language::Ja)
        } else if lang_tag.starts_with(LANG_KO) {
            Some(Language::Ko)
        } else if lang_tag.starts_with(LANG_FR) {
            Some(Language::Fr)
        } else if lang_tag.starts_with(LANG_ES) {
            Some(Language::Es)
        } else if lang_tag.starts_with(LANG_RU) {
            Some(Language::Ru)
        } else if lang_tag.starts_with(LANG_AR) {
            Some(Language::Ar)
        } else if lang_tag.starts_with(LANG_DE) {
            Some(Language::De)
        } else if lang_tag.starts_with(LANG_PT) {
            Some(Language::Pt)
        } else {
            None
        }
    }

    /// Return the uppercase language code (ZHS, ZHT, en, ja, etc.)
    pub fn code(&self) -> &'static str {
        match self {
            Language::ZHS => "zh-Hans",
            Language::ZHT => "zh-Hant",
            Language::En => "en",
            Language::Ja => "ja",
            Language::Ko => "ko",
            Language::Fr => "fr",
            Language::Es => "es",
            Language::Ru => "ru",
            Language::Ar => "ar",
            Language::De => "de",
            Language::Pt => "pt",
        }
    }

    /// Get the list of all supported languages
    pub fn all() -> &'static [Language] {
        SUPPORTED_LANGUAGES
    }
}

/// All supported languages (compile-time constant)
pub const SUPPORTED_LANGUAGES: &[Language] = &[
    Language::ZHS,
    Language::ZHT,
    Language::En,
    Language::Ja,
    Language::Ko,
    Language::Fr,
    Language::Es,
    Language::Ru,
    Language::Ar,
    Language::De,
    Language::Pt,
];

/// List of supported language codes (as strings)
pub const SUPPORTED_LANG_CODES: &[&str] = &[
    "zh-Hans", "zh-Hant", "en", "ja", "ko", "fr", "es", "ru", "ar", "de", "pt",
];

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_language_codes() -> Result<()> {
        assert_eq!(Language::ZHS.doc_code(), "zh-Hans");
        assert_eq!(Language::En.doc_code(), "en");
        assert_eq!(Language::Ja.doc_code(), "ja");
        Ok(())
    }

    #[test]
    fn test_entry_point_keys() -> Result<()> {
        assert_eq!(Language::ZHS.entry_point_lang_key(), "zh-Hans");
        assert_eq!(Language::En.entry_point_lang_key(), "en");
        Ok(())
    }

    #[test]
    fn test_from_code() -> Result<()> {
        assert_eq!(Language::from_code("zh-Hans"), Some(Language::ZHS));
        assert_eq!(Language::from_code("en"), Some(Language::En));
        assert_eq!(Language::from_code("invalid"), None);
        Ok(())
    }

    #[test]
    fn test_native_names() -> Result<()> {
        assert_eq!(Language::ZHS.native_name(), "简体中文");
        assert_eq!(Language::En.native_name(), "English");
        assert_eq!(Language::Ja.native_name(), "日本語");
        Ok(())
    }

    #[test]
    fn test_all_languages() -> Result<()> {
        let all = Language::all();
        assert_eq!(all.len(), 11);
        assert!(all.contains(&Language::ZHS));
        assert!(all.contains(&Language::En));
        assert!(all.contains(&Language::Ar));
        assert!(all.contains(&Language::De));
        assert!(all.contains(&Language::Pt));
        Ok(())
    }

    #[test]
    fn test_parse_bcp47() -> Result<()> {
        assert_eq!(Language::parse_bcp47("zh-CN"), Some(Language::ZHS));
        assert_eq!(Language::parse_bcp47("zh-Hans"), Some(Language::ZHS));
        assert_eq!(Language::parse_bcp47("zh-SG"), Some(Language::ZHS));
        assert_eq!(Language::parse_bcp47("zh-TW"), Some(Language::ZHT));
        assert_eq!(Language::parse_bcp47("zh-Hant"), Some(Language::ZHT));
        assert_eq!(Language::parse_bcp47("zh-HK"), Some(Language::ZHT));
        assert_eq!(Language::parse_bcp47("zh"), Some(Language::ZHS));
        assert_eq!(Language::parse_bcp47("ja"), Some(Language::Ja));
        assert_eq!(Language::parse_bcp47("ja-JP"), Some(Language::Ja));
        assert_eq!(Language::parse_bcp47("ko"), Some(Language::Ko));
        assert_eq!(Language::parse_bcp47("ko-KR"), Some(Language::Ko));
        assert_eq!(Language::parse_bcp47("fr"), Some(Language::Fr));
        assert_eq!(Language::parse_bcp47("fr-FR"), Some(Language::Fr));
        assert_eq!(Language::parse_bcp47("es"), Some(Language::Es));
        assert_eq!(Language::parse_bcp47("es-ES"), Some(Language::Es));
        assert_eq!(Language::parse_bcp47("ru"), Some(Language::Ru));
        assert_eq!(Language::parse_bcp47("ru-RU"), Some(Language::Ru));
        assert_eq!(Language::parse_bcp47("ar"), Some(Language::Ar));
        assert_eq!(Language::parse_bcp47("ar-SA"), Some(Language::Ar));
        assert_eq!(Language::parse_bcp47("de"), Some(Language::De));
        assert_eq!(Language::parse_bcp47("de-DE"), Some(Language::De));
        assert_eq!(Language::parse_bcp47("pt"), Some(Language::Pt));
        assert_eq!(Language::parse_bcp47("pt-BR"), Some(Language::Pt));
        assert_eq!(Language::parse_bcp47("en"), None);
        assert_eq!(Language::parse_bcp47("en-US"), None);
        assert_eq!(Language::parse_bcp47("xx"), None);
        Ok(())
    }
}
