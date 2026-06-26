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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpServersFile {
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
}

#[derive(Debug, Clone)]
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
}
