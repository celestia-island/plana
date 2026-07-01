//! Shared domain vocabulary enums.
//!
//! Platform-wide enumerations consumed by MCP tool types and protocol
//! messages. These are *vocabulary*, not tool I/O shapes, so they live at the
//! crate root rather than under `mcp/`. Per-agent tool request/result structs
//! (see `mcp/`) reference the types defined here.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! str_enum {
    ($name:ident { $($variant:ident = $val:literal),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS, schemars::JsonSchema)]
        #[ts(export, export_to = "enums.ts")]
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $($name::$variant => $val,)*
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl From<$name> for String {
            fn from(v: $name) -> String {
                v.as_str().to_string()
            }
        }
    };
}

str_enum!(FileOpStatus {
    Created = "created",
    Deleted = "deleted",
    Edited = "edited",
    Written = "written",
});

str_enum!(FileType {
    File = "file",
    Directory = "directory",
});

str_enum!(ContainerOpStatus {
    Created = "created",
    Running = "running",
    Stopped = "stopped",
    Removed = "removed",
    Forked = "forked",
});

str_enum!(ConsultationStatus {
    WaitingHuman = "waiting_human",
    Pending = "pending",
    Answered = "answered",
    Delivered = "delivered",
    Scheduled = "scheduled",
    Triggered = "triggered",
    Cancelled = "cancelled",
    Replied = "replied",
});

str_enum!(WebSearchEngine {
    Duckduckgo = "duckduckgo",
});

str_enum!(ScriptLanguage {
    Bash = "bash",
    Sh = "sh",
    Python = "python",
    Python3 = "python3",
    Javascript = "javascript",
    Typescript = "typescript",
    Node = "node",
    Zsh = "zsh",
    Layer2 = "layer2",
});

str_enum!(ObservationType {
    Reading = "reading",
    Editing = "editing",
    Deleting = "deleting",
    Watching = "watching",
});

str_enum!(FileOperationType {
    Reading = "Reading",
    Editing = "Editing",
    Deleting = "Deleting",
});

str_enum!(ConversationStatus {
    Active = "Active",
    Resolved = "Resolved",
    Deadlocked = "Deadlocked",
    Escalated = "Escalated",
});

str_enum!(ConversationMessageType {
    Question = "Question",
    Answer = "Answer",
    Clarification = "Clarification",
    Objection = "Objection",
    CounterProposal = "CounterProposal",
    Resolution = "Resolution",
});

str_enum!(AnnotationType {
    Note = "note",
    Warning = "warning",
    Todo = "todo",
    Suggestion = "suggestion",
    Conflict = "conflict",
});

str_enum!(GoalStatus {
    Active = "active",
    Completed = "completed",
    Abandoned = "abandoned",
});

str_enum!(TrackStatus {
    Active = "active",
    Completed = "completed",
    Abandoned = "abandoned",
});

str_enum!(GoalTaskStatus {
    Pending = "pending",
    InProgress = "in_progress",
    Completed = "completed",
    Failed = "failed",
    Cancelled = "cancelled",
});
