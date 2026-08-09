//! Shared domain vocabulary enums.
//!
//! Platform-wide enumerations consumed by MCP tool types and protocol
//! messages. These are *vocabulary*, not tool I/O shapes, so they live at the
//! crate root rather than under `tools/`. Per-agent tool request/result structs
//! (see `tools/`) reference the types defined here. The generic
//! connection-topology vocabulary (`ConnectionType`) lives in
//! `plana-protocol-core`.

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

#[cfg(test)]
mod tests {
    use super::*;

    // ── as_str() / Display / From<String> consistency ──────────────
    //
    // The str_enum! macro generates three traits: as_str(), Display, and
    // From<Enum> for String. All three must agree with the declared wire
    // value. NOTE: serde serialization uses the *PascalCase variant name*
    // (not the as_str() value) — this is documented behavior. The tests
    // below verify both the as_str()/Display/From trio AND the serde
    // representation so the divergence is explicit and guarded.

    #[test]
    fn file_op_status_as_str_values() {
        assert_eq!(FileOpStatus::Created.as_str(), "created");
        assert_eq!(FileOpStatus::Deleted.as_str(), "deleted");
        assert_eq!(FileOpStatus::Edited.as_str(), "edited");
        assert_eq!(FileOpStatus::Written.as_str(), "written");
    }

    #[test]
    fn file_type_as_str_values() {
        assert_eq!(FileType::File.as_str(), "file");
        assert_eq!(FileType::Directory.as_str(), "directory");
    }

    #[test]
    fn container_op_status_as_str_values() {
        assert_eq!(ContainerOpStatus::Created.as_str(), "created");
        assert_eq!(ContainerOpStatus::Running.as_str(), "running");
        assert_eq!(ContainerOpStatus::Stopped.as_str(), "stopped");
        assert_eq!(ContainerOpStatus::Removed.as_str(), "removed");
        assert_eq!(ContainerOpStatus::Forked.as_str(), "forked");
    }

    #[test]
    fn consultation_status_as_str_values() {
        assert_eq!(ConsultationStatus::WaitingHuman.as_str(), "waiting_human");
        assert_eq!(ConsultationStatus::Pending.as_str(), "pending");
        assert_eq!(ConsultationStatus::Answered.as_str(), "answered");
        assert_eq!(ConsultationStatus::Delivered.as_str(), "delivered");
        assert_eq!(ConsultationStatus::Scheduled.as_str(), "scheduled");
        assert_eq!(ConsultationStatus::Triggered.as_str(), "triggered");
        assert_eq!(ConsultationStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(ConsultationStatus::Replied.as_str(), "replied");
    }

    #[test]
    fn script_language_as_str_values() {
        assert_eq!(ScriptLanguage::Bash.as_str(), "bash");
        assert_eq!(ScriptLanguage::Sh.as_str(), "sh");
        assert_eq!(ScriptLanguage::Python.as_str(), "python");
        assert_eq!(ScriptLanguage::Python3.as_str(), "python3");
        assert_eq!(ScriptLanguage::Javascript.as_str(), "javascript");
        assert_eq!(ScriptLanguage::Typescript.as_str(), "typescript");
        assert_eq!(ScriptLanguage::Node.as_str(), "node");
        assert_eq!(ScriptLanguage::Zsh.as_str(), "zsh");
        assert_eq!(ScriptLanguage::Layer2.as_str(), "layer2");
    }

    #[test]
    fn goal_status_as_str_values() {
        assert_eq!(GoalStatus::Active.as_str(), "active");
        assert_eq!(GoalStatus::Completed.as_str(), "completed");
        assert_eq!(GoalStatus::Abandoned.as_str(), "abandoned");
    }

    #[test]
    fn track_status_as_str_values() {
        assert_eq!(TrackStatus::Active.as_str(), "active");
        assert_eq!(TrackStatus::Completed.as_str(), "completed");
        assert_eq!(TrackStatus::Abandoned.as_str(), "abandoned");
    }

    #[test]
    fn goal_task_status_as_str_values() {
        assert_eq!(GoalTaskStatus::Pending.as_str(), "pending");
        assert_eq!(GoalTaskStatus::InProgress.as_str(), "in_progress");
        assert_eq!(GoalTaskStatus::Completed.as_str(), "completed");
        assert_eq!(GoalTaskStatus::Failed.as_str(), "failed");
        assert_eq!(GoalTaskStatus::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn annotation_type_as_str_values() {
        assert_eq!(AnnotationType::Note.as_str(), "note");
        assert_eq!(AnnotationType::Warning.as_str(), "warning");
        assert_eq!(AnnotationType::Todo.as_str(), "todo");
        assert_eq!(AnnotationType::Suggestion.as_str(), "suggestion");
        assert_eq!(AnnotationType::Conflict.as_str(), "conflict");
    }

    // ── Display matches as_str ─────────────────────────────────────

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", ContainerOpStatus::Running), "running");
        assert_eq!(
            format!("{}", ConsultationStatus::WaitingHuman),
            "waiting_human"
        );
        assert_eq!(format!("{}", ScriptLanguage::Typescript), "typescript");
        assert_eq!(format!("{}", GoalTaskStatus::InProgress), "in_progress");
    }

    // ── From<Enum> for String matches as_str ───────────────────────

    #[test]
    fn from_enum_to_string_matches_as_str() {
        let s: String = ContainerOpStatus::Forked.into();
        assert_eq!(s, "forked");
        let s: String = ConsultationStatus::Replied.into();
        assert_eq!(s, "replied");
    }

    // ── serde uses PascalCase variant name (NOT as_str value) ──────
    //
    // This documents a deliberate divergence: serde_serialize produces
    // the Rust variant identifier (e.g. "WaitingHuman"), while as_str()
    // produces the domain vocabulary string (e.g. "waiting_human").
    // Both are stable contracts — changing either would break consumers.

    #[test]
    fn serde_serializes_variant_name_not_as_str() {
        let s = serde_json::to_string(&ConsultationStatus::WaitingHuman).unwrap();
        assert_eq!(s, r#""WaitingHuman""#);
        assert_ne!(s, r#""waiting_human""#);
    }

    #[test]
    fn serde_round_trip_uses_variant_name() {
        let val = ContainerOpStatus::Running;
        let s = serde_json::to_string(&val).unwrap();
        assert_eq!(s, r#""Running""#);
        let back: ContainerOpStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(back, val);
    }

    #[test]
    fn serde_rejects_as_str_value() {
        // Deserializing the as_str() value fails — serde expects the
        // PascalCase variant name.
        assert!(serde_json::from_str::<ConsultationStatus>(r#""waiting_human""#).is_err());
    }

    #[test]
    fn serde_rejects_unknown_variant() {
        assert!(serde_json::from_str::<ScriptLanguage>(r#""ruby""#).is_err());
        assert!(serde_json::from_str::<GoalStatus>(r#""frozen""#).is_err());
    }

    // ── PartialEq / Eq / Hash ──────────────────────────────────────

    #[test]
    fn enum_equality_and_copy() {
        let a = ContainerOpStatus::Running;
        let b = a; // Copy
        assert_eq!(a, b);
        assert_ne!(a, ContainerOpStatus::Stopped);
    }
}
