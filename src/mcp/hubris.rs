use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct ReportResult {
    pub summary: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoTreeNode {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub children: Vec<TodoTreeNode>,
    pub depth: usize,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub tags: Vec<String>,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct CreateTodoParams {
    pub title: String,
    pub workspace_id: Option<String>,
    pub user_id: Option<String>,
    pub parent_id: Option<Uuid>,
    pub description: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
    pub claimed_by: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct ListTodoParams {
    pub workspace_id: Option<String>,
    pub parent_id: Option<Uuid>,
    pub status: Option<String>,
    pub tree: Option<bool>,
    #[serde(default)]
    pub view: Option<String>,
}

impl ListTodoParams {
    pub fn normalize(&mut self) {
        if self.tree.is_none() {
            if let Some(ref v) = self.view {
                match v.as_str() {
                    "tree" | "true" => self.tree = Some(true),
                    _ => self.tree = Some(false),
                }
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct UpdateTodoParams {
    pub todo_id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub claimed_by: Option<String>,
    pub parent_id: Option<Uuid>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct DeleteTodoParams {
    pub todo_id: Uuid,
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct ClearTodoParams {
    pub workspace_id: Option<String>,
    pub dry_run: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct MoveTodoParams {
    pub todo_id: Uuid,
    pub new_parent_id: Option<Uuid>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct ReportParams {
    pub text: Option<String>,
    pub summary: Option<String>,
    pub body: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct ReportHumanParams {
    pub summary: String,
    pub body: Option<String>,
    pub text: Option<String>,
    pub mode: Option<String>,
    pub content: Option<String>,
}

// ── Tool result structs ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoCreateResult {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub parent_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoListItem {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub parent_id: Option<Uuid>,
    pub claimed_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoListResult {
    pub total: usize,
    pub items: Vec<TodoListItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoTreeListResult {
    pub total: usize,
    pub tree: Vec<TodoTreeNode>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoUpdateResult {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoDeleteResult {
    pub deleted_id: Uuid,
    pub success: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoClearItem {
    pub id: Uuid,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoClearDryRunResult {
    pub dry_run: bool,
    pub would_delete: usize,
    pub items: Vec<TodoClearItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoClearResult {
    pub deleted_count: u64,
    pub success: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/hubris.ts")]
pub struct TodoMoveResult {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub updated_at: String,
}
