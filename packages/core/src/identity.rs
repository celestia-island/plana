use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::AgentBadge;

const DEMIURGE: &str = "demiurge";
const SESSION_NUMBER_MAX: u16 = 999;
const RESERVED_ZERO: u16 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContainerId(String);

#[derive(Serialize, Deserialize)]
struct ContainerIdRepr {
    container_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_id: Option<u16>,
}

impl Serialize for ContainerId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let repr = if self.is_demiurge() {
            ContainerIdRepr {
                container_type: "demiurge".to_string(),
                container_id: None,
            }
        } else {
            ContainerIdRepr {
                container_type: "normal".to_string(),
                container_id: Some(self.0.parse::<u16>().ok().unwrap_or(0)),
            }
        };
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ContainerId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct FlexRepr {
            container_type: Option<String>,
            container_id: Option<u16>,
            #[serde(default)]
            raw: Option<String>,
        }
        let flex: FlexRepr = FlexRepr::deserialize(deserializer)?;
        if let Some(raw) = flex.raw {
            return ContainerId::new(&raw).ok_or_else(|| {
                serde::de::Error::custom(format!("invalid ContainerId raw: {}", raw))
            });
        }
        match flex.container_type.as_deref() {
            Some("demiurge") => Ok(ContainerId::demiurge()),
            Some("normal") => {
                let num = flex.container_id.ok_or_else(|| {
                    serde::de::Error::custom("normal container requires container_id")
                })?;
                ContainerId::new(&format!("{:03}", num)).ok_or_else(|| {
                    serde::de::Error::custom(format!("invalid ContainerId num: {}", num))
                })
            }
            _ => Err(serde::de::Error::custom(
                "missing or invalid container_type (expected 'demiurge' or 'normal')",
            )),
        }
    }
}

impl ContainerId {
    pub fn demiurge() -> Self {
        Self(DEMIURGE.to_string())
    }

    pub fn is_demiurge(&self) -> bool {
        self.0 == DEMIURGE
    }

    pub fn new(raw: &str) -> Option<Self> {
        let stripped = raw.trim_start_matches('#');
        if stripped.is_empty() {
            return None;
        }
        if stripped == DEMIURGE {
            return Some(Self(stripped.to_string()));
        }
        if stripped.len() == 3 && stripped.chars().all(|c| c.is_ascii_digit()) {
            if stripped == "000" {
                return None;
            }
            return Some(Self(stripped.to_string()));
        }
        None
    }

    pub fn from_raw_unchecked(raw: String) -> Self {
        Self(raw.trim_start_matches('#').to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_badge_string(&self) -> String {
        format!("#{}", self.0)
    }

    pub fn is_valid(&self) -> bool {
        Self::is_valid_str(&self.0)
    }

    fn is_valid_str(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        if s == DEMIURGE {
            return true;
        }
        s.len() == 3 && s.chars().all(|c| c.is_ascii_digit()) && s != "000"
    }
}

impl fmt::Display for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl AsRef<str> for ContainerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LlmSessionId {
    pub container_id: ContainerId,
    pub session_number: u16,
}

#[derive(Serialize, Deserialize)]
struct LlmSessionIdRepr {
    container_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_id: Option<u16>,
    session_number: u16,
}

impl Serialize for LlmSessionId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let repr = if self.container_id.is_demiurge() {
            LlmSessionIdRepr {
                container_type: "demiurge".to_string(),
                container_id: None,
                session_number: self.session_number,
            }
        } else {
            LlmSessionIdRepr {
                container_type: "normal".to_string(),
                container_id: Some(self.container_id.as_str().parse::<u16>().unwrap_or(0)),
                session_number: self.session_number,
            }
        };
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LlmSessionId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct FlexRepr {
            container_type: Option<String>,
            container_id: Option<u16>,
            session_number: Option<u16>,
            #[serde(default)]
            raw: Option<String>,
        }
        let flex: FlexRepr = FlexRepr::deserialize(deserializer)?;
        if let Some(raw) = flex.raw {
            return LlmSessionId::parse(&raw).ok_or_else(|| {
                serde::de::Error::custom(format!("invalid LlmSessionId raw: {}", raw))
            });
        }
        let container_id = match flex.container_type.as_deref() {
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
                    "missing container_type (expected 'demiurge' or 'normal')",
                ));
            }
        };
        let session_number = flex
            .session_number
            .ok_or_else(|| serde::de::Error::custom("missing session_number"))?;
        LlmSessionId::new(container_id, session_number).ok_or_else(|| {
            serde::de::Error::custom(format!("invalid session_number: {}", session_number))
        })
    }
}

impl LlmSessionId {
    pub fn new(container_id: ContainerId, session_number: u16) -> Option<Self> {
        if session_number == RESERVED_ZERO || session_number > SESSION_NUMBER_MAX {
            return None;
        }
        Some(Self {
            container_id,
            session_number,
        })
    }

    pub fn parse(s: &str) -> Option<Self> {
        let stripped = s.trim_start_matches('#');
        let dot_pos = stripped.find('.')?;
        let parent = &stripped[..dot_pos];
        let child = &stripped[dot_pos + 1..];
        let container_id = ContainerId::new(parent)?;
        let session_number = child.parse::<u16>().ok()?;
        Self::new(container_id, session_number)
    }

    pub fn to_badge_string(&self) -> String {
        format!("#{}.{:03}", self.container_id.as_str(), self.session_number)
    }

    pub fn container_badge_string(&self) -> String {
        self.container_id.to_badge_string()
    }

    pub fn session_number(&self) -> u16 {
        self.session_number
    }
}

impl fmt::Display for LlmSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{}.{:03}",
            self.container_id.as_str(),
            self.session_number
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub container_id: ContainerId,
    pub llm_session_id: LlmSessionId,
}

impl AgentIdentity {
    pub fn new(container_id: ContainerId, llm_session_id: LlmSessionId) -> Self {
        Self {
            container_id,
            llm_session_id,
        }
    }

    pub fn from_session(session_id: LlmSessionId) -> Self {
        let container_id = session_id.container_id.clone();
        Self {
            container_id,
            llm_session_id: session_id,
        }
    }

    pub fn to_container_badge(&self) -> String {
        self.container_id.to_badge_string()
    }

    pub fn to_session_badge(&self) -> String {
        self.llm_session_id.to_badge_string()
    }
}

impl fmt::Display for AgentIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.container_id, self.llm_session_id)
    }
}

impl From<LlmSessionId> for AgentIdentity {
    fn from(session_id: LlmSessionId) -> Self {
        Self::from_session(session_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(String);

impl AgentId {
    pub fn from_raw(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn generate(kind: &str) -> Self {
        Self(format!("{}-{}", kind, Uuid::now_v7()))
    }

    pub fn from_container_short(kind: &str, container_uuid: &str) -> Self {
        let short = if container_uuid.len() >= 8 {
            &container_uuid[..8]
        } else {
            container_uuid
        };
        Self(format!("{}-{}", kind, short))
    }

    pub fn system() -> Self {
        Self("system".to_string())
    }

    pub fn is_system(&self) -> bool {
        self.0 == "system"
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn kind(&self) -> &str {
        match self.0.find('-') {
            Some(pos) => &self.0[..pos],
            None => &self.0,
        }
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for AgentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<AgentId> for String {
    fn from(id: AgentId) -> String {
        id.0
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::borrow::Borrow<str> for AgentId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&AgentBadge> for ContainerId {
    type Error = ();

    fn try_from(badge: &AgentBadge) -> Result<Self, Self::Error> {
        ContainerId::new(badge.as_str()).ok_or(())
    }
}

impl TryFrom<&AgentBadge> for LlmSessionId {
    type Error = ();

    fn try_from(badge: &AgentBadge) -> Result<Self, Self::Error> {
        LlmSessionId::parse(badge.as_str()).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn container_id_demiurge() -> Result<()> {
        let id = ContainerId::demiurge();
        assert_eq!(id.as_str(), "demiurge");
        assert_eq!(id.to_badge_string(), "#demiurge");
        assert!(id.is_demiurge());
        Ok(())
    }

    #[test]
    fn container_id_numeric() -> Result<()> {
        let id = ContainerId::new("123").context("expected valid container id")?;
        assert_eq!(id.as_str(), "123");
        assert_eq!(id.to_badge_string(), "#123");
        assert!(!id.is_demiurge());
        Ok(())
    }

    #[test]
    fn container_id_rejects_zero() -> Result<()> {
        assert!(ContainerId::new("000").is_none());
        Ok(())
    }

    #[test]
    fn container_id_with_hash_prefix() -> Result<()> {
        let id = ContainerId::new("#456").context("expected valid container id")?;
        assert_eq!(id.as_str(), "456");
        Ok(())
    }

    #[test]
    fn llm_session_id_valid() -> Result<()> {
        let container = ContainerId::new("123").context("expected valid container id")?;
        let session = LlmSessionId::new(container, 1).context("expected valid session id")?;
        assert_eq!(session.to_badge_string(), "#123.001");
        assert_eq!(session.container_badge_string(), "#123");
        Ok(())
    }

    #[test]
    fn llm_session_id_rejects_zero() -> Result<()> {
        let container = ContainerId::new("123").context("expected valid container id")?;
        assert!(LlmSessionId::new(container, 0).is_none());
        Ok(())
    }

    #[test]
    fn llm_session_id_rejects_over_999() -> Result<()> {
        let container = ContainerId::new("123").context("expected valid container id")?;
        assert!(LlmSessionId::new(container, 1000).is_none());
        Ok(())
    }

    #[test]
    fn llm_session_id_max_999() -> Result<()> {
        let container = ContainerId::new("123").context("expected valid container id")?;
        let session = LlmSessionId::new(container, 999).context("expected valid session id")?;
        assert_eq!(session.to_badge_string(), "#123.999");
        Ok(())
    }

    #[test]
    fn llm_session_id_parse() -> Result<()> {
        let session = LlmSessionId::parse("#demiurge.002").context("expected valid session id")?;
        assert_eq!(session.container_id.as_str(), "demiurge");
        assert_eq!(session.session_number, 2);
        Ok(())
    }

    #[test]
    fn agent_identity_from_session() -> Result<()> {
        let container = ContainerId::new("123").context("expected valid container id")?;
        let session = LlmSessionId::new(container, 1).context("expected valid session id")?;
        let identity = AgentIdentity::from_session(session);
        assert_eq!(identity.to_container_badge(), "#123");
        assert_eq!(identity.to_session_badge(), "#123.001");
        Ok(())
    }

    #[test]
    fn container_id_serde_normal() -> Result<()> {
        let id = ContainerId::new("123").context("expected valid container id")?;
        let json = serde_json::to_string(&id)?;
        assert_eq!(json, r#"{"container_type":"normal","container_id":123}"#);
        let back: ContainerId = serde_json::from_str(&json)?;
        assert_eq!(back, id);
        Ok(())
    }

    #[test]
    fn container_id_serde_demiurge() -> Result<()> {
        let id = ContainerId::demiurge();
        let json = serde_json::to_string(&id)?;
        assert_eq!(json, r#"{"container_type":"demiurge"}"#);
        let back: ContainerId = serde_json::from_str(&json)?;
        assert_eq!(back, id);
        Ok(())
    }

    #[test]
    fn llm_session_id_serde_normal() -> Result<()> {
        let container = ContainerId::new("123").context("expected valid container id")?;
        let session = LlmSessionId::new(container, 5).context("expected valid session id")?;
        let json = serde_json::to_string(&session)?;
        assert_eq!(
            json,
            r#"{"container_type":"normal","container_id":123,"session_number":5}"#
        );
        let back: LlmSessionId = serde_json::from_str(&json)?;
        assert_eq!(back, session);
        Ok(())
    }

    #[test]
    fn llm_session_id_serde_demiurge() -> Result<()> {
        let container = ContainerId::demiurge();
        let session = LlmSessionId::new(container, 2).context("expected valid session id")?;
        let json = serde_json::to_string(&session)?;
        assert_eq!(json, r#"{"container_type":"demiurge","session_number":2}"#);
        let back: LlmSessionId = serde_json::from_str(&json)?;
        assert_eq!(back, session);
        Ok(())
    }
}
