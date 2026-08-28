//! Client-side vocabulary for the evernight terminal route: JSON-RPC method
//! names and the request-scoped authentication field.
//!
//! Specification: `docs/zh-Hans/designs/polemos-evernight-terminal-routing.md`
//! in the entelecheia repository (P83/M1 routing contract) — §4.1 for the
//! method surface and §3.2 / §7 decision two for the static-token model.
//!
//! Token discipline (hard rule): the shared secret exists only as a
//! process-environment value held in memory. It never lands in git, logs or
//! docs; every checked-in appearance is the literal placeholder
//! `<your-evernight-token>`.

/// JSON-RPC method for one-shot shell command execution (`Command.Exec`).
///
/// The broker executes the single `command` string with `sh -c` and answers
/// with `{exit_code, stdout, stderr}` (spec §4.1, fact F6). Structured argv
/// is *not* part of the protocol and must not be invented here.
pub const METHOD_COMMAND_EXEC: &str = "Command.Exec";

/// JSON-RPC method used as the connectivity probe against the broker
/// (`System.Ping`). The deployment runbook uses one ping to verify the route
/// after a token rotation (spec §7 decision two, rotation memo).
pub const METHOD_SYSTEM_PING: &str = "System.Ping";

/// Top-level params field that carries the request-scoped shared token when
/// one is configured. The broker is expected to strip this field before the
/// params reach the `Command.Exec` handler; it is never part of
/// [`crate::envelope::DispatchEnvelope::wire_params`] itself.
pub const AUTH_PARAM_KEY: &str = "auth";

/// Environment variable through which the deployment injects the static
/// shared token (spec §3.2 / §7 decision two).
pub const ENV_TOKEN_VAR: &str = "EVERNIGHT_TOKEN";

/// Reads the static token from the process environment.
///
/// Returns `None` when [`ENV_TOKEN_VAR`] is unset or empty. The returned
/// string is held in memory only — callers must not persist or log it.
pub fn token_from_env() -> Option<String> {
    std::env::var(ENV_TOKEN_VAR)
        .ok()
        .filter(|token| !token.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_match_the_evernight_surface() {
        assert_eq!(METHOD_COMMAND_EXEC, "Command.Exec");
        assert_eq!(METHOD_SYSTEM_PING, "System.Ping");
    }

    #[test]
    fn auth_param_key_is_request_scoped_auth() {
        assert_eq!(AUTH_PARAM_KEY, "auth");
    }

    #[test]
    fn env_var_name_is_the_documented_convention() {
        assert_eq!(ENV_TOKEN_VAR, "EVERNIGHT_TOKEN");
    }

    #[test]
    fn token_from_env_reads_and_filters_empty() {
        // Use a variable name we fully control: ENV_TOKEN_VAR is a compile-time
        // constant, so mutate the real one and restore it afterwards.
        unsafe {
            std::env::set_var(ENV_TOKEN_VAR, "<your-evernight-token>");
        }
        assert_eq!(token_from_env().as_deref(), Some("<your-evernight-token>"));
        unsafe {
            std::env::set_var(ENV_TOKEN_VAR, "");
        }
        assert_eq!(token_from_env(), None);
        unsafe {
            std::env::remove_var(ENV_TOKEN_VAR);
        }
        assert_eq!(token_from_env(), None);
    }
}
