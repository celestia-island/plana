//! Method-name namespaces and the reserved `relay.*` control surface.

/// A dotted method namespace (`"enrollment"` in `enrollment.mint`).
pub type Namespace = str;

/// The reserved system-control namespace. Everything under it executes
/// in the relay host and is NEVER forwarded — no matter what the
/// forwarding table says. The framework-specific battery (see the
/// `plana-tauri` crate) mounts its handlers here; the names below are
/// the pre-standardized cross-app surface.
pub const RELAY: &str = "relay";

/// Pre-standardized `relay.*` method names (see
/// `docs/en/rpc/relay-profile.md). The core only guarantees the namespace
/// reservation; which of these a given host mounts is the app's choice —
/// but mounting the standard names means every consumer app speaks the
/// same configuration language.
pub mod relay_methods {
    // transport & policy
    /// Install the outbound proxy (scheme + host:port + credentials).
    /// Empty host = explicitly direct.
    pub const NET_SET_PROXY: &str = "relay.net.set_proxy";
    /// Point the standard login handshake at an entrypoint URL (the
    /// public front door; inner-layer forwarding is the server's job).
    pub const NET_CONFIGURE_ENTRYPOINT: &str = "relay.net.configure_entrypoint";
    /// Open (or reuse) the WebSocket / HTTP long-poll connection to an
    /// endpoint; returns the connection handle forwarding maps onto.
    pub const CONN_OPEN: &str = "relay.conn.open";
    /// Map a namespace prefix onto an endpoint (whitelist edit).
    pub const FWD_MAP: &str = "relay.fwd.map";
    /// List the current forwarding whitelist and endpoints.
    pub const FWD_LIST: &str = "relay.fwd.list";
    // host window controls (any host framework may implement)
    pub const WINDOW_MINIMIZE: &str = "relay.window.minimize";
    pub const WINDOW_MAXIMIZE_TOGGLE: &str = "relay.window.maximize_toggle";
    pub const WINDOW_CLOSE: &str = "relay.window.close";
    /// Window state (handle/id, maximized flag) for UI chrome syncing.
    pub const WINDOW_STATE: &str = "relay.window.state";
}

/// Extract the leading dotted namespace from a JSON-RPC method name.
/// Methods without a dot have no namespace and route nowhere (deny).
pub fn namespace_of(method: &str) -> &str {
    match method.split_once('.') {
        Some((head, _)) => head,
        None => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_dotted_namespace() {
        assert_eq!(namespace_of("enrollment.mint"), "enrollment");
        assert_eq!(namespace_of("a.b.c"), "a");
        assert_eq!(namespace_of("relay.net.set_proxy"), RELAY);
        assert_eq!(namespace_of("bare"), "");
    }
}
