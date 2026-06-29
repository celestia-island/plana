//! File Browsing — TuiMessage variant params.
//!
//! Browse/read files inside a container (#demiurge / #NNN), on a host machine,
//! or in a workspace checkout. Targets are distinguished by `FileTargetKind`.
//! The node-list container cards, the Bridge Network host cards and the
//! workspace cards all open this same file browser.
//!
//! Mirrors `entelecheia/.../tui_types/message/types/mod.rs`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Which filesystem a file operation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/fileBrowsing.ts")]
#[serde(rename_all = "snake_case")]
pub enum FileTargetKind {
    /// A container slot — `#demiurge` or `#NNN` (id is the slot badge).
    Container,
    /// A host machine (id is the host_id / device id; "localhost" for self).
    Host,
    /// A workspace checkout (id is the workspace_id).
    Workspace,
}

/// A file-operation target — a (kind, id) pair plus optional workspace
/// context (container slots are per-workspace).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/fileBrowsing.ts")]
pub struct FileTarget {
    pub kind: FileTargetKind,
    /// Container badge (`#demiurge` / `#001`), host id, or workspace id.
    pub id: String,
    /// Owning workspace id (container slots are workspace-scoped).
    #[serde(default)]
    #[ts(optional, type = "string")]
    pub workspace_id: Option<uuid::Uuid>,
}

/// One entry in a directory listing.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/fileBrowsing.ts")]
pub struct FileTreeEntry {
    pub name: String,
    /// `"file"` | `"dir"` | `"symlink"`.
    pub kind: String,
    pub size: u64,
}

/// `Tui.RequestFileTree` — list one level of a directory.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/fileBrowsing.ts")]
pub struct RequestFileTreeParams {
    pub target: FileTarget,
    /// Sub-path under the target root (empty/`""` = root).
    #[serde(default)]
    pub path: String,
}

/// `Tui.FileTree` — directory listing response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/fileBrowsing.ts")]
pub struct FileTreeParams {
    pub target: FileTarget,
    pub path: String,
    pub entries: Vec<FileTreeEntry>,
}

/// `Tui.RequestFileRead` — read a single (text) file, capped server-side.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/fileBrowsing.ts")]
pub struct RequestFileReadParams {
    pub target: FileTarget,
    pub path: String,
}

/// `Tui.FileRead` — file-content response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/fileBrowsing.ts")]
pub struct FileReadParams {
    pub target: FileTarget,
    pub path: String,
    pub content: String,
    pub size: u64,
    /// `true` when content was truncated to the server read cap.
    #[serde(default)]
    pub truncated: bool,
}
