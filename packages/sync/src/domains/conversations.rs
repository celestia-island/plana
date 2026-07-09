//! conversations 域 —— chat 会话列表（user 私有）。
//!
//! chat conversations 是 per-user 的（每个 user 有自己的会话列表），所以
//! 放 user 私有树 `state.user.conversations.<id>`。服务端 CRUD（chat.*
//! conversation 系列 RPC）是 shittim-chest 本地处理，每个成功分支后 upsert/
//! del 进该 user 的树，订阅了 `state.user.conversations` 的客户端自动收到
//! 增量 patch + 周期全量快照。
//!
//! 与 agents/devices 同构：完整对象列表 upsert，键 = conversation id。

use serde_json::Value;
use uuid::Uuid;

use crate::{PatchOp, ScopeKey, StateTree};

/// 把 conversation 列表 upsert 进 user 树 `state.user.conversations.<id>`。
pub async fn upsert_conversations(tree: &StateTree, convs: &[Value]) {
    let ops: Vec<PatchOp> = convs
        .iter()
        .filter_map(|c| {
            let id = c.get("id").and_then(|v| v.as_str())?.to_string();
            Some(PatchOp::set(
                format!("state.user.conversations.{id}"),
                c.clone(),
            ))
        })
        .collect();
    if !ops.is_empty() {
        tree.write_ops(ops).await;
    }
}

/// upsert 单个 conversation。
pub async fn upsert_conversation(tree: &StateTree, id: &str, conv: Value) {
    tree.write(PatchOp::set(format!("state.user.conversations.{id}"), conv))
        .await;
}

/// 删除一个 conversation。
pub async fn remove_conversation(tree: &StateTree, id: &str) {
    tree.write(PatchOp::del(format!("state.user.conversations.{id}")))
        .await;
}

/// user 私有 scope（conversations 专属）。
pub fn user_scope(workspace_id: Uuid, user_id: Uuid) -> ScopeKey {
    ScopeKey::user(workspace_id, user_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn upsert_places_conversations() {
        let tree = StateTree::new(ScopeKey::user(Uuid::nil(), Uuid::nil()));
        upsert_conversations(
            &tree,
            &[
                json!({"id":"c1","title":"对话1","mode":"reports"}),
                json!({"id":"c2","title":"对话2","mode":"nodes"}),
            ],
        )
        .await;
        let all = tree.read_all().await;
        assert_eq!(
            all["state"]["user"]["conversations"]["c1"]["title"],
            json!("对话1")
        );
        assert_eq!(
            all["state"]["user"]["conversations"]["c2"]["mode"],
            json!("nodes")
        );
    }

    #[tokio::test]
    async fn upsert_single_and_remove() {
        let tree = StateTree::new(ScopeKey::user(Uuid::nil(), Uuid::nil()));
        upsert_conversation(&tree, "c1", json!({"id":"c1","title":"x"})).await;
        assert!(tree.read_all().await["state"]["user"]["conversations"]["c1"].is_object());
        remove_conversation(&tree, "c1").await;
        assert!(
            tree.read_all().await["state"]["user"]["conversations"]
                .as_object()
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn upsert_skips_no_id() {
        let tree = StateTree::new(ScopeKey::user(Uuid::nil(), Uuid::nil()));
        upsert_conversations(&tree, &[json!({"title":"no id"})]).await;
        let all = tree.read_all().await;
        let c = all["state"]["user"]["conversations"].as_object();
        assert!(c.is_none() || c.unwrap().is_empty());
    }
}
