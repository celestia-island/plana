use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum dot-separated segments a badge may carry: the root segment plus
/// up to four sub-levels (`demiurge.001.002.003.004`). The cap exists so
/// nested "ghost" sub-agents cannot recurse without bound.
pub const BADGE_MAX_SEGMENTS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentBadge(String);

impl AgentBadge {
    pub fn new(number: &str) -> Option<Self> {
        if Self::is_valid_str(number) {
            Some(Self(number.to_string()))
        } else {
            None
        }
    }

    pub fn from_raw_unchecked(raw: String) -> Self {
        Self(raw)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn is_valid(&self) -> bool {
        Self::is_valid_str(&self.0)
    }

    /// Grammar: an optional `@workspace` scope, then 1..=[`BADGE_MAX_SEGMENTS`]
    /// dot-separated segments. The root segment is `demiurge` or a three-digit
    /// number; every subsequent segment is a three-digit number. Before the
    /// ghost-subagent extension this accepted exactly one sub-segment (two
    /// total); deeper badges were rejected — the relaxation is strictly
    /// additive for previously-valid input.
    fn is_valid_str(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        let bare = if s.starts_with('@') {
            if let Some(hash_pos) = s.find('#') {
                &s[hash_pos + 1..]
            } else {
                return false;
            }
        } else {
            s
        };
        let mut segments = bare.split('.');
        let root = segments.next().unwrap_or("");
        if !(root == "demiurge" || (root.len() == 3 && root.chars().all(|c| c.is_ascii_digit()))) {
            return false;
        }
        let mut count = 1usize;
        for seg in segments {
            if seg.len() != 3 || !seg.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            count += 1;
            if count > BADGE_MAX_SEGMENTS {
                return false;
            }
        }
        true
    }

    /// Number of dot-separated segments (workspace scope excluded). A bare
    /// root badge has 1; `demiurge.001.002` has 3.
    pub fn segment_count(&self) -> usize {
        self.bare().split('.').count()
    }

    /// The badge string without any `@workspace#` scope prefix.
    fn bare(&self) -> &str {
        if self.0.starts_with('@') {
            if let Some(hash_pos) = self.0.find('#') {
                &self.0[hash_pos + 1..]
            } else {
                &self.0
            }
        } else {
            &self.0
        }
    }

    pub fn is_sub_badge(&self) -> bool {
        self.bare().contains('.')
    }

    /// The ROOT segment (before the first dot): the container identity the
    /// whole badge tree hangs off (`demiurge.001.002` → `demiurge`).
    /// Historical name kept: pre-depth callers treated this as "the parent"
    /// because root and parent coincided at depth two.
    pub fn parent_key(&self) -> Option<&str> {
        self.bare().find('.').map(|pos| &self.bare()[..pos])
    }

    /// Alias of [`Self::parent_key`] stating its actual semantics.
    pub fn root_key(&self) -> &str {
        let bare = self.bare();
        match bare.find('.') {
            Some(pos) => &bare[..pos],
            None => bare,
        }
    }

    /// The DIRECT parent badge: the string with the final segment removed
    /// (`demiurge.001.002` → `demiurge.001`). `None` for a root badge —
    /// a root has no parent. This is the key badge allocators nest under.
    pub fn immediate_parent_key(&self) -> Option<&str> {
        let bare = self.bare();
        bare.rfind('.').map(|pos| &bare[..pos])
    }

    /// The FINAL segment (`demiurge.001.002` → `002`). A root badge is its
    /// own leaf.
    pub fn leaf(&self) -> &str {
        let bare = self.bare();
        match bare.rfind('.') {
            Some(pos) => &bare[pos + 1..],
            None => bare,
        }
    }
}

impl fmt::Display for AgentBadge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl AsRef<str> for AgentBadge {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_badges_stay_valid() {
        assert!(AgentBadge::new("demiurge").is_some());
        assert!(AgentBadge::new("001").is_some());
        assert!(AgentBadge::new("").is_none());
        assert!(AgentBadge::new("demi").is_none());
        assert!(AgentBadge::new("01").is_none());
        assert!(AgentBadge::new("0001").is_none());
    }

    #[test]
    fn two_segment_badges_stay_valid() {
        assert!(AgentBadge::new("demiurge.001").is_some());
        assert!(AgentBadge::new("123.456").is_some());
        assert!(AgentBadge::new("demiurge.1").is_none());
        assert!(AgentBadge::new("demiurge.0001").is_none());
        assert!(AgentBadge::new(".001").is_none());
        assert!(AgentBadge::new("demiurge.").is_none());
    }

    #[test]
    fn deep_badges_pass_up_to_five_segments() {
        assert!(AgentBadge::new("demiurge.001.002").is_some());
        assert!(AgentBadge::new("demiurge.001.002.003").is_some());
        assert!(AgentBadge::new("demiurge.001.002.003.004").is_some());
        // Cap: a sixth segment is rejected.
        assert!(AgentBadge::new("demiurge.001.002.003.004.005").is_none());
    }

    #[test]
    fn deep_badge_segments_must_be_three_digits() {
        assert!(AgentBadge::new("demiurge.001.2").is_none());
        assert!(AgentBadge::new("demiurge.001.0002").is_none());
        assert!(AgentBadge::new("demiurge.001.abc").is_none());
    }

    #[test]
    fn workspace_scope_accepts_deep_badges() {
        assert!(AgentBadge::new("@proj#demiurge.001.002").is_some());
        assert!(AgentBadge::new("@a1b2c3#123.456.789").is_some());
        assert!(AgentBadge::new("@noscope").is_none());
    }

    #[test]
    fn segment_count_walks_the_depth() {
        assert_eq!(AgentBadge::new("demiurge").unwrap().segment_count(), 1);
        assert_eq!(AgentBadge::new("demiurge.001").unwrap().segment_count(), 2);
        assert_eq!(
            AgentBadge::new("demiurge.001.002").unwrap().segment_count(),
            3
        );
        assert_eq!(
            AgentBadge::new("@ws#demiurge.001.002.003")
                .unwrap()
                .segment_count(),
            4
        );
    }

    #[test]
    fn root_and_immediate_parent_split_at_depth() {
        let deep = AgentBadge::new("demiurge.001.002").unwrap();
        assert_eq!(deep.root_key(), "demiurge");
        assert_eq!(deep.parent_key(), Some("demiurge"));
        assert_eq!(deep.immediate_parent_key(), Some("demiurge.001"));
        assert_eq!(deep.leaf(), "002");

        let two = AgentBadge::new("demiurge.001").unwrap();
        assert_eq!(two.root_key(), "demiurge");
        assert_eq!(two.parent_key(), Some("demiurge"));
        // Depth two: root and immediate parent coincide (historical shape).
        assert_eq!(two.immediate_parent_key(), Some("demiurge"));
        assert_eq!(two.leaf(), "001");

        let root = AgentBadge::new("demiurge").unwrap();
        assert_eq!(root.root_key(), "demiurge");
        assert_eq!(root.parent_key(), None);
        assert_eq!(root.immediate_parent_key(), None);
        assert_eq!(root.leaf(), "demiurge");

        let scoped = AgentBadge::new("@ws#demiurge.001.002").unwrap();
        // Scope prefixes never leak into the split keys.
        assert_eq!(scoped.root_key(), "demiurge");
        assert_eq!(scoped.immediate_parent_key(), Some("demiurge.001"));
    }

    #[test]
    fn display_and_as_str_round_trip() {
        let badge = AgentBadge::new("demiurge.001.002").unwrap();
        assert_eq!(badge.to_string(), "#demiurge.001.002");
        assert_eq!(badge.as_str(), "demiurge.001.002");
    }
}
