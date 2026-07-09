use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::identity::{ContainerId, LlmSessionId};

const SHORT_ID_LEN: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkspaceRef {
    ShortId(String),
    Alias(String),
}

impl WorkspaceRef {
    pub fn from_uuid_short(uuid: &Uuid) -> Self {
        let hex = uuid.as_hyphenated().to_string();
        Self::ShortId(hex[hex.len() - SHORT_ID_LEN..].to_string())
    }

    pub fn alias(name: impl Into<String>) -> Self {
        Self::Alias(name.into())
    }

    pub fn parse_from_prefix(s: &str) -> Option<(Self, &str)> {
        let rest = s.strip_prefix('@')?;
        if rest.is_empty() {
            return None;
        }
        let (ws_part, badge_part) = if let Some(pos) = rest.find('#') {
            (&rest[..pos], &rest[pos..])
        } else {
            return None;
        };
        if ws_part.is_empty() {
            return None;
        }
        let ws_ref =
            if ws_part.len() == SHORT_ID_LEN && ws_part.chars().all(|c| c.is_ascii_hexdigit()) {
                WorkspaceRef::ShortId(ws_part.to_string())
            } else {
                WorkspaceRef::Alias(ws_part.to_string())
            };
        Some((ws_ref, badge_part))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::ShortId(s) => s,
            Self::Alias(s) => s,
        }
    }

    pub fn is_short_id(&self) -> bool {
        matches!(self, Self::ShortId(_))
    }

    pub fn is_alias(&self) -> bool {
        matches!(self, Self::Alias(_))
    }
}

impl fmt::Display for WorkspaceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceScopedBadge {
    pub workspace_ref: Option<WorkspaceRef>,
    pub badge: ContainerId,
}

impl WorkspaceScopedBadge {
    pub fn new(workspace_ref: Option<WorkspaceRef>, badge: ContainerId) -> Self {
        Self {
            workspace_ref,
            badge,
        }
    }

    pub fn local(badge: ContainerId) -> Self {
        Self {
            workspace_ref: None,
            badge,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        if let Some((ws_ref, badge_str)) = WorkspaceRef::parse_from_prefix(s) {
            let badge = ContainerId::new(badge_str)?;
            Some(Self {
                workspace_ref: Some(ws_ref),
                badge,
            })
        } else {
            let badge = ContainerId::new(s)?;
            if badge.is_demiurge() {
                return None;
            }
            Some(Self {
                workspace_ref: None,
                badge,
            })
        }
    }

    pub fn is_workspace_local(&self) -> bool {
        self.workspace_ref.is_none()
    }

    pub fn is_demiurge(&self) -> bool {
        self.badge.is_demiurge()
    }

    pub fn to_badge_string(&self) -> String {
        match &self.workspace_ref {
            Some(ws) => format!("{}{}", ws, self.badge),
            None => self.badge.to_badge_string(),
        }
    }
}

impl fmt::Display for WorkspaceScopedBadge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.workspace_ref {
            Some(ws) => write!(f, "{}{}", ws, self.badge),
            None => write!(f, "{}", self.badge),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceScopedSessionId {
    pub workspace_ref: Option<WorkspaceRef>,
    pub session: LlmSessionId,
}

impl WorkspaceScopedSessionId {
    pub fn new(workspace_ref: Option<WorkspaceRef>, session: LlmSessionId) -> Self {
        Self {
            workspace_ref,
            session,
        }
    }

    pub fn local(session: LlmSessionId) -> Self {
        Self {
            workspace_ref: None,
            session,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        if let Some((ws_ref, badge_str)) = WorkspaceRef::parse_from_prefix(s) {
            let session = LlmSessionId::parse(badge_str)?;
            Some(Self {
                workspace_ref: Some(ws_ref),
                session,
            })
        } else {
            let session = LlmSessionId::parse(s)?;
            if session.container_id.is_demiurge() {
                return None;
            }
            Some(Self {
                workspace_ref: None,
                session,
            })
        }
    }

    pub fn is_workspace_local(&self) -> bool {
        self.workspace_ref.is_none()
    }

    pub fn is_demiurge(&self) -> bool {
        self.session.container_id.is_demiurge()
    }

    pub fn to_container_badge_string(&self) -> String {
        match &self.workspace_ref {
            Some(ws) => format!("{}{}", ws, self.session.container_badge_string()),
            None => self.session.container_badge_string(),
        }
    }

    pub fn to_badge_string(&self) -> String {
        match &self.workspace_ref {
            Some(ws) => format!("{}{}", ws, self.session.to_badge_string()),
            None => self.session.to_badge_string(),
        }
    }

    pub fn container_badge(&self) -> WorkspaceScopedBadge {
        WorkspaceScopedBadge {
            workspace_ref: self.workspace_ref.clone(),
            badge: self.session.container_id.clone(),
        }
    }
}

impl fmt::Display for WorkspaceScopedSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.workspace_ref {
            Some(ws) => write!(f, "{}{}", ws, self.session),
            None => write!(f, "{}", self.session),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WorkspaceScopedBadgeRepr {
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_ref: Option<WorkspaceRefRepr>,
    container_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_id: Option<u16>,
}

#[derive(Serialize, Deserialize)]
struct WorkspaceRefRepr {
    #[serde(rename = "type")]
    ref_type: String,
    value: String,
}

impl Serialize for WorkspaceScopedBadge {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ws_repr = self.workspace_ref.as_ref().map(|ws| match ws {
            WorkspaceRef::ShortId(s) => WorkspaceRefRepr {
                ref_type: "short_id".to_string(),
                value: s.clone(),
            },
            WorkspaceRef::Alias(s) => WorkspaceRefRepr {
                ref_type: "alias".to_string(),
                value: s.clone(),
            },
        });

        let (container_type, container_id) = if self.badge.is_demiurge() {
            ("demiurge".to_string(), None)
        } else {
            (
                "normal".to_string(),
                Some(self.badge.as_str().parse::<u16>().unwrap_or(0)),
            )
        };

        let repr = WorkspaceScopedBadgeRepr {
            workspace_ref: ws_repr,
            container_type,
            container_id,
        };
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WorkspaceScopedBadge {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct FlexRepr {
            workspace_ref: Option<WorkspaceRefRepr>,
            container_type: Option<String>,
            container_id: Option<u16>,
            #[serde(default)]
            raw: Option<String>,
        }
        let flex: FlexRepr = FlexRepr::deserialize(deserializer)?;
        if let Some(raw) = flex.raw {
            return WorkspaceScopedBadge::parse(&raw).ok_or_else(|| {
                serde::de::Error::custom(format!("invalid WorkspaceScopedBadge raw: {}", raw))
            });
        }
        let badge = match flex.container_type.as_deref() {
            Some("demiurge") => ContainerId::demiurge(),
            Some("normal") => {
                let num = flex.container_id.ok_or_else(|| {
                    serde::de::Error::custom("normal container requires container_id")
                })?;
                ContainerId::new(&format!("{:03}", num)).ok_or_else(|| {
                    serde::de::Error::custom(format!("invalid container_id num: {}", num))
                })?
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "missing or invalid container_type",
                ));
            }
        };
        let workspace_ref = flex.workspace_ref.map(|r| match r.ref_type.as_str() {
            "short_id" => WorkspaceRef::ShortId(r.value),
            _ => WorkspaceRef::Alias(r.value),
        });
        Ok(WorkspaceScopedBadge {
            workspace_ref,
            badge,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ContainerId;
    use anyhow::{Context, Result};

    #[test]
    fn workspace_ref_from_uuid_short() -> Result<()> {
        let uid =
            Uuid::parse_str("3a7bc1d2-e4f5-6789-0abc-def012345678").context("test precondition")?;
        let ws = WorkspaceRef::from_uuid_short(&uid);
        assert!(ws.is_short_id());
        assert_eq!(ws.as_str(), "345678");
        Ok(())
    }

    #[test]
    fn workspace_ref_alias() -> Result<()> {
        let ws = WorkspaceRef::alias("my-project");
        assert!(ws.is_alias());
        assert_eq!(ws.as_str(), "my-project");
        Ok(())
    }

    #[test]
    fn workspace_ref_parse_short_id() -> Result<()> {
        let (ws, rest) =
            WorkspaceRef::parse_from_prefix("@a1b2c3#demiurge").context("test precondition")?;
        assert!(ws.is_short_id());
        assert_eq!(ws.as_str(), "a1b2c3");
        assert_eq!(rest, "#demiurge");
        Ok(())
    }

    #[test]
    fn workspace_ref_parse_alias() -> Result<()> {
        let (ws, rest) =
            WorkspaceRef::parse_from_prefix("@myproject#001").context("test precondition")?;
        assert!(ws.is_alias());
        assert_eq!(ws.as_str(), "myproject");
        assert_eq!(rest, "#001");
        Ok(())
    }

    #[test]
    fn workspace_ref_parse_no_at() -> Result<()> {
        assert!(WorkspaceRef::parse_from_prefix("#demiurge").is_none());
        Ok(())
    }

    #[test]
    fn workspace_ref_parse_empty_after_at() -> Result<()> {
        assert!(WorkspaceRef::parse_from_prefix("@#demiurge").is_none());
        Ok(())
    }

    #[test]
    fn workspace_ref_parse_no_hash() -> Result<()> {
        assert!(WorkspaceRef::parse_from_prefix("@myproject").is_none());
        Ok(())
    }

    #[test]
    fn workspace_ref_display() -> Result<()> {
        assert_eq!(format!("{}", WorkspaceRef::alias("proj")), "@proj");
        assert_eq!(
            format!(
                "{}",
                WorkspaceRef::from_uuid_short(
                    &Uuid::parse_str("3a7bc1d2-e4f5-6789-0abc-def012345678")
                        .context("test precondition")?
                )
            ),
            "@345678"
        );
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_bare_demiurge_rejected() -> Result<()> {
        assert!(
            WorkspaceScopedBadge::parse("#demiurge").is_none(),
            "bare #demiurge without @workspace must be rejected"
        );
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_local_numeric() -> Result<()> {
        let b = WorkspaceScopedBadge::parse("#123").context("test precondition")?;
        assert!(b.is_workspace_local());
        assert!(!b.is_demiurge());
        assert_eq!(b.to_badge_string(), "#123");
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_workspace_demiurge() -> Result<()> {
        let b = WorkspaceScopedBadge::parse("@a1b2c3#demiurge").context("test precondition")?;
        assert!(!b.is_workspace_local());
        assert!(b.is_demiurge());
        assert_eq!(b.to_badge_string(), "@a1b2c3#demiurge");
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_alias_numeric() -> Result<()> {
        let b = WorkspaceScopedBadge::parse("@myproj#001").context("test precondition")?;
        assert!(!b.is_workspace_local());
        assert!(!b.is_demiurge());
        assert_eq!(b.to_badge_string(), "@myproj#001");
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_invalid() -> Result<()> {
        assert!(WorkspaceScopedBadge::parse("").is_none());
        assert!(WorkspaceScopedBadge::parse("#").is_none());
        assert!(WorkspaceScopedBadge::parse("#000").is_none());
        assert!(WorkspaceScopedBadge::parse("#demiurge").is_none());
        Ok(())
    }

    #[test]
    fn scoped_session_parse_bare_demiurge_rejected() -> Result<()> {
        assert!(
            WorkspaceScopedSessionId::parse("#demiurge.002").is_none(),
            "bare #demiurge.002 without @workspace must be rejected"
        );
        Ok(())
    }

    #[test]
    fn scoped_session_parse_workspace() -> Result<()> {
        let s =
            WorkspaceScopedSessionId::parse("@a1b2c3#demiurge.002").context("test precondition")?;
        assert!(!s.is_workspace_local());
        assert!(s.is_demiurge());
        assert_eq!(s.to_badge_string(), "@a1b2c3#demiurge.002");
        assert_eq!(s.to_container_badge_string(), "@a1b2c3#demiurge");
        Ok(())
    }

    #[test]
    fn scoped_session_parse_numeric() -> Result<()> {
        let s = WorkspaceScopedSessionId::parse("#123.005").context("test precondition")?;
        assert!(s.is_workspace_local());
        assert!(!s.is_demiurge());
        assert_eq!(s.to_badge_string(), "#123.005");
        Ok(())
    }

    #[test]
    fn scoped_session_container_badge() -> Result<()> {
        let s = WorkspaceScopedSessionId::parse("@proj#001.003").context("test precondition")?;
        let b = s.container_badge();
        assert_eq!(b.to_badge_string(), "@proj#001");
        assert_eq!(b.workspace_ref, s.workspace_ref);
        Ok(())
    }

    #[test]
    fn scoped_session_parse_invalid() -> Result<()> {
        assert!(WorkspaceScopedSessionId::parse("").is_none());
        assert!(WorkspaceScopedSessionId::parse("#demiurge.000").is_none());
        assert!(WorkspaceScopedSessionId::parse("#demiurge.1000").is_none());
        assert!(WorkspaceScopedSessionId::parse("#demiurge.001").is_none());
        Ok(())
    }

    #[test]
    fn scoped_badge_display_roundtrip() -> Result<()> {
        let cases = vec!["#123", "@a1b2c3#demiurge", "@myproj#001"];
        for case in cases {
            let parsed = WorkspaceScopedBadge::parse(case).context("test precondition")?;
            assert_eq!(parsed.to_string(), case, "roundtrip failed for: {}", case);
        }
        Ok(())
    }

    #[test]
    fn scoped_session_display_roundtrip() -> Result<()> {
        let cases = vec!["#123.005", "@a1b2c3#demiurge.002", "@myproj#001.003"];
        for case in cases {
            let parsed = WorkspaceScopedSessionId::parse(case).context("test precondition")?;
            assert_eq!(parsed.to_string(), case, "roundtrip failed for: {}", case);
        }
        Ok(())
    }

    #[test]
    fn scoped_badge_equality() -> Result<()> {
        let a = WorkspaceScopedBadge::parse("@proj#001").context("test precondition")?;
        let b = WorkspaceScopedBadge::new(
            Some(WorkspaceRef::alias("proj")),
            ContainerId::new("001").context("test precondition")?,
        );
        assert_eq!(a, b);
        Ok(())
    }

    #[test]
    fn scoped_session_equality() -> Result<()> {
        let a = WorkspaceScopedSessionId::parse("@proj#001.003").context("test precondition")?;
        let b = WorkspaceScopedSessionId::new(
            Some(WorkspaceRef::alias("proj")),
            LlmSessionId::new(ContainerId::new("001").context("test precondition")?, 3)
                .context("test precondition")?,
        );
        assert_eq!(a, b);
        Ok(())
    }

    #[test]
    fn workspace_ref_parse_alias_with_hyphens_and_underscores() -> Result<()> {
        let (ws, rest) = WorkspaceRef::parse_from_prefix("@my-cool_proj#demiurge")
            .context("test precondition")?;
        assert!(ws.is_alias());
        assert_eq!(ws.as_str(), "my-cool_proj");
        assert_eq!(rest, "#demiurge");
        Ok(())
    }

    #[test]
    fn workspace_ref_parse_6_char_alias_not_hex() -> Result<()> {
        let (ws, _) =
            WorkspaceRef::parse_from_prefix("@zzzzzz#demiurge").context("test precondition")?;
        assert!(
            ws.is_alias(),
            "non-hex 6-char string should be alias, not short_id"
        );
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_with_dots_in_alias() -> Result<()> {
        let b = WorkspaceScopedBadge::parse("@my.proj#demiurge").context("test precondition")?;
        assert_eq!(
            b.workspace_ref.context("test precondition")?.as_str(),
            "my.proj"
        );
        Ok(())
    }

    #[test]
    fn scoped_session_parse_with_dots_in_alias() -> Result<()> {
        let s = WorkspaceScopedSessionId::parse("@my.proj#001.003").context("test precondition")?;
        assert_eq!(
            s.workspace_ref.context("test precondition")?.as_str(),
            "my.proj"
        );
        Ok(())
    }

    #[test]
    fn scoped_badge_serde_roundtrip_local() -> Result<()> {
        let badge = WorkspaceScopedBadge::parse("#123").context("test precondition")?;
        let json = serde_json::to_string(&badge).context("test precondition")?;
        let back: WorkspaceScopedBadge =
            serde_json::from_str(&json).context("test precondition")?;
        assert_eq!(back, badge);
        Ok(())
    }

    #[test]
    fn scoped_badge_serde_roundtrip_workspace() -> Result<()> {
        let badge = WorkspaceScopedBadge::parse("@a1b2c3#demiurge").context("test precondition")?;
        let json = serde_json::to_string(&badge).context("test precondition")?;
        let back: WorkspaceScopedBadge =
            serde_json::from_str(&json).context("test precondition")?;
        assert_eq!(back, badge);
        Ok(())
    }

    #[test]
    fn workspace_ref_from_uuid_short_deterministic() -> Result<()> {
        let uid =
            Uuid::parse_str("3a7bc1d2-e4f5-6789-0abc-def012345678").context("test precondition")?;
        let a = WorkspaceRef::from_uuid_short(&uid);
        let b = WorkspaceRef::from_uuid_short(&uid);
        assert_eq!(a, b);
        Ok(())
    }

    #[test]
    fn workspace_ref_different_uuids_different_short_ids() -> Result<()> {
        let uid_a =
            Uuid::parse_str("3a7bc1d2-e4f5-6789-0abc-def012345678").context("test precondition")?;
        let uid_b =
            Uuid::parse_str("3a7bc1d2-e4f5-6789-0abc-def012345679").context("test precondition")?;
        let a = WorkspaceRef::from_uuid_short(&uid_a);
        let b = WorkspaceRef::from_uuid_short(&uid_b);
        assert_ne!(a, b);
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_mixed_case_hex() -> Result<()> {
        let b = WorkspaceScopedBadge::parse("@A1B2C3#demiurge").context("test precondition")?;
        assert_eq!(
            b.workspace_ref.context("test precondition")?.as_str(),
            "A1B2C3"
        );
        Ok(())
    }

    #[test]
    fn scoped_session_parse_numeric_workspace() -> Result<()> {
        let s = WorkspaceScopedSessionId::parse("@123abc#001.003").context("test precondition")?;
        assert_eq!(
            s.workspace_ref.context("test precondition")?.as_str(),
            "123abc"
        );
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_long_alias() -> Result<()> {
        let b = WorkspaceScopedBadge::parse("@this-is-a-very-long-workspace-name#demiurge")
            .context("test precondition")?;
        assert_eq!(
            b.workspace_ref.context("test precondition")?.as_str(),
            "this-is-a-very-long-workspace-name"
        );
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_at_only() -> Result<()> {
        assert!(WorkspaceScopedBadge::parse("@#demiurge").is_none());
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_at_without_hash() -> Result<()> {
        assert!(WorkspaceScopedBadge::parse("@myproj").is_none());
        Ok(())
    }

    #[test]
    fn scoped_badge_parse_double_at() -> Result<()> {
        let b = WorkspaceScopedBadge::parse("@@proj#001").context("test precondition")?;
        assert_eq!(
            b.workspace_ref.context("test precondition")?.as_str(),
            "@proj"
        );
        Ok(())
    }

    #[test]
    fn scoped_session_container_badge_preserves_workspace() -> Result<()> {
        let s =
            WorkspaceScopedSessionId::parse("@alias#demiurge.001").context("test precondition")?;
        let cb = s.container_badge();
        assert_eq!(cb.workspace_ref, s.workspace_ref);
        assert!(cb.is_demiurge());
        assert_eq!(cb.to_badge_string(), "@alias#demiurge");
        Ok(())
    }

    #[test]
    fn scoped_badge_new_local_constructor_numeric() -> Result<()> {
        let b = WorkspaceScopedBadge::local(ContainerId::new("123").context("test precondition")?);
        assert!(b.is_workspace_local());
        assert!(!b.is_demiurge());
        assert_eq!(b.to_badge_string(), "#123");
        Ok(())
    }

    #[test]
    fn scoped_session_new_local_constructor() -> Result<()> {
        let s = WorkspaceScopedSessionId::local(
            LlmSessionId::new(ContainerId::new("123").context("test precondition")?, 5)
                .context("test precondition")?,
        );
        assert!(s.is_workspace_local());
        assert_eq!(s.to_badge_string(), "#123.005");
        Ok(())
    }

    #[test]
    fn workspace_ref_display_roundtrip() -> Result<()> {
        let ws = WorkspaceRef::alias("my-proj");
        assert_eq!(format!("{}", ws), "@my-proj");

        let uid =
            Uuid::parse_str("3a7bc1d2-e4f5-6789-0abc-def012345678").context("test precondition")?;
        let ws2 = WorkspaceRef::from_uuid_short(&uid);
        assert_eq!(format!("{}", ws2), "@345678");
        Ok(())
    }
}
