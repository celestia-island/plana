//! Multi-hop guard: chains compose, loops must not.

use serde::{Deserialize, Serialize};

/// Default maximum hops a relayed frame may traverse (A0..A4 by default
/// — deliberately generous; hosts may tighten, never loosen beyond
/// their own policy).
pub const DEFAULT_MAX_HOPS: u8 = 4;

/// Per-frame relay context carried in the JSON-RPC request's extension
/// member `relay` (JSON-RPC 2.0 permits unknown members; the service
/// profile's strictness applies to the server side, not to edge
/// envelopes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RelayContext {
    /// Hops already traversed. Incremented by each forwarding relay;
    /// a frame whose count would exceed the host's maximum is refused
    /// with an error instead of being forwarded (loop guard).
    #[serde(default)]
    pub hops: u8,
    /// Diagnostic trace of endpoint names traversed so far
    /// (`"edge", "gateway"`). Never load-bearing; hosts may strip it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub via: Vec<String>,
}

/// Loop guard evaluated before every forward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HopGuard {
    pub max_hops: u8,
}

impl Default for HopGuard {
    fn default() -> Self {
        Self {
            max_hops: DEFAULT_MAX_HOPS,
        }
    }
}

impl HopGuard {
    /// `Ok(())` when the frame may traverse one more hop; the error
    /// string is a JSON-RPC-error-ready message.
    pub fn admit(&self, context: &RelayContext) -> Result<(), String> {
        if context.hops >= self.max_hops {
            return Err(format!(
                "relay hop limit reached ({} >= {}); refusing to forward — possible loop",
                context.hops, self.max_hops
            ));
        }
        Ok(())
    }

    /// The context a forwarder must attach before handing the frame to
    /// the next hop.
    pub fn step(context: &RelayContext, hop_name: &str) -> RelayContext {
        let mut via = context.via.clone();
        via.push(hop_name.to_string());
        RelayContext {
            hops: context.hops + 1,
            via,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_beyond_limit_and_steps_trace() {
        let guard = HopGuard::default();
        let ctx = RelayContext {
            hops: 4,
            via: vec![],
        };
        assert!(guard.admit(&ctx).is_err());

        let ok = RelayContext {
            hops: 1,
            via: vec!["edge".into()],
        };
        assert!(guard.admit(&ok).is_ok());
        let stepped = HopGuard::step(&ok, "gateway");
        assert_eq!(stepped.hops, 2);
        assert_eq!(stepped.via, vec!["edge".to_string(), "gateway".to_string()]);
    }
}
