use include_dir::{Dir, include_dir};

use crate::Language;

/// Compile-time embedded locale directories for all languages
///
/// Path relative to packages/i18n, resolved from CARGO_MANIFEST_DIR
pub static LOCALES_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../res/i18n/locales");

/// Get the locale directory for a given language
pub fn get_locale_dir(language: Language) -> Option<&'static Dir<'static>> {
    LOCALES_DIR.get_dir(language.locale_code())
}

/// Get all available locale directories
pub fn get_all_locale_dirs() -> Vec<(Language, &'static Dir<'static>)> {
    Language::all()
        .iter()
        .filter_map(|&lang| get_locale_dir(lang).map(|dir| (lang, dir)))
        .collect()
}

/// Check if the specified language has complete translation files
///
/// Returns a list of missing files, or an empty list if complete
pub fn check_locale_completeness(language: Language) -> Vec<String> {
    let mut missing = Vec::new();

    // Compare against Chinese (zh-Hans) as baseline
    let zhs_dir = match LOCALES_DIR.get_dir("zh-Hans") {
        Some(dir) => dir,
        None => return vec!["Chinese (zh-Hans) locale directory not found".to_string()],
    };

    let lang_dir = match LOCALES_DIR.get_dir(language.locale_code()) {
        Some(dir) => dir,
        None => {
            return vec![format!(
                "Locale directory '{}' not found",
                language.locale_code()
            )];
        }
    };

    // Compare the file lists
    let zhs_files: Vec<_> = zhs_dir
        .files()
        .map(|f| {
            f.path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    let lang_files: Vec<_> = lang_dir
        .files()
        .map(|f| {
            f.path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    for zhs_file in &zhs_files {
        if !lang_files.contains(zhs_file) {
            missing.push(zhs_file.clone());
        }
    }

    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_locales_dir_exists() -> Result<()> {
        // Check that the Chinese (zh-Hans) locale directory exists
        assert!(LOCALES_DIR.get_dir("zh-Hans").is_some());
        Ok(())
    }

    #[test]
    fn test_get_locale_dir() -> Result<()> {
        assert!(get_locale_dir(Language::ZHS).is_some());
        assert!(get_locale_dir(Language::En).is_some());
        Ok(())
    }

    #[test]
    fn test_all_locale_dirs() -> Result<()> {
        let all = get_all_locale_dirs();
        assert!(!all.is_empty());
        Ok(())
    }

    // ── Agent-error catalog completeness ───────────────────────────

    /// Compile-time exhaustiveness probe for `AgentErrorCode`: no wildcard
    /// arm, so a variant added to the enum fails to compile here until it
    /// gets an arm AND a line in the census below. This is what makes the
    /// catalog tests unable to silently skip a code.
    fn probe_agent_error_code(code: &plana_core::AgentErrorCode) {
        use plana_core::AgentErrorCode as C;
        match code {
            C::ModelNoProviders
            | C::ModelNoModels
            | C::ModelTierMismatch
            | C::ModelAllExcluded
            | C::ModelEnvIncomplete
            | C::ModelSelectionRetryExhausted
            | C::ModelNoCapableModel
            | C::LlmCallFailed
            | C::LlmEmptyResponse
            | C::LlmRateLimited
            | C::LlmAuthFailed
            | C::LlmTimeout
            | C::CosmosNoConnection
            | C::CosmosToolFailed
            | C::CosmosLocalUnavailable
            | C::ChainMaxDepth
            | C::ChainCycle
            | C::ChainFailed
            | C::SkillFailed
            | C::SkillEmptyOutput
            | C::SkillMissingReport
            | C::DependencyDown
            | C::AgentBusy
            | C::BudgetExhausted => {}
        }
    }

    /// Every variant in declaration order (keep in sync with the probe).
    fn all_agent_error_codes() -> Vec<plana_core::AgentErrorCode> {
        use plana_core::AgentErrorCode as C;
        let all = vec![
            C::ModelNoProviders,
            C::ModelNoModels,
            C::ModelTierMismatch,
            C::ModelAllExcluded,
            C::ModelEnvIncomplete,
            C::ModelSelectionRetryExhausted,
            C::ModelNoCapableModel,
            C::LlmCallFailed,
            C::LlmEmptyResponse,
            C::LlmRateLimited,
            C::LlmAuthFailed,
            C::LlmTimeout,
            C::CosmosNoConnection,
            C::CosmosToolFailed,
            C::CosmosLocalUnavailable,
            C::ChainMaxDepth,
            C::ChainCycle,
            C::ChainFailed,
            C::SkillFailed,
            C::SkillEmptyOutput,
            C::SkillMissingReport,
            C::DependencyDown,
            C::AgentBusy,
            C::BudgetExhausted,
        ];
        for code in &all {
            probe_agent_error_code(code);
        }
        all
    }

    /// The `{{name}}` placeholders a catalog value interpolates.
    fn placeholders(value: &str) -> std::collections::BTreeSet<String> {
        let mut set = std::collections::BTreeSet::new();
        let mut rest = value;
        while let Some(start) = rest.find("{{") {
            let after = &rest[start + 2..];
            match after.find("}}") {
                Some(end) => {
                    set.insert(after[..end].trim().to_string());
                    rest = &after[end + 2..];
                }
                None => break,
            }
        }
        set
    }

    /// The embedded `[errors.agent]` table of every supported locale, keyed
    /// by locale code. Reads the compile-time embedded copy, i.e. exactly
    /// the resources that ship at runtime.
    fn locale_agent_tables()
    -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> {
        let mut tables = std::collections::BTreeMap::new();
        for lang in Language::all() {
            let code = lang.locale_code();
            let dir = LOCALES_DIR
                .get_dir(code)
                .unwrap_or_else(|| panic!("locale dir {code} missing from embedded resources"));
            debug_assert!(!dir.entries().is_empty());
            // include_dir lookups match whole relative paths, not bare names.
            let file = LOCALES_DIR
                .get_file(format!("{code}/errors.toml"))
                .unwrap_or_else(|| {
                    panic!("locale dir {code} has no embedded errors.toml");
                });
            let raw = std::str::from_utf8(file.contents())
                .unwrap_or_else(|e| panic!("locale {code} errors.toml is not UTF-8: {e}"));
            let parsed: toml::Table = toml::from_str(raw)
                .unwrap_or_else(|e| panic!("locale {code} errors.toml does not parse: {e}"));
            let agent = parsed
                .get("errors")
                .and_then(|errors| errors.get("agent"))
                .and_then(|agent| agent.as_table())
                .unwrap_or_else(|| panic!("locale {code} errors.toml has no [errors.agent] table"))
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_str()
                            .unwrap_or_else(|| {
                                panic!("locale {code} errors.agent.{k} is not a string")
                            })
                            .to_string(),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            tables.insert(code.to_string(), agent);
        }
        tables
    }

    /// Every `AgentErrorCode` variant must have an entry in every locale's
    /// `[errors.agent]` table, keyed by its serde wire name.
    #[test]
    fn every_agent_error_code_has_an_entry_in_every_locale() -> Result<()> {
        let tables = locale_agent_tables();
        assert_eq!(tables.len(), Language::all().len());
        let mut failures = Vec::new();
        for code in all_agent_error_codes() {
            let wire = serde_json::to_string(&code).unwrap();
            let key = wire.trim_matches('"');
            for (locale, table) in &tables {
                if !table.contains_key(key) {
                    failures.push(format!("{locale} is missing errors.agent.{key}"));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "agent-error catalog is incomplete:\n  {}",
            failures.join("\n  ")
        );
        Ok(())
    }

    /// Every `[errors.agent]` entry must interpolate exactly the same
    /// placeholder set as the English entry for the same key, so no locale
    /// renders a raw `{{placeholder}}` or drops an interpolation the UI
    /// passes in.
    #[test]
    fn every_agent_error_entry_matches_the_en_placeholder_set() -> Result<()> {
        let tables = locale_agent_tables();
        let en = tables.get("en").unwrap_or_else(|| panic!("no en table"));
        let mut failures = Vec::new();
        for (locale, table) in &tables {
            if locale == "en" {
                continue;
            }
            for (key, value) in table {
                let en_placeholders = en.get(key).map(|v| placeholders(v));
                let locale_placeholders = placeholders(value);
                match en_placeholders {
                    None => failures.push(format!(
                        "{locale} has errors.agent.{key} but en does not"
                    )),
                    Some(expected) if expected != locale_placeholders => failures.push(format!(
                        "{locale} errors.agent.{key} placeholders {locale_placeholders:?} != en {expected:?}"
                    )),
                    Some(_) => {}
                }
            }
        }
        assert!(
            failures.is_empty(),
            "agent-error placeholder parity is broken:\n  {}",
            failures.join("\n  ")
        );
        Ok(())
    }
}
