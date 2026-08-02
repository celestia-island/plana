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
        if self.tree.is_none()
            && let Some(ref v) = self.view
        {
            match v.as_str() {
                "tree" | "true" => self.tree = Some(true),
                _ => self.tree = Some(false),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn report_result_round_trip() {
        let r = ReportResult {
            summary: "task done".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v, json!({"summary": "task done"}));
        let back: ReportResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.summary, "task done");
    }

    #[test]
    fn todo_tree_node_nested_round_trip() {
        let child = TodoTreeNode {
            id: Uuid::new_v4(),
            title: "child".into(),
            status: "pending".into(),
            children: vec![],
            depth: 1,
            description: Some("a child".into()),
            priority: Some("high".into()),
            tags: vec!["bug".into()],
        };
        let parent = TodoTreeNode {
            id: Uuid::new_v4(),
            title: "parent".into(),
            status: "in_progress".into(),
            children: vec![child.clone()],
            depth: 0,
            description: None,
            priority: None,
            tags: vec![],
        };
        let v = serde_json::to_value(&parent).unwrap();
        assert_eq!(v["children"].as_array().unwrap().len(), 1);
        assert_eq!(v["children"][0]["title"], "child");
        let back: TodoTreeNode = serde_json::from_value(v).unwrap();
        assert_eq!(back.children.len(), 1);
        assert_eq!(back.children[0].id, child.id);
    }

    #[test]
    fn create_todo_params_optional_fields() {
        let p = CreateTodoParams {
            title: "new task".into(),
            workspace_id: None,
            user_id: None,
            parent_id: None,
            description: None,
            metadata: None,
            claimed_by: None,
        };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["title"], "new task");
        assert!(
            v.get("description").is_some(),
            "Option fields present as null"
        );
    }

    #[test]
    fn list_todo_params_normalize_view_tree() {
        let mut p = ListTodoParams {
            workspace_id: None,
            parent_id: None,
            status: None,
            tree: None,
            view: Some("tree".into()),
        };
        p.normalize();
        assert_eq!(p.tree, Some(true));
    }

    #[test]
    fn list_todo_params_normalize_view_true() {
        let mut p = ListTodoParams {
            workspace_id: None,
            parent_id: None,
            status: None,
            tree: None,
            view: Some("true".into()),
        };
        p.normalize();
        assert_eq!(p.tree, Some(true));
    }

    #[test]
    fn list_todo_params_normalize_view_other() {
        let mut p = ListTodoParams {
            workspace_id: None,
            parent_id: None,
            status: None,
            tree: None,
            view: Some("list".into()),
        };
        p.normalize();
        assert_eq!(p.tree, Some(false));
    }

    #[test]
    fn list_todo_params_normalize_no_view() {
        let mut p = ListTodoParams {
            workspace_id: None,
            parent_id: None,
            status: None,
            tree: None,
            view: None,
        };
        p.normalize();
        assert_eq!(p.tree, None);
    }

    #[test]
    fn list_todo_params_normalize_preserves_existing_tree() {
        let mut p = ListTodoParams {
            workspace_id: None,
            parent_id: None,
            status: None,
            tree: Some(false),
            view: Some("tree".into()),
        };
        p.normalize();
        // Existing tree value takes precedence.
        assert_eq!(p.tree, Some(false));
    }

    #[test]
    fn todo_clear_dry_run_result_round_trip() {
        let r = TodoClearDryRunResult {
            dry_run: true,
            would_delete: 3,
            items: vec![TodoClearItem {
                id: Uuid::new_v4(),
                title: "t".into(),
                status: "done".into(),
            }],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["dry_run"], true);
        assert_eq!(v["would_delete"], 3);
        assert_eq!(v["items"].as_array().unwrap().len(), 1);
        let back: TodoClearDryRunResult = serde_json::from_value(v).unwrap();
        assert!(back.dry_run);
        assert_eq!(back.would_delete, 3);
    }

    #[test]
    fn todo_delete_result_round_trip() {
        let id = Uuid::new_v4();
        let r = TodoDeleteResult {
            deleted_id: id,
            success: true,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["deleted_id"], id.to_string());
        assert_eq!(v["success"], true);
    }

    #[test]
    fn report_params_all_optional() {
        let p = ReportParams {
            text: None,
            summary: None,
            body: None,
            content: None,
        };
        let v = serde_json::to_value(&p).unwrap();
        // All fields present as null (no skip_serializing_if).
        assert_eq!(v.as_object().unwrap().len(), 4);
    }
}
