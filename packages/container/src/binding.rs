use anyhow::{Context, Error, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BindingId {
    pub platform: String,
    pub external_id: String,
    pub floor: Option<u64>,
}

impl BindingId {
    pub fn new(platform: impl Into<String>, external_id: impl Into<String>) -> Self {
        Self {
            platform: platform.into(),
            external_id: external_id.into(),
            floor: None,
        }
    }

    pub fn with_floor(mut self, floor: u64) -> Self {
        self.floor = Some(floor);
        self
    }

    pub fn parse(s: &str) -> Result<Self> {
        if !s.starts_with('@') {
            bail!("binding ID must start with '@'");
        }
        let rest = &s[1..];
        let parts: Vec<&str> = rest.splitn(2, '#').collect();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            bail!("invalid format, expected @platform#id[#floor]");
        }
        let platform = parts[0].to_string();
        let id_and_floor: Vec<&str> = parts[1].splitn(2, '#').collect();
        let external_id = id_and_floor[0].to_string();
        let floor = if id_and_floor.len() > 1 {
            Some(id_and_floor[1].parse::<u64>().with_context(|| {
                format!(
                    "floor must be a positive integer, got '{}'",
                    id_and_floor[1]
                )
            })?)
        } else {
            None
        };
        Ok(Self {
            platform,
            external_id,
            floor,
        })
    }

    pub fn primary(&self) -> Self {
        Self {
            platform: self.platform.clone(),
            external_id: self.external_id.clone(),
            floor: None,
        }
    }

    pub fn to_branch_name_segment(&self) -> String {
        format!("{}", self)
    }

    pub fn platform_prefix(&self) -> &str {
        &self.platform
    }
}

impl fmt::Display for BindingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.floor {
            Some(floor) => write!(f, "@{}#{}#{}", self.platform, self.external_id, floor),
            None => write!(f, "@{}#{}", self.platform, self.external_id),
        }
    }
}

impl std::str::FromStr for BindingId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, arona_macros::Getters)]
pub struct ContainerBinding {
    pub container_uuid: Uuid,
    pub binding_id: BindingId,
    #[getter(skip)]
    pub bound_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, arona_macros::Getters)]
pub struct ContainerBindResult {
    pub container_uuid: Uuid,
    pub binding_id: String,
    pub total_bindings: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn parse_simple() -> Result<()> {
        let id = BindingId::parse("@github#234")?;
        assert_eq!(id.platform, "github");
        assert_eq!(id.external_id, "234");
        assert_eq!(id.floor, None);
        Ok(())
    }

    #[test]
    fn parse_with_floor() -> Result<()> {
        let id = BindingId::parse("@github#234#5")?;
        assert_eq!(id.platform, "github");
        assert_eq!(id.external_id, "234");
        assert_eq!(id.floor, Some(5));
        Ok(())
    }

    #[test]
    fn parse_uuid_external_id() -> Result<()> {
        let id = BindingId::parse("@qq#a8sv71-4f2c")?;
        assert_eq!(id.platform, "qq");
        assert_eq!(id.external_id, "a8sv71-4f2c");
        Ok(())
    }

    #[test]
    fn parse_external_id_with_hyphens() -> Result<()> {
        let id = BindingId::parse("@feishu#OKR-Q2-003")?;
        assert_eq!(id.platform, "feishu");
        assert_eq!(id.external_id, "OKR-Q2-003");
        Ok(())
    }

    #[test]
    fn parse_errors() -> Result<()> {
        assert!(BindingId::parse("github#234").is_err());
        assert!(BindingId::parse("@#234").is_err());
        assert!(BindingId::parse("@github#").is_err());
        assert!(BindingId::parse("@github").is_err());
        Ok(())
    }

    #[test]
    fn display_roundtrip() -> Result<()> {
        let id = BindingId::parse("@gitlab#158#3")?;
        assert_eq!(format!("{}", id), "@gitlab#158#3");
        Ok(())
    }

    #[test]
    fn primary_strips_floor() -> Result<()> {
        let id = BindingId::parse("@github#234#5")?;
        let primary = id.primary();
        assert_eq!(primary.floor, None);
        assert_eq!(format!("{}", primary), "@github#234");
        Ok(())
    }
}
