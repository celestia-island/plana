//! External (third-party) MCP server connection configuration.
//!
//! These types describe how scepter connects to *out-of-tree* MCP servers
//! declared in the operator's `mcp_servers` TOML file. They are internal
//! config types — not exported to the TypeScript bindings — and are distinct
//! from the per-agent tool I/O structs under `mcp/`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Sse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub transport: TransportType,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub denylist: Vec<String>,
    /// Per-server environment variables the spawned stdio server receives on
    /// top of the consumer's allowlisted baseline. Consumers do not forward
    /// their own process environment to MCP servers, so anything the server
    /// needs beyond that baseline must be declared here; consumers resolve
    /// `${VAR}` interpolations in the values themselves.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpServersFile {
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalToolInfo {
    pub server_name: String,
    pub tool_name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn parse_empty_file() -> Result<()> {
        let raw = "";
        let file: McpServersFile = toml::from_str(raw).context("test precondition")?;
        assert!(file.mcp_servers.is_empty());
        Ok(())
    }

    #[test]
    fn parse_single_stdio_server() -> Result<()> {
        let raw = r#"
[[mcp_servers]]
name = "filesystem"
transport = "stdio"
command = ["npx", "-y", "@anthropic/mcp-server-filesystem"]
args = ["/home/user/projects"]
"#;
        let file: McpServersFile = toml::from_str(raw).context("test precondition")?;
        assert_eq!(file.mcp_servers.len(), 1);
        let server = &file.mcp_servers[0];
        assert_eq!(server.name, "filesystem");
        assert_eq!(server.transport, TransportType::Stdio);
        assert_eq!(
            server.command,
            vec!["npx", "-y", "@anthropic/mcp-server-filesystem"]
        );
        assert_eq!(server.args, vec!["/home/user/projects"]);
        Ok(())
    }

    #[test]
    fn parse_multiple_servers() -> Result<()> {
        let raw = r#"
[[mcp_servers]]
name = "filesystem"
transport = "stdio"
command = ["npx", "-y", "@anthropic/mcp-server-filesystem"]

[[mcp_servers]]
name = "postgres"
transport = "sse"
endpoint = "https://pg-mcp.example.com/mcp"
"#;
        let file: McpServersFile = toml::from_str(raw).context("test precondition")?;
        assert_eq!(file.mcp_servers.len(), 2);
        assert_eq!(file.mcp_servers[0].transport, TransportType::Stdio);
        assert_eq!(file.mcp_servers[1].transport, TransportType::Sse);
        assert_eq!(
            file.mcp_servers[1].endpoint.as_deref(),
            Some("https://pg-mcp.example.com/mcp")
        );
        Ok(())
    }

    #[test]
    fn parse_server_with_denylist() -> Result<()> {
        let raw = r#"
[[mcp_servers]]
name = "filesystem"
transport = "stdio"
command = ["npx", "mcp-server"]
denylist = ["rm_rf", "format_disk"]
"#;
        let file: McpServersFile = toml::from_str(raw).context("test precondition")?;
        assert_eq!(file.mcp_servers[0].denylist, vec!["rm_rf", "format_disk"]);
        Ok(())
    }

    #[test]
    fn parse_commented_out_file() -> Result<()> {
        let raw = r#"
# [[mcp_servers]]
# name = "filesystem"
# transport = "stdio"
# command = ["npx"]
"#;
        let file: McpServersFile = toml::from_str(raw).context("test precondition")?;
        assert!(file.mcp_servers.is_empty());
        Ok(())
    }

    #[test]
    fn parse_server_with_env_map() -> Result<()> {
        let raw = r#"
[[mcp_servers]]
name = "files"
transport = "stdio"
command = ["uvx", "mcp-server-files"]
env = { FILES_ROOT = "/srv/files", API_KEY = "${FILES_API_KEY}" }
"#;
        let file: McpServersFile = toml::from_str(raw).context("test precondition")?;
        let server = file.mcp_servers.first().context("server must parse")?;
        assert_eq!(
            server.env.get("FILES_ROOT").map(String::as_str),
            Some("/srv/files")
        );
        // Interpolation placeholders are carried verbatim; the consumer
        // resolves them against its own environment when spawning.
        assert_eq!(
            server.env.get("API_KEY").map(String::as_str),
            Some("${FILES_API_KEY}")
        );
        Ok(())
    }

    #[test]
    fn env_defaults_to_empty() -> Result<()> {
        let raw = r#"
[[mcp_servers]]
name = "files"
transport = "stdio"
command = ["uvx", "mcp-server-files"]
"#;
        let file: McpServersFile = toml::from_str(raw).context("test precondition")?;
        let server = file.mcp_servers.first().context("server must parse")?;
        assert!(server.env.is_empty());
        Ok(())
    }
}
