use include_dir::{Dir, include_dir};

use crate::Language;

/// Compile-time embedded locale directories for all languages
///
/// Path relative to packages/res, resolved from CARGO_MANIFEST_DIR
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

    // Compare against Chinese (zhs) as baseline
    let zhs_dir = match LOCALES_DIR.get_dir("zhs") {
        Some(dir) => dir,
        None => return vec!["Chinese (zhs) locale directory not found".to_string()],
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
        // Check that the Chinese (zhs) locale directory exists
        assert!(LOCALES_DIR.get_dir("zhs").is_some());
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
}
