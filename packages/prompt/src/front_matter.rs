#[derive(Debug, Clone, PartialEq)]
pub struct FrontMatterParts<'a> {
    pub toml_text: &'a str,
    pub body: &'a str,
}

impl<'a> FrontMatterParts<'a> {
    pub fn parse_toml<T: serde::de::DeserializeOwned>(&self) -> Result<T, toml::de::Error> {
        toml::from_str(self.toml_text)
    }

    pub fn parse_toml_value(&self) -> Result<toml::Value, toml::de::Error> {
        toml::from_str(self.toml_text)
    }
}

pub fn extract_front_matter(content: &str) -> Option<FrontMatterParts<'_>> {
    let start = content.find("+++")?;
    let after_first = start + 3;
    let rest = content.get(after_first..)?;
    let end_in_rest = rest.find("+++")?;
    let toml_text = rest[..end_in_rest].trim();
    let body_start = after_first + end_in_rest + 3;
    let body = content
        .get(body_start..)
        .map(|s| s.trim_start_matches(['\r', '\n']))
        .unwrap_or("");
    Some(FrontMatterParts { toml_text, body })
}

pub fn strip_front_matter(content: &str) -> &str {
    extract_front_matter(content)
        .map(|p| p.body)
        .unwrap_or(content)
}
