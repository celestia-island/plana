//! Namespace-prefix whitelist routing.

use super::names::{namespace_of, RELAY};

/// What the bridge does with an inbound JSON-RPC request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// Execute a registered local handler (all `tauri.*` methods, plus
    /// any app-mounted local namespaces).
    Local,
    /// Forward to the named endpoint (an entrypoint the app opened via
    /// `tauri.conn.open`).
    Forward(String),
    /// No whitelist entry matched — deny with `-32601` semantics. The
    /// bridge never guesses an endpoint for an unlisted namespace.
    Deny,
}

/// The forwarding whitelist: namespace prefix → endpoint name. Longest
/// prefix wins (`device.internal.*` can override `device.*`).
///
/// The `tauri` namespace is structurally reserved: entries for it are
/// rejected at insertion so routing can never be talked into forwarding
/// a system-control method to a server.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ForwardingTable {
    entries: Vec<(String, String)>,
}

impl ForwardingTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow `namespace_prefix.*` on `endpoint`. Returns `Err` when the
    /// prefix would capture the reserved `tauri` namespace.
    pub fn allow(&mut self, namespace_prefix: &str, endpoint: &str) -> Result<(), String> {
        let trimmed = namespace_prefix.trim();
        // A bare namespace ("device") covers its whole subtree; a dotted
        // prefix ("device.internal") narrows it. Either way the FIRST
        // segment decides reservation.
        let head = trimmed.split('.').next().unwrap_or_default();
        if head.is_empty() {
            return Err(format!("'{namespace_prefix}' carries no namespace"));
        }
        if head == RELAY {
            return Err(format!(
                "the '{RELAY}' namespace is reserved for local system control"
            ));
        }
        self.entries
            .push((trimmed.to_string(), endpoint.to_string()));
        Ok(())
    }

    /// Decide the route for `method`.
    pub fn route(&self, method: &str) -> Route {
        if namespace_of(method) == RELAY {
            return Route::Local;
        }
        // Longest matching prefix first; ties resolve to the last edit
        // (latest mapping wins, mirroring how config layers read).
        let mut best: Option<(usize, &str)> = None;
        for (prefix, endpoint) in &self.entries {
            let matches = method == prefix.as_str()
                || method
                    .strip_prefix(prefix.as_str())
                    .is_some_and(|rest| rest.starts_with('.'));
            if matches {
                let len = prefix.len();
                if best.is_none_or(|(blen, _)| len >= blen) {
                    best = Some((len, endpoint));
                }
            }
        }
        match best {
            Some((_, endpoint)) => Route::Forward(endpoint.to_string()),
            None => Route::Deny,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tauri_namespace_always_local() {
        let mut table = ForwardingTable::new();
        table.allow("net", "public").unwrap();
        assert_eq!(table.route("relay.net.set_proxy"), Route::Local);
    }

    #[test]
    fn tauri_cannot_be_captured() {
        let mut table = ForwardingTable::new();
        assert!(table.allow("relay", "evil").is_err());
        assert!(table.allow("relay.evil", "evil").is_err());
    }

    #[test]
    fn longest_prefix_wins_and_unlisted_denied() {
        let mut table = ForwardingTable::new();
        table.allow("device", "gateway").unwrap();
        table.allow("device.internal", "lab").unwrap();
        assert_eq!(
            table.route("device.register"),
            Route::Forward("gateway".into())
        );
        assert_eq!(
            table.route("device.internal.diag"),
            Route::Forward("lab".into())
        );
        assert_eq!(table.route("enrollment.mint"), Route::Deny);
        // Exact-prefix boundary: `deviceX.*` must not match `device`.
        assert_eq!(table.route("deviceX.anything"), Route::Deny);
    }

    #[test]
    fn latest_edit_wins_on_tie() {
        let mut table = ForwardingTable::new();
        table.allow("device", "gateway").unwrap();
        table.allow("device", "gateway-2").unwrap();
        assert_eq!(table.route("device.x"), Route::Forward("gateway-2".into()));
    }
}
