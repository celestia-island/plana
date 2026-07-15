use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    Local,
    Remote,
}

impl std::fmt::Display for ContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextType::Local => write!(f, "local"),
            ContextType::Remote => write!(f, "remote"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionContext {
    pub name: String,
    #[serde(rename = "type")]
    pub ctx_type: ContextType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ws_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The last workspace opened via `workspace open`. Used by `send` to
    /// route user messages to the correct workspace context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

impl ConnectionContext {
    pub fn local_default() -> Self {
        Self {
            name: "default".to_string(),
            ctx_type: ContextType::Local,
            socket_path: None,
            ws_url: None,
            bearer_token: None,
            description: Some("Local Unix socket".to_string()),
            workspace_id: None,
        }
    }

    pub fn resolve_socket_path(&self) -> std::path::PathBuf {
        if let Some(ref p) = self.socket_path {
            std::path::PathBuf::from(p)
        } else {
            let dir = std::env::var("ENTELECHEIA_SOCKET_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("/run/entelecheia"));
            dir.join("entelecheia-tui.sock")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextStore {
    #[serde(default)]
    pub contexts: Vec<ConnectionContext>,
    #[serde(default)]
    pub current_context: Option<String>,
}

impl ContextStore {
    pub fn contexts_dir() -> PathBuf {
        super::app_config::UserConfig::config_dir().join("contexts")
    }

    pub fn store_file() -> PathBuf {
        Self::contexts_dir().join("contexts.toml")
    }

    pub fn load() -> Self {
        let file = Self::store_file();
        if file.exists()
            && let Ok(content) = std::fs::read_to_string(&file)
            && let Ok(store) = toml::from_str(&content)
        {
            return store;
        }
        Self::with_default()
    }

    pub fn with_default() -> Self {
        let default_ctx = ConnectionContext::local_default();
        Self {
            contexts: vec![default_ctx],
            current_context: Some("default".to_string()),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Self::contexts_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(Self::store_file(), content)
    }

    pub fn current(&self) -> Option<&ConnectionContext> {
        self.current_context
            .as_ref()
            .and_then(|name| self.contexts.iter().find(|c| c.name == *name))
    }

    pub fn current_mut(&mut self) -> Option<&mut ConnectionContext> {
        let name = self.current_context.clone()?;
        self.contexts.iter_mut().find(|c| c.name == name)
    }

    pub fn add(&mut self, ctx: ConnectionContext) -> Result<()> {
        if self.contexts.iter().any(|c| c.name == ctx.name) {
            bail!("context '{}' already exists", ctx.name);
        }
        self.contexts.push(ctx);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        if name == "default" {
            bail!("cannot remove the default context");
        }
        let idx = self
            .contexts
            .iter()
            .position(|c| c.name == name)
            .ok_or_else(|| anyhow!("context '{}' not found", name))?;
        self.contexts.remove(idx);
        if self.current_context.as_deref() == Some(name) {
            self.current_context = Some("default".to_string());
        }
        Ok(())
    }

    pub fn use_context(&mut self, name: &str) -> Result<()> {
        if !self.contexts.iter().any(|c| c.name == name) {
            bail!("context '{}' not found", name);
        }
        self.current_context = Some(name.to_string());
        Ok(())
    }
}
