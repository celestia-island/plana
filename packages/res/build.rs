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
const SECTION_SKILLS: &str = "skills";
const FILE_EXT_MD_DOT: &str = ".md";

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
    println!("cargo:rerun-if-changed=../../res/prompts");
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

    log_info!("Checking Markdown documentation...");
    match check_markdown_docs() {
        Ok(_) => log_ok!("Markdown documentation validation: PASSED"),
        Err(errors) => {
            has_errors = true;
            error_count += errors.len();
            for error in &errors {
                log_err!("{}", error);
            }
        }
    }

    log_info!("Checking soul front matter invariants...");
    match check_soul_docs() {
        Ok(_) => log_ok!("Soul front matter validation: PASSED"),
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
        return Err(vec!["Chinese (zh-Hans) locale directory not found".to_string()]);
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
                        let missing: Vec<_> = REQUIRED_LANGS
                            .iter()
                            .filter(|&&lang| !name_table.contains_key(lang))
                            .copied()
                            .collect();

                        if !missing.is_empty() {
                            errors.push(format!(
                                "{}/{}: missing languages in [entrypoint.name]: {}",
                                provider_id,
                                toml_path.file_name().unwrap_or_default().to_string_lossy(),
                                missing.join(", ")
                            ));
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

fn check_markdown_docs() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let docs_dirs = [
        Path::new("../../res/prompts/agents"),
        Path::new("../../res/prompts/domain_agents"),
    ];

    for docs_dir in &docs_dirs {
        if !docs_dir.exists() {
            continue;
        }

        let agent_entries = match docs_dir.read_dir() {
            Ok(entries) => entries,
            Err(e) => {
                errors.push(format!(
                    "Failed to read docs directory {:?}: {}",
                    docs_dir, e
                ));
                continue;
            }
        };

        for agent_entry in agent_entries {
            let agent_entry = match agent_entry {
                Ok(e) => e,
                Err(e) => {
                    errors.push(format!("Failed to read agent entry: {}", e));
                    continue;
                }
            };
            let agent_path = agent_entry.path();

            if !agent_path.is_dir() {
                continue;
            }

            let agent_name = agent_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            for section_name in ["skills", "mcp"] {
                let section_path = agent_path.join(section_name);
                if !section_path.exists() {
                    continue;
                }

                let section_entries = match section_path.read_dir() {
                    Ok(entries) => entries,
                    Err(e) => {
                        errors.push(format!(
                            "Failed to read section directory {:?}: {}",
                            section_path, e
                        ));
                        continue;
                    }
                };

                for item_entry in section_entries {
                    let item_entry = match item_entry {
                        Ok(e) => e,
                        Err(e) => {
                            errors.push(format!("Failed to read item entry: {}", e));
                            continue;
                        }
                    };
                    let item_path = item_entry.path();

                    let item_name = match item_path.file_name().and_then(|n| n.to_str()) {
                        Some(name) => name,
                        None => continue,
                    };

                    if !item_name.ends_with(FILE_EXT_MD_DOT) {
                        continue;
                    }

                    if item_path.is_dir() {
                        continue;
                    }

                    let skill_name = item_name.strip_suffix(FILE_EXT_MD_DOT).unwrap_or(item_name);

                    if let Ok(content) = fs::read_to_string(&item_path)
                        && let Some(front_matter) = extract_front_matter(&content)
                    {
                        match toml::from_str::<toml::Value>(&front_matter) {
                            Ok(parsed) => {
                                if section_name == SECTION_SKILLS {
                                    for field in ["name", "agent"] {
                                        if parsed.get(field).and_then(|v| v.as_str()).is_none() {
                                            errors.push(format!(
                                                "Agent {}/{}/{}: missing '{}' in front matter",
                                                agent_name, section_name, skill_name, field
                                            ));
                                        }
                                    }
                                    if parsed.get("description").is_none() {
                                        errors.push(format!(
                                            "Agent {}/{}/{}: missing 'description' in front matter",
                                            agent_name, section_name, skill_name
                                        ));
                                    }

                                    if let Some(declared_agent) =
                                        parsed.get("agent").and_then(|v| v.as_str())
                                        && normalize_agent_name(declared_agent)
                                            != normalize_agent_name(agent_name)
                                    {
                                        errors.push(format!(
                                                    "Agent {}/{}/{}: front matter agent '{}' does not match directory '{}'",
                                                    agent_name,
                                                    section_name,
                                                    skill_name,
                                                    declared_agent,
                                                    agent_name
                                                ));
                                    }
                                }
                            }
                            Err(_) => {
                                errors.push(format!(
                                    "Agent {}/{}/{}: invalid front matter TOML",
                                    agent_name, section_name, skill_name
                                ));
                            }
                        }
                    }
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

fn check_soul_docs() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let soul_dir = Path::new("../../res/prompts/soul");
    if !soul_dir.exists() {
        errors.push("Soul directory not found: res/prompts/soul".to_string());
        return Err(errors);
    }

    let Ok(files) = soul_dir.read_dir() else {
        errors.push("Failed to read soul directory".to_string());
        return Err(errors);
    };

    for file in files.flatten() {
        let path = file.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let agent = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };

        if let Some(front_matter) = extract_front_matter(&content) {
            match toml::from_str::<toml::Value>(&front_matter) {
                Ok(parsed) => {
                    if parsed.get("preferred_language").is_some() {
                        errors.push(format!(
                            "Soul {}: preferred_language must not live in soul front matter",
                            agent
                        ));
                    }
                }
                Err(err) => errors.push(format!(
                    "Soul {}: invalid front matter TOML: {}",
                    agent, err
                )),
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_agent_name(value: &str) -> Option<String> {
    let candidate = value.trim().to_lowercase().replace([' ', '-', '_'], "");

    KNOWN_AGENTS
        .iter()
        .find(|&&agent| agent == candidate)
        .map(|agent| agent.to_string())
}

fn extract_front_matter(content: &str) -> Option<String> {
    let start = content.find("+++")?;
    let after_first = start + 3;
    let rest = content.get(after_first..)?;
    let end_in_rest = rest.find("+++")?;
    let toml_text = rest[..end_in_rest].trim();
    Some(toml_text.to_string())
}

// ── about/protocol doc bundling from arona ────────────────────────────

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
