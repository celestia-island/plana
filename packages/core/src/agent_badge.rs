use serde::{Deserialize, Serialize};
use std::fmt;

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
        if let Some(dot_pos) = bare.find('.') {
            let parent = &bare[..dot_pos];
            let child = &bare[dot_pos + 1..];
            !parent.is_empty() && child.len() == 3 && child.chars().all(|c| c.is_ascii_digit())
        } else {
            bare == "demiurge" || (bare.len() == 3 && bare.chars().all(|c| c.is_ascii_digit()))
        }
    }

    pub fn is_sub_badge(&self) -> bool {
        self.0.contains('.')
    }

    pub fn parent_key(&self) -> Option<&str> {
        self.0.find('.').map(|pos| &self.0[..pos])
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
