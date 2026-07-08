use std::collections::HashMap;

use tracing::debug;

use crate::{Agent, McpToolInfo, McpToolParameters};

const MARKDOWN_SECTION_PARAMS: &str = "## Parameters";
const MARKDOWN_SECTION_PREFIX: &str = "## ";
const TYPE_ARRAY_PREFIX: &str = "array";

#[derive(Debug, Clone)]
pub struct McpToolDoc {
    pub description: String,
    pub parameters: McpToolParameters,
    pub body: String,
}

pub struct McpToolDocLoader;

struct ParsedParam {
    name: String,
    required: bool,
    separate_call: bool,
    schema: serde_json::Value,
}

fn make_prop_schema(type_str: &str, desc: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "type".to_string(),
        serde_json::Value::String(type_str.to_string()),
    );
    map.insert(
        "description".to_string(),
        serde_json::Value::String(desc.to_string()),
    );
    serde_json::Value::Object(map)
}

fn make_array_prop_schema(desc: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "type".to_string(),
        serde_json::Value::String("array".to_string()),
    );
    map.insert(
        "description".to_string(),
        serde_json::Value::String(desc.to_string()),
    );
    let mut items = serde_json::Map::new();
    items.insert(
        "type".to_string(),
        serde_json::Value::String("string".to_string()),
    );
    map.insert("items".to_string(), serde_json::Value::Object(items));
    serde_json::Value::Object(map)
}

impl McpToolDocLoader {
    pub fn load(agent_name: &str, tool_name: &str, lang: &str) -> Option<McpToolDoc> {
        let mcp_path = std::path::Path::new("res/prompts/agents")
            .join(agent_name)
            .join("mcp")
            .join(format!("{}.md", tool_name));

        let skills_path = std::path::Path::new("res/prompts/agents")
            .join(agent_name)
            .join("skills")
            .join(format!("{}.md", tool_name));

        let content = if let Ok(c) = std::fs::read_to_string(&mcp_path) {
            c
        } else if let Ok(c) = std::fs::read_to_string(&skills_path) {
            debug!(
                agent = agent_name,
                tool = tool_name,
                "Loaded skill doc as fallback (not an MCP tool, but related_skills entry)"
            );
            c
        } else {
            debug!(
                agent = agent_name,
                tool = tool_name,
                "MCP tool doc file not found, skipping (skill docs are not MCP tools)"
            );
            return None;
        };

        let (front_matter, body) = Self::split_front_matter(&content)?;
        let toml_value: toml::Value = toml::from_str(&front_matter).ok()?;

        let description = Self::extract_description(&toml_value, lang).unwrap_or_else(|| {
            toml_value
                .get("description")
                .and_then(|d| d.get("en"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        });

        let parameters = Self::parse_parameters_from_body(&body);

        Some(McpToolDoc {
            description,
            parameters,
            body,
        })
    }

    pub fn load_from_content(content: &str, lang: &str) -> Option<McpToolDoc> {
        let (front_matter, body) = Self::split_front_matter(content)?;
        let toml_value: toml::Value = toml::from_str(&front_matter).ok()?;

        let description = Self::extract_description(&toml_value, lang).unwrap_or_else(|| {
            toml_value
                .get("description")
                .and_then(|d| d.get("en"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        });

        let parameters = Self::parse_parameters_from_body(&body);

        Some(McpToolDoc {
            description,
            parameters,
            body,
        })
    }

    fn split_front_matter(content: &str) -> Option<(String, String)> {
        let start = content.find("+++")?;
        let rest = &content[start + 3..];
        let end = rest.find("+++")?;
        let front_matter = rest[..end].trim().to_string();
        let body = rest[end + 3..].trim().to_string();
        Some((front_matter, body))
    }

    fn extract_description(toml_value: &toml::Value, lang: &str) -> Option<String> {
        let desc_table = toml_value.get("description")?.as_table()?;
        let normalized = match lang {
            "zh" | "zhs" => "zhs",
            "zht" => "zht",
            "ja" => "ja",
            "ko" => "ko",
            "fr" => "fr",
            "es" => "es",
            "ru" => "ru",
            _ => "en",
        };
        desc_table
            .get(normalized)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                desc_table
                    .get("en")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
    }

    fn parse_parameters_from_body(body: &str) -> McpToolParameters {
        let mut required = Vec::new();
        let mut properties = HashMap::new();
        let mut separate_call_keys = Vec::new();

        let in_params = false;
        let lines = body.lines().peekable();
        let mut in_params_section = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with(MARKDOWN_SECTION_PARAMS) {
                in_params_section = true;
                continue;
            }

            if in_params_section && trimmed.starts_with(MARKDOWN_SECTION_PREFIX) {
                break;
            }

            if in_params_section && let Some(param) = Self::parse_param_line(trimmed) {
                if param.required {
                    required.push(param.name.clone());
                }
                if param.separate_call {
                    separate_call_keys.push(param.name.clone());
                }
                properties.insert(param.name, param.schema);
            }
        }

        let _ = in_params;

        McpToolParameters {
            param_type: "object".to_string(),
            required,
            properties,
            separate_call_keys,
        }
    }

    fn parse_param_line(line: &str) -> Option<ParsedParam> {
        let re = regex::Regex::new(r#"^-?\s*\*\*(.+?)\*\*\s*\((.+?)\)\s*:\s*(.+)$"#).ok()?;

        let caps = re.captures(line)?;
        let name = caps.get(1)?.as_str().to_string();
        let type_and_modifier = caps.get(2)?.as_str().trim();
        let desc = caps.get(3)?.as_str().trim().to_string();

        let parts: Vec<&str> = type_and_modifier.split(',').map(|s| s.trim()).collect();
        let type_str = parts.first()?;
        let is_required = parts.contains(&"required");
        let is_separate_call = parts.contains(&"separate-call");

        let schema = match *type_str {
            "string" => make_prop_schema("string", &desc),
            "boolean" => make_prop_schema("boolean", &desc),
            "integer" | "number" => make_prop_schema("integer", &desc),
            t if t.starts_with(TYPE_ARRAY_PREFIX) => make_array_prop_schema(&desc),
            "object" => make_prop_schema("object", &desc),
            _ => make_prop_schema("string", &desc),
        };

        Some(ParsedParam {
            name,
            required: is_required,
            separate_call: is_separate_call,
            schema,
        })
    }

    pub fn enrich_tool_info(info: &mut McpToolInfo, agent: &Agent, lang: &str) {
        let agent_name = agent.folder_name();
        match Self::load(agent_name, &info.name, lang) {
            Some(doc) => {
                if !doc.description.is_empty() {
                    info.description = doc.description;
                }
                if !doc.parameters.properties.is_empty() {
                    info.parameters = doc.parameters;
                }
            },
            None => {
                debug!(
                    agent = agent_name,
                    tool = %info.name,
                    "McpToolDocLoader: no doc file found, using default parameters"
                );
            },
        }
    }
}
