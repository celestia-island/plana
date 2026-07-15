use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown execution mode: {0}")]
pub struct UnknownExecutionModeError(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ExecutionMode {
    #[default]
    Read,
    Write,
    Edge,
}

impl std::str::FromStr for ExecutionMode {
    type Err = UnknownExecutionModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(ExecutionMode::Read),
            "write" => Ok(ExecutionMode::Write),
            "edge" => Ok(ExecutionMode::Edge),
            "query" => Ok(ExecutionMode::Read),
            "read_only" => Ok(ExecutionMode::Read),
            _ => Err(UnknownExecutionModeError(s.to_string())),
        }
    }
}

impl ExecutionMode {
    pub fn is_read(&self) -> bool {
        matches!(self, ExecutionMode::Read)
    }

    pub fn is_write(&self) -> bool {
        matches!(self, ExecutionMode::Write)
    }

    pub fn is_edge(&self) -> bool {
        matches!(self, ExecutionMode::Edge)
    }

    pub fn is_query(&self) -> bool {
        self.is_read()
    }

    pub fn is_read_only(&self) -> bool {
        self.is_read()
    }

    pub fn needs_container(&self) -> bool {
        true
    }

    pub fn needs_write_access(&self) -> bool {
        matches!(self, ExecutionMode::Write | ExecutionMode::Edge)
    }
}
