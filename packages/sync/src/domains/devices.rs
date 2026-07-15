//! devices 域 —— 状态树第二个域（验证迁移模式可复制）。
//!
//! 把 PolemosDeviceList（mock/空 或 scepter 推的 `Tui.PolemosDeviceList`）
//! upsert 进 `state.devices.<node_id>`。客户端声明视口 `state.devices` 即可
//! 收到增量 patch + 周期全量快照，不再需要重连时发 `Tui.ListPolemosDevices`。
//!
//! 与 agents 域同构：完整对象列表 upsert，键 = `node_id`。device 没有字段级
//! 增量格式（不像 entelecheia 的 AgentPatch），所以只提供 `upsert_devices`。

use serde_json::Value;

use crate::{PatchOp, ScopeKey, StateTree};

/// 把 device 列表 upsert 进指定 scope 的树 `state.devices.<node_id>`。
///
/// 每个 device 的 `node_id`（字符串形态的 UUID）作为键。已存在的同键
/// device 会被深合并覆盖。调用方通常是 `process_upstream_text`
/// （收到 Tui.PolemosDeviceList 时）。
pub async fn upsert_devices(tree: &StateTree, devices: &[Value]) {
    let ops: Vec<PatchOp> = devices
        .iter()
        .filter_map(|d| {
            let id = device_key(d)?;
            Some(PatchOp::set(format!("state.devices.{id}"), d.clone()))
        })
        .collect();
    if !ops.is_empty() {
        tree.write_ops(ops).await;
    }
}

/// 删除一个 device（节点下线时用）。
pub async fn remove_device(tree: &StateTree, node_id: &str) {
    tree.write(PatchOp::del(format!("state.devices.{node_id}")))
        .await;
}

/// 取一个 device 条目的键（node_id）。PolemosDeviceInfo.node_id 是 Uuid，
/// serde 序列化成字符串。缺失/非字符串返回 None（跳过）。
fn device_key(d: &Value) -> Option<String> {
    d.get("node_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// 首次访问该 scope 的树时的懒载入（预留）。device 在 mock-mode 下为空
/// （mock 的 ListPolemosDevices 返回 []），非 mock 由 scepter 的
/// Tui.PolemosDeviceList 推送填充，所以这里不需要预置 roster。
pub async fn load_initial(_registry: &crate::StateTreeRegistry, scope: ScopeKey) {
    let _ = scope;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn upsert_places_devices_under_state_devices() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_devices(
            &tree,
            &[
                json!({"node_id":"node-1","name":"worker-a","status":"online"}),
                json!({"node_id":"node-2","name":"worker-b","status":"offline"}),
            ],
        )
        .await;
        let all = tree.read_all().await;
        assert_eq!(
            all,
            json!({
                "state": {
                    "devices": {
                        "node-1": {"node_id":"node-1","name":"worker-a","status":"online"},
                        "node-2": {"node_id":"node-2","name":"worker-b","status":"offline"},
                    }
                }
            })
        );
    }

    #[tokio::test]
    async fn upsert_deep_merges_existing_device() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_devices(
            &tree,
            &[json!({"node_id":"node-1","name":"worker-a","status":"online"})],
        )
        .await;
        // 部分更新 —— name 保留，status 覆盖。
        upsert_devices(&tree, &[json!({"node_id":"node-1","status":"busy"})]).await;
        let all = tree.read_all().await;
        assert_eq!(
            all["state"]["devices"]["node-1"],
            json!({"node_id":"node-1","name":"worker-a","status":"busy"})
        );
    }

    #[tokio::test]
    async fn upsert_skips_entries_without_node_id() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_devices(&tree, &[json!({"no_id":true}), json!({"name":"x"})]).await;
        let all = tree.read_all().await;
        let devices = all["state"]["devices"].as_object();
        assert!(devices.is_none() || devices.unwrap().is_empty());
    }

    #[tokio::test]
    async fn viewport_snapshot_returns_only_devices() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_devices(&tree, &[json!({"node_id":"node-1","status":"online"})]).await;
        // 另写一个非 devices 的键。
        tree.write(PatchOp::set(
            "state.agents.hubris",
            json!({"status":"idle"}),
        ))
        .await;
        let snap = tree.read_viewport(&["state.devices".into()]).await;
        assert_eq!(
            snap,
            json!({"state":{"devices":{"node-1":{"node_id":"node-1","status":"online"}}}})
        );
    }

    #[tokio::test]
    async fn remove_device_deletes_key() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_devices(&tree, &[json!({"node_id":"node-1"})]).await;
        remove_device(&tree, "node-1").await;
        let all = tree.read_all().await;
        assert!(all["state"]["devices"].as_object().unwrap().is_empty());
    }
}
