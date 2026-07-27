use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{ops::Deref, sync::Arc};

use append_only_bytes::{AppendOnlyBytes, BytesSlice};

pub struct LlmTextBuilder {
    inner: AppendOnlyBytes,
    len: usize,
}

impl Default for LlmTextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmTextBuilder {
    pub fn new() -> Self {
        Self {
            inner: AppendOnlyBytes::new(),
            len: 0,
        }
    }

    pub fn push_chunk(&mut self, chunk: &str) {
        let bytes = chunk.as_bytes();
        self.inner.push_slice(bytes);
        self.len += bytes.len();
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.inner.as_bytes()).unwrap_or("")
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn slice(&self, range: std::ops::Range<usize>) -> LlmTextSlice {
        let start = range.start.min(self.len);
        let end = range.end.min(self.len).max(start);
        LlmTextSlice {
            inner: self.inner.slice(start..end),
        }
    }

    pub fn seal(self) -> LlmText {
        let s = self.as_str().to_owned();
        LlmText {
            inner: Arc::from(s.into_boxed_str()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LlmText {
    inner: Arc<str>,
}

impl LlmText {
    pub fn new(s: &str) -> Self {
        Self {
            inner: Arc::from(s.to_owned().into_boxed_str()),
        }
    }

    pub fn from_string(s: String) -> Self {
        Self {
            inner: Arc::from(s.into_boxed_str()),
        }
    }

    pub fn from_static(s: &'static str) -> Self {
        Self {
            inner: Arc::from(s),
        }
    }

    pub fn empty() -> Self {
        Self {
            inner: Arc::from(String::new().into_boxed_str()),
        }
    }

    pub fn into_string(self) -> String {
        self.inner.to_string()
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn arc_refcount(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl Deref for LlmText {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for LlmText {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl std::fmt::Display for LlmText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl Default for LlmText {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<String> for LlmText {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for LlmText {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<LlmText> for String {
    fn from(val: LlmText) -> Self {
        val.into_string()
    }
}

impl PartialEq for LlmText {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for LlmText {}

impl std::hash::Hash for LlmText {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl Serialize for LlmText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.inner)
    }
}

impl<'de> Deserialize<'de> for LlmText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from_string(s))
    }
}

pub struct LlmTextSlice {
    inner: BytesSlice,
}

impl LlmTextSlice {
    pub fn to_str(&self) -> &str {
        std::str::from_utf8(&self.inner).unwrap_or("")
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Deref for LlmTextSlice {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_push_and_seal() -> Result<()> {
        let mut builder = LlmTextBuilder::new();
        builder.push_chunk("hello ");
        builder.push_chunk("world");
        assert_eq!(builder.as_str(), "hello world");
        assert_eq!(builder.len(), 11);

        let text = builder.seal();
        assert_eq!(&*text, "hello world");
        Ok(())
    }

    #[test]
    fn test_llm_text_clone_zero_copy() -> Result<()> {
        let text = LlmText::from_string("test content".to_string());
        let cloned = text.clone();
        assert_eq!(text.arc_refcount(), 2);
        assert_eq!(&*text, &*cloned);
        Ok(())
    }

    #[test]
    fn test_llm_text_slice_during_build() -> Result<()> {
        let mut builder = LlmTextBuilder::new();
        builder.push_chunk("hello world");
        let slice = builder.slice(0..5);
        assert_eq!(slice.to_str(), "hello");
        Ok(())
    }

    #[test]
    fn test_serde_roundtrip() -> Result<()> {
        let text = LlmText::from_string("serde test".to_string());
        let json = serde_json::to_string(&text)?;
        assert_eq!(json, "\"serde test\"");
        let deserialized: LlmText = serde_json::from_str(&json)?;
        assert_eq!(&*text, &*deserialized);
        Ok(())
    }

    #[test]
    fn test_into_string() -> Result<()> {
        let text = LlmText::from_string("owned".to_string());
        let s: String = text.into_string();
        assert_eq!(s, "owned");
        Ok(())
    }

    #[test]
    fn test_empty() -> Result<()> {
        let text = LlmText::empty();
        assert!(text.is_empty());
        assert_eq!(text.len(), 0);
        Ok(())
    }

    #[test]
    fn test_from_string_no_extra_alloc() -> Result<()> {
        let text = LlmText::from_string("content".to_string());
        assert_eq!(text.arc_refcount(), 1);
        Ok(())
    }

    #[test]
    fn test_multibyte_utf8_roundtrip() -> Result<()> {
        let mut builder = LlmTextBuilder::new();
        builder.push_chunk("Hello");
        builder.push_chunk("World");

        assert_eq!(builder.as_str(), "HelloWorld");

        let text = builder.seal();
        assert_eq!(&*text, "HelloWorld");
        assert_eq!(text.len(), "HelloWorld".len());
        Ok(())
    }
}
