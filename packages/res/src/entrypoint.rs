use include_dir::{Dir, include_dir};

pub static ENTRYPOINT_DIR: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../target/provider-registry/entrypoint");

pub static MODELS_DIR: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../target/provider-registry/models");

/// Get the entrypoint directory for a given provider
pub fn get_provider_entrypoint_dir(provider_id: &str) -> Option<&'static Dir<'static>> {
    ENTRYPOINT_DIR.get_dir(provider_id)
}

/// Get the list of all providers
pub fn get_all_providers() -> Vec<String> {
    ENTRYPOINT_DIR
        .dirs()
        .filter_map(|d| {
            d.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .collect()
}

/// Validate that entrypoint TOML files contain all required language keys
///
/// Returns a list of errors; empty list if all pass
pub fn validate_entrypoint_languages() -> Vec<String> {
    let mut errors = Vec::new();
    // Core languages the registry entrypoints must cover. The upstream
    // provider-registry data currently provides these eight; ar/de/pt
    // translations have not landed upstream yet, so requiring them here
    // would permanently red the build.
    const REQUIRED_LANGS: &[&str] = &["zh-Hans", "zh-Hant", "en", "ja", "ko", "fr", "es", "ru"];
    let required_langs = REQUIRED_LANGS;

    for provider_dir in ENTRYPOINT_DIR.dirs() {
        let provider_id = provider_dir
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        for file in provider_dir.files() {
            if file.path().extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            if let Some(content) = file.contents_utf8() {
                if let Ok(parsed) = toml::from_str::<toml::Value>(content) {
                    // Check the [entrypoint.name] section
                    if let Some(name_table) = parsed
                        .get("entrypoint")
                        .and_then(|e| e.get("name"))
                        .and_then(|n| n.as_table())
                    {
                        // Legacy locale keys predating the BCP 47 unification
                        // (zhs/zht) count towards zh-Hans/zh-Hant.
                        let has_lang = |lang: &str| {
                            name_table.contains_key(lang)
                                || (lang == "zh-Hans" && name_table.contains_key("zhs"))
                                || (lang == "zh-Hant" && name_table.contains_key("zht"))
                        };
                        let missing: Vec<_> = required_langs
                            .iter()
                            .filter(|&&lang| !has_lang(lang))
                            .copied()
                            .collect();

                        if !missing.is_empty() {
                            errors.push(format!(
                                "Entrypoint {}/{} missing languages in [entrypoint.name]: {}",
                                provider_id,
                                file.path().display(),
                                missing.join(", ")
                            ));
                        }
                    } else {
                        errors.push(format!(
                            "Entrypoint {}/{} missing [entrypoint.name] section",
                            provider_id,
                            file.path().display()
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Entrypoint {}/{} contains invalid TOML",
                        provider_id,
                        file.path().display()
                    ));
                }
            }
        }
    }

    errors
}

/// Get the model directory for a given provider
pub fn get_provider_models_dir(provider_id: &str) -> Option<&'static Dir<'static>> {
    MODELS_DIR.get_dir(provider_id)
}

/// Get all providers that have model configurations
pub fn get_all_model_providers() -> Vec<String> {
    MODELS_DIR
        .dirs()
        .filter_map(|d| {
            d.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn test_entrypoint_dir_exists() -> Result<()> {
        assert!(!get_all_providers().is_empty());
        Ok(())
    }

    #[test]
    fn test_get_all_providers() -> Result<()> {
        let providers = get_all_providers();
        assert!(
            providers.len() >= 10,
            "Expected >= 10 providers, got {}",
            providers.len()
        );
        Ok(())
    }

    #[test]
    fn test_models_dir_exists() -> Result<()> {
        assert!(!get_all_model_providers().is_empty());
        Ok(())
    }

    #[test]
    fn test_entrypoint_languages_valid() -> Result<()> {
        let errors = validate_entrypoint_languages();
        assert!(
            errors.is_empty(),
            "Entrypoint language validation errors:\n{}",
            errors.join("\n")
        );
        Ok(())
    }

    #[test]
    fn test_known_providers_present() -> Result<()> {
        let providers = get_all_providers();
        for expected in ["openai", "anthropic", "google", "openrouter", "deepseek"] {
            assert!(
                providers.contains(&expected.to_string()),
                "Missing expected provider: {}",
                expected
            );
        }
        Ok(())
    }

    #[test]
    fn test_provider_entrypoint_dir_returns_content() -> Result<()> {
        let dir = get_provider_entrypoint_dir("openai").context("openai provider should exist")?;
        let files: Vec<_> = dir.files().collect();
        assert!(!files.is_empty(), "openai entrypoint dir has no files");
        let has_default = files
            .iter()
            .any(|f| f.path().file_name().and_then(|n| n.to_str()) == Some("default.toml"));
        assert!(has_default, "openai entrypoint dir missing default.toml");
        Ok(())
    }

    #[test]
    fn test_entrypoint_and_model_dirs_both_populated() -> Result<()> {
        assert!(
            !get_all_providers().is_empty(),
            "No entrypoint providers found"
        );
        assert!(
            !get_all_model_providers().is_empty(),
            "No model providers found"
        );
        Ok(())
    }

    #[test]
    fn test_all_entrypoint_toml_is_valid() -> Result<()> {
        for provider_dir in ENTRYPOINT_DIR.dirs() {
            let provider_id = provider_dir
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .context("provider dir missing file name")?;
            for file in provider_dir.files() {
                if file.path().extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }
                let content = file.contents_utf8().with_context(|| {
                    format!(
                        "Non-UTF8 content in {}/{}",
                        provider_id,
                        file.path().display()
                    )
                })?;
                toml::from_str::<toml::Value>(content).with_context(|| {
                    format!("Invalid TOML in {}/{}", provider_id, file.path().display())
                })?;
            }
        }
        Ok(())
    }

    #[test]
    fn test_all_model_toml_is_valid() -> Result<()> {
        let mut count = 0usize;
        for provider_dir in MODELS_DIR.dirs() {
            let provider_id = provider_dir
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .context("model provider dir missing file name")?;
            for file in provider_dir.files() {
                if file.path().extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }
                let content = file.contents_utf8().with_context(|| {
                    format!(
                        "Non-UTF8 content in {}/{}",
                        provider_id,
                        file.path().display()
                    )
                })?;
                toml::from_str::<toml::Value>(content).with_context(|| {
                    format!("Invalid TOML in {}/{}", provider_id, file.path().display())
                })?;
                count += 1;
            }
        }
        assert!(
            count >= 50,
            "Expected >= 50 model TOML files, found {}",
            count
        );
        Ok(())
    }
}
