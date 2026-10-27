//! Provider/API-key constants and validation helpers (moved from
//! plana_core, T1 decoupling 2026-10-27 — config-domain values that do not
//! belong to the platform-vocabulary crate).

/// Default Docker network name for the platform stack.
pub const DEFAULT_NETWORK: &str = "entelecheia-network";

/// An API key that is empty/whitespace is "invalid" (not yet configured).
pub fn is_invalid_api_key(api_key: &str) -> bool {
    api_key.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_api_keys_are_invalid() {
        assert!(is_invalid_api_key(""));
        assert!(is_invalid_api_key("   "));
        assert!(is_invalid_api_key("\t\n"));
        assert!(!is_invalid_api_key("sk-abc123"));
        assert!(!is_invalid_api_key(" key "));
    }
}
