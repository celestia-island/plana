pub trait ToolIdentity: std::fmt::Debug + Clone + Send + Sync {
    fn as_str(&self) -> &str;
}

impl ToolIdentity for String {
    fn as_str(&self) -> &str {
        self
    }
}

impl ToolIdentity for &str {
    fn as_str(&self) -> &str {
        self
    }
}
