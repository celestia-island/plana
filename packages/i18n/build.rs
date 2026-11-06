use std::{collections::HashSet, error::Error, fs, path::Path};

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

fn main() {
    println!("cargo:rerun-if-changed=../../res/i18n");

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

    log_blank!();
    if has_errors {
        log_err!("i18n resource validation FAILED with {} error(s)", error_count);
        log_info!("Please fix the issues above before proceeding.");
        eprintln!("i18n resource validation failed. See warnings above for details.");
        std::process::exit(1);
    } else {
        log_ok!("i18n resource validation PASSED!");
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
