use serde::{Deserialize, Serialize};
use ts_rs::TS;

const SHORT_ID_LEN: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "value")]
#[ts(export, export_to = "HttpTypes.ts")]
pub enum WorkspaceRef {
    #[serde(rename = "short_id")]
    ShortId(String),
    #[serde(rename = "alias")]
    Alias(String),
}

impl WorkspaceRef {
    pub fn from_uuid_short(uuid: &uuid::Uuid) -> Self {
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

impl std::fmt::Display for WorkspaceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "HttpTypes.ts")]
pub enum ContainerKind {
    #[serde(rename = "demiurge")]
    Demiurge,
    #[serde(rename = "normal")]
    Normal { id: u16 },
}

impl ContainerKind {
    pub fn demiurge() -> Self {
        Self::Demiurge
    }

    pub fn normal(id: u16) -> Option<Self> {
        if id == 0 {
            return None;
        }
        Some(Self::Normal { id })
    }

    pub fn is_demiurge(&self) -> bool {
        matches!(self, Self::Demiurge)
    }

    pub fn to_badge_string(&self) -> String {
        match self {
            Self::Demiurge => "#demiurge".to_string(),
            Self::Normal { id } => format!("#{:03}", id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WorkspaceScopedBadge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_ref: Option<WorkspaceRef>,
    pub container: ContainerKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl WorkspaceScopedBadge {
    pub fn new(workspace_ref: Option<WorkspaceRef>, container: ContainerKind) -> Self {
        Self {
            workspace_ref,
            container,
            raw: None,
        }
    }

    pub fn local(container: ContainerKind) -> Self {
        Self {
            workspace_ref: None,
            container,
            raw: None,
        }
    }

    pub fn is_workspace_local(&self) -> bool {
        self.workspace_ref.is_none()
    }

    pub fn is_demiurge(&self) -> bool {
        self.container.is_demiurge()
    }

    pub fn to_badge_string(&self) -> String {
        match &self.workspace_ref {
            Some(ws) => format!("{}{}", ws, self.container.to_badge_string()),
            None => self.container.to_badge_string(),
        }
    }
}

impl std::fmt::Display for WorkspaceScopedBadge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.workspace_ref {
            Some(ws) => write!(f, "{}{}", ws, self.container.to_badge_string()),
            None => write!(f, "{}", self.container.to_badge_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WorkspaceScopedSessionId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_ref: Option<WorkspaceRef>,
    pub container: ContainerKind,
    pub session_number: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl WorkspaceScopedSessionId {
    pub fn to_badge_string(&self) -> String {
        match &self.workspace_ref {
            Some(ws) => format!(
                "{}{}.{:03}",
                ws,
                self.container.to_badge_string(),
                self.session_number
            ),
            None => format!("{}.{:03}", self.container.to_badge_string(), self.session_number),
        }
    }
}

impl std::fmt::Display for WorkspaceScopedSessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.workspace_ref {
            Some(ws) => write!(
                f,
                "{}{}.{:03}",
                ws,
                self.container.to_badge_string(),
                self.session_number
            ),
            None => write!(f, "{}.{:03}", self.container.to_badge_string(), self.session_number),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct BindingId {
    pub platform: String,
    pub external_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl BindingId {
    pub fn new(platform: impl Into<String>, external_id: impl Into<String>) -> Self {
        Self {
            platform: platform.into(),
            external_id: external_id.into(),
            floor: None,
            raw: None,
        }
    }

    pub fn with_floor(mut self, floor: u64) -> Self {
        self.floor = Some(floor);
        self
    }

    pub fn to_binding_string(&self) -> String {
        match self.floor {
            Some(f) => format!("@{}#{}@#{}", self.platform, self.external_id, f),
            None => format!("@{}#{}", self.platform, self.external_id),
        }
    }
}

impl std::fmt::Display for BindingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_binding_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "HttpTypes.ts")]
pub enum MessageRef {
    #[serde(rename = "workspace_badge")]
    WorkspaceBadge(WorkspaceScopedBadge),
    #[serde(rename = "workspace_session")]
    WorkspaceSession(WorkspaceScopedSessionId),
    #[serde(rename = "binding")]
    Binding(BindingId),
}

pub fn sanitize_alias(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else if c == ' ' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

pub fn parse_message_refs(text: &str) -> Vec<MessageRef> {
    let mut refs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for part in text.split_whitespace() {
        let key = part.to_string();
        if seen.contains(&key) {
            continue;
        }

        if let Some(binding) = try_parse_binding(part) {
            seen.insert(key);
            refs.push(MessageRef::Binding(binding));
            continue;
        }

        if let Some(session) = try_parse_workspace_session(part) {
            seen.insert(key);
            refs.push(MessageRef::WorkspaceSession(session));
            continue;
        }

        if let Some(badge) = try_parse_workspace_badge(part) {
            seen.insert(key);
            refs.push(MessageRef::WorkspaceBadge(badge));
            continue;
        }
    }

    refs
}

fn try_parse_workspace_badge(s: &str) -> Option<WorkspaceScopedBadge> {
    if let Some((ws_ref, rest)) = WorkspaceRef::parse_from_prefix(s) {
        let container = parse_container_badge(rest)?;
        Some(WorkspaceScopedBadge {
            workspace_ref: Some(ws_ref),
            container,
            raw: Some(s.to_string()),
        })
    } else {
        let container = parse_container_badge(s)?;
        if container.is_demiurge() {
            return None;
        }
        Some(WorkspaceScopedBadge {
            workspace_ref: None,
            container,
            raw: Some(s.to_string()),
        })
    }
}

fn try_parse_workspace_session(s: &str) -> Option<WorkspaceScopedSessionId> {
    let dot_pos = s.rfind('.')?;
    let badge_part = &s[..dot_pos];
    let session_str = &s[dot_pos + 1..];

    if session_str.len() != 3 || !session_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let session_number: u16 = session_str.parse().ok()?;
    if session_number == 0 {
        return None;
    }

    let (ws_ref, container) = if let Some((ws_ref, rest)) = WorkspaceRef::parse_from_prefix(badge_part) {
        let container = parse_container_badge(rest)?;
        (Some(ws_ref), container)
    } else {
        let container = parse_container_badge(badge_part)?;
        if container.is_demiurge() {
            return None;
        }
        (None, container)
    };

    Some(WorkspaceScopedSessionId {
        workspace_ref: ws_ref,
        container,
        session_number,
        raw: Some(s.to_string()),
    })
}

fn try_parse_binding(s: &str) -> Option<BindingId> {
    let rest = s.strip_prefix('@')?;
    let hash_pos = rest.find('#')?;
    let platform = &rest[..hash_pos];
    let remainder = &rest[hash_pos + 1..];

    if platform.is_empty() || remainder.is_empty() {
        return None;
    }

    if is_valid_platform(platform) {
        if let Some(at_hash_pos) = remainder.find("@#") {
            let external_id = &remainder[..at_hash_pos];
            let floor_str = &remainder[at_hash_pos + 2..];
            if external_id.is_empty() || floor_str.is_empty() {
                return None;
            }
            let floor: u64 = floor_str.parse().ok()?;
            return Some(BindingId {
                platform: platform.to_string(),
                external_id: external_id.to_string(),
                floor: Some(floor),
                raw: Some(s.to_string()),
            });
        }
        return Some(BindingId {
            platform: platform.to_string(),
            external_id: remainder.to_string(),
            floor: None,
            raw: Some(s.to_string()),
        });
    }
    None
}

fn is_valid_platform(s: &str) -> bool {
    matches!(
        s,
        "github"
            | "gitee"
            | "gitlab"
            | "feishu"
            | "discord"
            | "telegram"
            | "device"
            | "slack"
            | "lark"
            | "wecom"
            | "qqbot"
    )
}

fn parse_container_badge(s: &str) -> Option<ContainerKind> {
    let s = s.strip_prefix('#')?;
    if s.is_empty() {
        return None;
    }
    if s == "demiurge" {
        return Some(ContainerKind::Demiurge);
    }
    if s.len() == 3 && s.chars().all(|c| c.is_ascii_digit()) {
        let num: u16 = s.parse().ok()?;
        return ContainerKind::normal(num);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_ref_from_uuid_short() {
        let uid = uuid::Uuid::parse_str("3a7bc1d2-e4f5-6789-0abc-def012345678").unwrap();
        let ws = WorkspaceRef::from_uuid_short(&uid);
        assert!(ws.is_short_id());
        assert_eq!(ws.as_str(), "345678");
    }

    #[test]
    fn workspace_ref_parse_short_id() {
        let (ws, rest) = WorkspaceRef::parse_from_prefix("@a1b2c3#demiurge").unwrap();
        assert!(ws.is_short_id());
        assert_eq!(ws.as_str(), "a1b2c3");
        assert_eq!(rest, "#demiurge");
    }

    #[test]
    fn workspace_ref_parse_alias() {
        let (ws, rest) = WorkspaceRef::parse_from_prefix("@myproject#001").unwrap();
        assert!(ws.is_alias());
        assert_eq!(ws.as_str(), "myproject");
        assert_eq!(rest, "#001");
    }

    #[test]
    fn parse_message_refs_workspace_badge() {
        let refs = parse_message_refs("check @a1b2c3#demiurge for details");
        assert_eq!(refs.len(), 1);
        match &refs[0] {
            MessageRef::WorkspaceBadge(b) => {
                assert_eq!(b.to_badge_string(), "@a1b2c3#demiurge");
            }
            _ => panic!("expected WorkspaceBadge"),
        }
    }

    #[test]
    fn parse_message_refs_binding() {
        let refs = parse_message_refs("see @github#123 for context");
        assert_eq!(refs.len(), 1);
        match &refs[0] {
            MessageRef::Binding(b) => {
                assert_eq!(b.platform, "github");
                assert_eq!(b.external_id, "123");
            }
            _ => panic!("expected Binding"),
        }
    }

    #[test]
    fn parse_message_refs_session() {
        let refs = parse_message_refs("@a1b2c3#demiurge.002 is running");
        assert_eq!(refs.len(), 1);
        match &refs[0] {
            MessageRef::WorkspaceSession(s) => {
                assert_eq!(s.session_number, 2);
                assert_eq!(s.to_badge_string(), "@a1b2c3#demiurge.002");
            }
            _ => panic!("expected WorkspaceSession"),
        }
    }

    #[test]
    fn parse_message_refs_mixed() {
        let refs = parse_message_refs("@a1b2c3#demiurge @github#456 #123");
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn sanitize_alias_basic() {
        assert_eq!(sanitize_alias("My Cool Project"), "my-cool-project");
        assert_eq!(sanitize_alias("proj@123!"), "proj_123_");
    }

    #[test]
    fn binding_with_floor() {
        let refs = parse_message_refs("@github#123@#5 comment");
        assert_eq!(refs.len(), 1);
        match &refs[0] {
            MessageRef::Binding(b) => {
                assert_eq!(b.floor, Some(5));
                assert_eq!(b.to_binding_string(), "@github#123@#5");
            }
            _ => panic!("expected Binding"),
        }
    }
}
