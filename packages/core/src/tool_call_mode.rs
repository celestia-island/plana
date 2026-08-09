use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallMode {
    FireAndForget,
    #[default]
    Blocking,
    AsyncCallback,
}
