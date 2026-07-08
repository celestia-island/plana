use serde::Deserialize;

#[derive(serde::Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicContent>,
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub usage: Option<AnthropicUsageResponse>,
}

#[derive(serde::Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContent {
    Text {
        text: Option<String>,
    },
    ToolUse {
        id: Option<String>,
        name: Option<String>,
        input: Option<serde_json::Value>,
    },
    Thinking {
        thinking: Option<String>,
    },
}

#[derive(serde::Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicUsageResponse {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicStreamEvent {
    MessageStart {
        message: AnthropicStreamMessageStart,
    },
    ContentBlockStart {
        content_block: AnthropicStreamContentBlockStart,
    },
    ContentBlockDelta {
        delta: AnthropicStreamDelta,
    },
    MessageDelta {
        delta: AnthropicStreamMessageDeltaInfo,
        usage: AnthropicStreamMessageDeltaUsage,
    },
    MessageStop,
}

#[derive(Deserialize, Debug)]
pub struct AnthropicStreamMessageStart {
    pub usage: AnthropicStreamUsageStart,
}

#[derive(Deserialize, Debug)]
pub struct AnthropicStreamUsageStart {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicStreamContentBlockStart {
    ToolUse {
        id: String,
        name: String,
    },
    Text {
        #[serde(rename = "text")]
        _text: String,
    },
    Thinking {
        #[serde(rename = "thinking")]
        _thinking: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AnthropicStreamDelta {
    #[serde(rename = "text_delta")]
    Text { text: String },
    #[serde(rename = "input_json_delta")]
    InputJson { partial_json: String },
    #[serde(rename = "thinking_delta")]
    Thinking {
        #[serde(rename = "thinking")]
        _thinking: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct AnthropicStreamMessageDeltaInfo {
    pub stop_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AnthropicStreamMessageDeltaUsage {
    #[serde(default)]
    pub output_tokens: u64,
}
