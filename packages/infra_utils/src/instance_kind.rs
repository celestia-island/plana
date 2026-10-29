//! Instance-kind resolution — "which deployment lane am I on" read from a
//! named env var, with the vocabulary and key as caller parameters (chest's
//! `CHEST_INSTANCE_KIND` and scepter's `ENTELECHIA_KIND` share the concept
//! but not the key; the family convention keeps one implementation).
//!
//! Semantics (mirroring both services): trim and case-fold the value,
//! accept it when it is in the caller's vocabulary, otherwise fail **loudly
//! but open** — a one-time warning per env key names the offending value,
//! and the answer degrades to the caller's fallback with
//! [`ResolvedInstanceKind::from_env == false`] so the degraded state stays
//! observable. Zero web-framework dependencies.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// One resolved instance kind plus the observability marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedInstanceKind {
    /// The effective kind word: the lowercased env value when it matched
    /// the vocabulary, otherwise the caller's fallback.
    pub kind: String,
    /// `true` when the env var supplied a vocabulary word; `false` means
    /// the fallback answered (var unset, blank, or an unknown value).
    pub from_env: bool,
}

/// Resolve the instance kind from the named env var.
///
/// - var **unset or blank** → the fallback, `from_env = false`;
/// - value (trimmed, case-folded) **in `known`** → that word,
///   `from_env = true`;
/// - value **outside the vocabulary** → warn once per env key (naming the
///   offending value — miswiring stays visible without taking health
///   down), answer the fallback, `from_env = false`.
pub fn resolve_instance_kind(
    env_key: &str,
    known: &[&str],
    unknown_fallback: &str,
) -> ResolvedInstanceKind {
    let raw = std::env::var(env_key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let Some(raw) = raw else {
        return ResolvedInstanceKind {
            kind: unknown_fallback.to_string(),
            from_env: false,
        };
    };
    let candidate = raw.to_ascii_lowercase();
    if known.iter().any(|k| k.eq_ignore_ascii_case(&candidate)) {
        return ResolvedInstanceKind {
            kind: candidate,
            from_env: true,
        };
    }
    warn_unknown_once(env_key, &raw);
    ResolvedInstanceKind {
        kind: unknown_fallback.to_string(),
        from_env: false,
    }
}

/// One warning per env key per process (mirrors scepter's once-only
/// unknown-value alarm — repeated requests must not spam the log).
fn warn_unknown_once(env_key: &str, value: &str) {
    static WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let warned = WARNED.get_or_init(|| Mutex::new(HashSet::new()));
    let mut guard = warned.lock().expect("instance-kind warn set lock");
    if guard.insert(env_key.to_string()) {
        tracing::warn!(
            env = %env_key,
            value = %value,
            "unknown instance kind — falling back to the caller's default"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "T3SIB_INSTANCE_KIND";

    fn set_env(value: Option<&str>) {
        match value {
            Some(v) => unsafe { std::env::set_var(KEY, v) },
            None => unsafe { std::env::remove_var(KEY) },
        }
    }

    #[test]
    fn known_vocabulary_word_is_accepted_case_and_space_tolerant() {
        for raw in ["prod", "  PROD ", "\tProd\n"] {
            set_env(Some(raw));
            let resolved = resolve_instance_kind(KEY, &["prod", "test", "dev"], "dev");
            assert_eq!(resolved.kind, "prod");
            assert!(resolved.from_env);
        }
        set_env(None);
    }

    #[test]
    fn unset_or_blank_degrades_to_the_fallback() {
        set_env(None);
        let resolved = resolve_instance_kind(KEY, &["prod", "test", "dev"], "dev");
        assert_eq!(resolved.kind, "dev");
        assert!(!resolved.from_env);
        set_env(Some("   "));
        let resolved = resolve_instance_kind(KEY, &["prod", "test", "dev"], "prod");
        assert_eq!(resolved.kind, "prod");
        assert!(!resolved.from_env);
        set_env(None);
    }

    #[test]
    fn unknown_value_degrades_loudly_with_the_marker() {
        set_env(Some("staging"));
        let resolved = resolve_instance_kind(KEY, &["prod", "test", "dev"], "dev");
        assert_eq!(resolved.kind, "dev");
        assert!(!resolved.from_env);
        set_env(None);
    }

    #[test]
    fn vocabulary_is_a_caller_parameter() {
        set_env(Some("stage"));
        let resolved = resolve_instance_kind(KEY, &["stage", "live"], "live");
        assert_eq!(resolved.kind, "stage");
        assert!(resolved.from_env);
        set_env(None);
    }
}
