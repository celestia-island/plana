use std::{
    collections::HashSet,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

include!("src/agent_names.rs");

const REQUIRED_LANGS: &[&str] = &[
    "zh-Hans", "en", "zh-Hant", "ja", "ko", "fr", "es", "ru", "ar", "de", "pt",
];

macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("cargo:warning=\x1b[1;36m[ INFO ]\x1b[0m {}", format!($($arg)*))
    };
}

macro_rules! log_ok {
    ($($arg:tt)*) => {
        println!("cargo:warning=\x1b[1;32m[  OK  ]\x1b[0m {}", format!($($arg)*))
    };
}

macro_rules! log_err {
    ($($arg:tt)*) => {
        println!("cargo:warning=\x1b[1;31m[ ERR  ]\x1b[0m {}", format!($($arg)*))
    };
}

macro_rules! log_blank {
    () => {
        println!("cargo:warning=")
    };
}

/// Ensure the directories referenced by `include_dir!` in src/entrypoint.rs
/// exist. Inside the arona workspace they are populated by provider-registry;
/// outside (e.g. evernight via git dep) they are absent, so we create empty
/// placeholders to keep the compile-time macro happy. The embedded content is
/// empty in that case — harmless because the data is arona-internal.
fn ensure_include_dir_targets() {
    for rel in [
        "../../target/provider-registry/entrypoint",
        "../../target/provider-registry/models",
    ] {
        let p = Path::new(rel);
        if !p.exists() {
            let _ = std::fs::create_dir_all(p);
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=../../res/i18n");
    println!("cargo:rerun-if-changed=../../target/provider-registry/entrypoint");
    println!("cargo:rerun-if-env-changed=ARONA_ROOT");

    // When `_res` is consumed from outside the arona workspace (e.g. evernight
    // via a git dependency), the provider-registry output directories do not
    // exist — they are generated inside arona only. The `include_dir!` macro
    // in src/entrypoint.rs is a compile-time requirement and panics if the
    // directory is missing. Create empty placeholder directories so the macro
    // succeeds (the embedded content is simply empty); downstream callers
    // that touch ENTRYPOINT_DIR / MODELS_DIR must already tolerate emptiness.
    ensure_include_dir_targets();

    bundle_about_docs();

    let mut has_errors = false;
    let mut error_count = 0;

    log_info!("Checking i18n TOML key consistency...");
    match check_i18n_toml_keys() {
        Ok(_) => log_ok!("i18n TOML keys validation: PASSED"),
        Err(errors) => {
            has_errors = true;
            error_count += errors.len();
            for error in &errors {
                log_err!("{}", error);
            }
        }
    }

    log_info!("Checking entrypoint language completeness...");
    match check_entrypoint_languages() {
        Ok(_) => log_ok!("Entrypoint language validation: PASSED"),
        Err(errors) => {
            has_errors = true;
            error_count += errors.len();
            for error in &errors {
                log_err!("{}", error);
            }
        }
    }

    log_blank!();
    if has_errors {
        log_err!("Resource validation FAILED with {} error(s)", error_count);
        log_info!("Please fix the issues above before proceeding.");
        eprintln!("Resource validation failed. See warnings above for details.");
        std::process::exit(1);
    } else {
        log_ok!("All resource validations PASSED!");
    }
}

fn check_i18n_toml_keys() -> Result<(), Vec<String>> {
    let locales_path = Path::new("../../res/i18n/locales");
    let zhs_path = locales_path.join("zh-Hans");
    let mut errors = Vec::new();

    if !zhs_path.exists() {
        return Err(vec!["Chinese (zhs) locale directory not found".to_string()]);
    }

    let zhs_fields = collect_toml_fields(&zhs_path)
        .map_err(|e| vec![format!("Failed to collect Chinese fields: {}", e)])?;

    for &lang in &REQUIRED_LANGS[1..] {
        let lang_path = locales_path.join(lang);
        if !lang_path.exists() {
            errors.push(format!("Language directory '{}' not found", lang));
            continue;
        }

        let lang_fields = match collect_toml_fields(&lang_path) {
            Ok(fields) => fields,
            Err(e) => {
                errors.push(format!("Failed to collect fields for '{}': {}", lang, e));
                continue;
            }
        };

        let missing: Vec<_> = zhs_fields
            .difference(&lang_fields)
            .map(|s| s.as_str())
            .collect();

        if !missing.is_empty() {
            errors.push(format!(
                "Language '{}': missing fields: {}",
                lang,
                missing.join(", ")
            ));
        }

        let extra: Vec<_> = lang_fields
            .difference(&zhs_fields)
            .map(|s| s.as_str())
            .collect();

        if !extra.is_empty() {
            errors.push(format!(
                "Language '{}': extra fields (not in Chinese): {}",
                lang,
                extra.join(", ")
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_toml_fields(dir: &Path) -> Result<HashSet<String>, Box<dyn Error>> {
    let mut fields = HashSet::new();

    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.starts_with('[') || line.is_empty() {
                continue;
            }

            if let Some(eq_pos) = line.find('=') {
                let field_name = line[..eq_pos].trim().to_string();
                if !field_name.is_empty() {
                    fields.insert(field_name);
                }
            }
        }
    }

    Ok(fields)
}

fn check_entrypoint_languages() -> Result<(), Vec<String>> {
    let entrypoint_path = Path::new("../../target/provider-registry/entrypoint");
    let mut errors = Vec::new();

    if !entrypoint_path.exists() {
        // The entrypoint directory is generated by the provider-registry step
        // and only exists inside the arona workspace. When `_res` is consumed
        // from another workspace (e.g. evernight via a git dependency) the
        // directory is legitimately absent — downgrade to a warning so the
        // build does not fail. The language-completeness check is a no-op in
        // that case.
        println!(
            "cargo:warning=[ INFO ] Entrypoint directory not found ({:?}) — \
             skipping language-completeness check (expected outside arona workspace)",
            entrypoint_path
        );
        return Ok(());
    }

    let provider_dirs = fs::read_dir(entrypoint_path)
        .map_err(|e| vec![format!("Failed to read entrypoint directory: {}", e)])?;

    for provider_entry in provider_dirs {
        let provider_entry =
            provider_entry.map_err(|e| vec![format!("Failed to read provider entry: {}", e)])?;
        let provider_path = provider_entry.path();

        if !provider_path.is_dir() {
            continue;
        }

        let provider_id = provider_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let toml_files = fs::read_dir(&provider_path).map_err(|e| {
            vec![format!(
                "Failed to read provider directory {}: {}",
                provider_id, e
            )]
        })?;

        for toml_entry in toml_files {
            let toml_entry =
                toml_entry.map_err(|e| vec![format!("Failed to read toml entry: {}", e)])?;
            let toml_path = toml_entry.path();

            if toml_path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            let content = fs::read_to_string(&toml_path)
                .map_err(|e| vec![format!("Failed to read file {:?}: {}", toml_path, e)])?;

            match toml::from_str::<toml::Value>(&content) {
                Ok(parsed) => {
                    if let Some(name_table) = parsed
                        .get("entrypoint")
                        .and_then(|e| e.get("name"))
                        .and_then(|n| n.as_table())
                    {
                        // Legacy locale keys from before the BCP 47 unification.
                        let legacy_aliases: &[(&str, &str)] =
                            &[("zhs", "zh-Hans"), ("zht", "zh-Hant")];
                        let has_lang = |lang: &str| {
                            name_table.contains_key(lang)
                                || legacy_aliases
                                    .iter()
                                    .any(|(old, new)| *new == lang && name_table.contains_key(*old))
                        };
                        let missing: Vec<_> = REQUIRED_LANGS
                            .iter()
                            .filter(|&&lang| !has_lang(lang))
                            .copied()
                            .collect();

                        if !missing.is_empty() {
                            // The registry is synced from an external repository that
                            // may lag behind the BCP 47 locale unification; do not
                            // gate the build on data completeness, only warn.
                            println!(
                                "cargo:warning=[ WARN ] {}/{}: missing languages in [entrypoint.name]: {}",
                                provider_id,
                                toml_path.file_name().unwrap_or_default().to_string_lossy(),
                                missing.join(", ")
                            );
                        }
                    } else {
                        errors.push(format!(
                            "{}/{}: missing [entrypoint.name] section",
                            provider_id,
                            toml_path.file_name().unwrap_or_default().to_string_lossy()
                        ));
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "{}/{}: invalid TOML: {}",
                        provider_id,
                        toml_path.file_name().unwrap_or_default().to_string_lossy(),
                        e
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

const ABOUT_LANGS: &[&str] = &[
    "zh-Hans", "zh-Hant", "en", "ja", "ko", "fr", "es", "ru", "ar", "de", "pt",
];
const ABOUT_DOC_TYPES: &[&str] = &["license", "cla", "code-of-conduct", "security"];

fn bundle_about_docs() {
    let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_dir = Path::new(&out_dir);

    let arona = find_arona_root();
    if arona.is_some() {
        log_info!("Bundling about/protocol docs from arona...");
    } else {
        log_info!("arona not found — about docs will be empty (set ARONA_ROOT to enable)");
    }

    let mut code = String::new();
    code.push_str("// AUTO-GENERATED by build.rs — do not edit\n");
    code.push_str("// Source: arona/docs/licenses/<lang>/*.md\n\n");
    code.push_str("pub struct AboutDoc {\n");
    code.push_str("    pub license: &'static str,\n");
    code.push_str("    pub cla: &'static str,\n");
    code.push_str("    pub code_of_conduct: &'static str,\n");
    code.push_str("    pub security: &'static str,\n");
    code.push_str("}\n\n");
    code.push_str("pub fn about_docs_for(lang: &str) -> AboutDoc {\n");
    code.push_str("    match lang {\n");

    for &lang in ABOUT_LANGS {
        code.push_str(&format!("        \"{lang}\" => AboutDoc {{\n"));
        for &doc_type in ABOUT_DOC_TYPES {
            let content = arona
                .as_ref()
                .and_then(|root| {
                    let p = root
                        .join("docs")
                        .join("licenses")
                        .join(lang)
                        .join(format!("{doc_type}.md"));
                    fs::read_to_string(&p).ok()
                })
                .unwrap_or_default();
            let field = doc_type.replace('-', "_");
            code.push_str(&format!(
                "            {field}: {raw},\n",
                field = field,
                raw = raw_str(&content)
            ));
        }
        code.push_str("        },\n");
    }

    code.push_str("        _ => AboutDoc { license: \"\", cla: \"\", code_of_conduct: \"\", security: \"\" },\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    fs::write(out_dir.join("about_gen.rs"), code).expect("failed to write about_gen.rs");

    if arona.is_some() {
        log_ok!("About docs bundled for {} languages", ABOUT_LANGS.len());
    }
}

fn find_arona_root() -> Option<PathBuf> {
    if let Ok(root) = std::env::var("ARONA_ROOT") {
        let p = PathBuf::from(root);
        if p.join("docs/licenses").is_dir() {
            return Some(p);
        }
    }
    let manifest =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let sibling = manifest.join("../../../arona");
    if sibling.join("docs/licenses").is_dir() {
        return Some(sibling);
    }
    None
}

fn raw_str(content: &str) -> String {
    let mut hashes = 0usize;
    loop {
        let close = format!("\"{}", "#".repeat(hashes));
        if !content.contains(&close) {
            break;
        }
        hashes += 1;
    }
    let h = "#".repeat(hashes);
    format!("r{h}\"{content}\"{h}")
}
