//! todo 域 —— 状态树第三个 workspace-scoped 域。
//!
//! 把 `todo.list_tree` 的响应（递归树结构 `{ tree: TodoTreeNode[] }`）
//! 以整体 replace 方式写进 `state.todo`。客户端声明视口 `state.todo`
//! 即可收到全量快照 + 增量 patch，不再需要重连时发 `todo.list_tree`。
//!
//! 存 `state.todo`（整体 replace，非逐节点）。因为 todo 是递归树结构，
//! 逐节点 upsert 需要服务端复制客户端递归合并逻辑；整体 replace 更简单。

use serde_json::Value;

use crate::{PatchOp, ScopeKey, StateTree};

/// 把 todo 树整体写进指定 scope 的 `state.todo`。
///
/// `tree_nodes` 是完整的递归树数组（来自 `todo.list_tree` 响应）。
/// 若数组为空则删除 `state.todo` 键。
pub async fn upsert_todo(tree: &StateTree, tree_nodes: &[Value]) {
    if tree_nodes.is_empty() {
        tree.write(PatchOp::del("state.todo")).await;
    } else {
        tree.write(PatchOp::replace(
            "state.todo",
            Value::Array(tree_nodes.to_vec()),
        ))
        .await;
    }
}

/// 删除整个 todo 树（dev 工具用，或 workspace 清空时）。
pub async fn remove_todo(tree: &StateTree) {
    tree.write(PatchOp::del("state.todo")).await;
}

/// 首次访问该 scope 的树时的懒载入。todo 数据由 scepter 推送填充，
/// mock-mode 下从 mock roster 预置，不留空树。
pub async fn load_initial(_registry: &crate::StateTreeRegistry, scope: ScopeKey) {
    let _ = scope;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn upsert_writes_tree_to_state_todo() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_todo(
            &tree,
            &[
                json!({"id":"t1","title":"Task 1","status":"pending","depth":0,"tags":[],"children":[]}),
                json!({"id":"t2","title":"Task 2","status":"done","depth":0,"tags":[],"children":[]}),
            ],
        )
        .await;
        let all = tree.read_all().await;
        let todo_nodes = all["state"]["todo"].as_array().unwrap();
        assert_eq!(todo_nodes.len(), 2);
        assert_eq!(todo_nodes[0]["id"], "t1");
    }

    #[tokio::test]
    async fn upsert_replaces_entire_tree() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_todo(&tree, &[json!({"id":"t1","title":"old"})]).await;
        upsert_todo(&tree, &[json!({"id":"t2","title":"new"})]).await;
        let all = tree.read_all().await;
        let todo_nodes = all["state"]["todo"].as_array().unwrap();
        assert_eq!(todo_nodes.len(), 1);
        assert_eq!(todo_nodes[0]["id"], "t2");
    }

    #[tokio::test]
    async fn upsert_empty_removes_todo_key() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_todo(&tree, &[json!({"id":"t1"})]).await;
        upsert_todo(&tree, &[]).await;
        let all = tree.read_all().await;
        assert!(all["state"]["todo"].is_null());
    }

    #[tokio::test]
    async fn viewport_snapshot_returns_only_todo() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_todo(&tree, &[json!({"id":"t1"})]).await;
        tree.write(PatchOp::set(
            "state.agents.hubris",
            json!({"status":"idle"}),
        ))
        .await;
        let snap = tree.read_viewport(&["state.todo".into()]).await;
        assert_eq!(snap, json!({"state":{"todo":[{"id":"t1"}]}}));
    }

    #[tokio::test]
    async fn remove_todo_deletes_key() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_todo(&tree, &[json!({"id":"t1"})]).await;
        remove_todo(&tree).await;
        let all = tree.read_all().await;
        assert!(all["state"]["todo"].is_null());
    }

    #[tokio::test]
    async fn upsert_handles_nested_tree() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_todo(
            &tree,
            &[json!({
                "id":"root",
                "title":"Root",
                "status":"pending",
                "depth":0,
                "tags":[],
                "children":[{
                    "id":"child",
                    "title":"Child",
                    "status":"pending",
                    "depth":1,
                    "tags":[],
                    "children":[]
                }]
            })],
        )
        .await;
        let all = tree.read_all().await;
        let root = &all["state"]["todo"].as_array().unwrap()[0];
        assert_eq!(root["children"].as_array().unwrap().len(), 1);
    }
}
