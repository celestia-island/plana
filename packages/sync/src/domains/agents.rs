//! agents 域 —— 状态树试点域。
//!
//! 把 agent 列表（mock roster 或 scepter 的 Tui.AgentListResponse /
//! AgentUpdate）upsert 进 `state.agents.<agent_id>`。客户端声明视口
//! `state.agents` 即可收到这些 agent 的增量 patch + 周期全量快照，不再
//! 需要重连时发 `Tui.ListAgents`。
//!
//! 这是「联机游戏式」同步的第一个落地点，验证整条链路（树 → 广播 →
//! 视口过滤 → patch/snapshot → 客户端 apply）打通。其余 6 个旧域
//! （devices/chat/todo/reports/logs/preferences）暂不动、双轨开关可回退。

use serde_json::Value;
use uuid::Uuid;

use crate::{PatchOp, ScopeKey, StateTree, StateTreeRegistry};

/// 把一个 agent 列表 upsert 进指定 scope 的树 `state.agents.<agent_id>`。
///
/// 每个 agent 的 `agent_id`（缺失则退 `agent_type`）作为键。已存在的同
/// 键 agent 会被深合并覆盖。调用方通常是 `process_upstream_text`
/// （收到 Tui.AgentListResponse / AgentUpdate 时）或 Loader。
pub async fn upsert_agents(tree: &StateTree, agents: &[Value]) {
    let ops: Vec<PatchOp> = agents
        .iter()
        .filter_map(|a| {
            let id = agent_key(a)?;
            Some(PatchOp::set(format!("state.agents.{id}"), a.clone()))
        })
        .collect();
    if !ops.is_empty() {
        tree.write_ops(ops).await;
    }
}

/// 删除一个 agent（收到 AgentUpdate 且 status 暗示下线时用，预留）。
pub async fn remove_agent(tree: &StateTree, agent_id: &str) {
    tree.write(PatchOp::del(format!("state.agents.{agent_id}"))).await;
}

/// 把 entelecheia 的字段级增量 `AgentPatch` 数组 upsert 进树。
///
/// entelecheia 的 `TuiMessage::AgentPatch { patches: Vec<AgentPatch> }`
/// 序列化后 jsonrpc method = `Tui.AgentPatch`，params = `{"patches":[...]}`
/// （action 字段被 infra_jsonrpc 剥掉）。每个 patch 有 `agent_id` +
/// 若干可选字段（work_status / current_model / cpu_usage / ...）—— 是
/// 字段级增量，不是完整 agent 对象。
///
/// 这里把每个非 null 字段 set 到 `state.agents.<agent_id>.<field>`。
/// `agent_id` 缺失的 patch 用 `agent_number` 或 `agent_type` 退化作键。
pub async fn upsert_agent_patches(tree: &StateTree, patches: &[Value]) {
    let mut ops: Vec<PatchOp> = Vec::new();
    for p in patches {
        let Some(id) = patch_agent_key(p) else {
            continue;
        };
        // 保留 agent_id 自身作为键下的字段（客户端用它定位），其余非 null
        // 字段逐个 replace（字段级增量：整个字段值是新值，不深合并 ——
        // 否则 work_status 这种 tagged-union 会错误地合并 {Running}+{Completed}）。
        for (field, val) in p.as_object().into_iter().flatten() {
            // agent_id / agent_number / agent_type 已经用作键，但仍写入值
            // （客户端要能读到）。跳过 null（entelecheia 的 Option 字段在
            // 缺省时序列化可能省略或为 null）。
            if val.is_null() {
                continue;
            }
            ops.push(PatchOp::replace(
                format!("state.agents.{id}.{field}"),
                val.clone(),
            ));
        }
    }
    if !ops.is_empty() {
        tree.write_ops(ops).await;
    }
}

/// 取字段级 patch 的键（优先 agent_id，退 agent_number，再退 agent_type）。
/// 与完整 agent 的 agent_key 区分：patch 里 agent_id 可能是 agent_number
/// 形态（entelecheia 的 panel_agent_id）。
fn patch_agent_key(p: &Value) -> Option<String> {
    p.get("agent_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            p.get("agent_number")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            p.get("agent_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

/// 取一个 agent 条目的键（优先 agent_id，退 agent_type）。
fn agent_key(a: &Value) -> Option<String> {
    a.get("agent_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            a.get("agent_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

/// 首次访问该 scope 的树时的懒载入钩子。
///
/// 消费方可自行实现初始数据载入（如从数据库或 mock roster）。
/// 默认实现为 no-op（agent 由后续 Tui.AgentListResponse 填充）。
pub async fn load_initial(_registry: &StateTreeRegistry, _scope: ScopeKey, _workspace_id: Uuid) {
    // No-op by default. Consumers can override this behavior.
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn upsert_places_agents_under_state_agents() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_agents(
            &tree,
            &[
                json!({"agent_id":"hubris-1","agent_type":"HubRis","status":"Online"}),
                json!({"agent_id":"kalos-1","agent_type":"KaLos","status":"Idle"}),
            ],
        )
        .await;
        let all = tree.read_all().await;
        assert_eq!(
            all,
            json!({
                "state": {
                    "agents": {
                        "hubris-1": {"agent_id":"hubris-1","agent_type":"HubRis","status":"Online"},
                        "kalos-1": {"agent_id":"kalos-1","agent_type":"KaLos","status":"Idle"},
                    }
                }
            })
        );
    }

    #[tokio::test]
    async fn upsert_deep_merges_existing_agent() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_agents(
            &tree,
            &[json!({"agent_id":"hubris-1","status":"Online","model":"glm"})],
        )
        .await;
        // 部分更新 —— model 应保留，status 覆盖。
        upsert_agents(
            &tree,
            &[json!({"agent_id":"hubris-1","status":"Busy"})],
        )
        .await;
        let all = tree.read_all().await;
        assert_eq!(
            all["state"]["agents"]["hubris-1"],
            json!({"agent_id":"hubris-1","status":"Busy","model":"glm"})
        );
    }

    #[tokio::test]
    async fn upsert_skips_entries_without_id() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_agents(&tree, &[json!({"no_id":true}), json!({"agent_type":"Fallback"})]).await;
        let all = tree.read_all().await;
        // 第二个用 agent_type 作键。
        assert_eq!(all["state"]["agents"]["Fallback"]["agent_type"], "Fallback");
    }

    #[tokio::test]
    async fn viewport_snapshot_returns_only_agents() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_agents(&tree, &[json!({"agent_id":"hubris-1","status":"Online"})]).await;
        // 另写一个非 agents 的键。
        tree.write(PatchOp::set("state.devices.n1", json!({"online":true})))
            .await;
        let snap = tree.read_viewport(&["state.agents".into()]).await;
        assert_eq!(
            snap,
            json!({"state":{"agents":{"hubris-1":{"agent_id":"hubris-1","status":"Online"}}}})
        );
    }

    #[tokio::test]
    async fn remove_agent_deletes_key() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_agents(&tree, &[json!({"agent_id":"hubris-1"})]).await;
        remove_agent(&tree, "hubris-1").await;
        let all = tree.read_all().await;
        assert!(all["state"]["agents"].as_object().unwrap().is_empty());
    }

    #[tokio::test]
    async fn upsert_patches_field_level_incremental() {
        // 模拟 entelecheia 的 Tui.AgentPatch：字段级增量。
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        // 先建一个 agent。
        upsert_agents(
            &tree,
            &[json!({"agent_id":"hubris-1","agent_type":"HubRis","work_status":{"Running":{}}})],
        )
        .await;
        // 收到一个字段级 patch（只改 work_status + current_model，null 跳过）。
        upsert_agent_patches(
            &tree,
            &[json!({
                "agent_id": "hubris-1",
                "agent_type": "HubRis",
                "version": 2,
                "work_status": {"Completed": {}},
                "current_model": "glm-5",
                "cpu_usage": null,
            })],
        )
        .await;
        let all = tree.read_all().await;
        let agent = &all["state"]["agents"]["hubris-1"];
        // work_status 被覆盖，current_model 新增，agent_type 保留（patch 里也带，
        // 但同值），version 写入。cpu_usage 为 null 不写入。
        assert_eq!(agent["work_status"], json!({"Completed": {}}));
        assert_eq!(agent["current_model"], json!("glm-5"));
        assert_eq!(agent["version"], json!(2));
        assert_eq!(agent["agent_type"], json!("HubRis"));
        assert!(agent.get("cpu_usage").is_none(), "null 字段不应写入");
    }

    #[tokio::test]
    async fn upsert_patches_skips_no_id() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_agent_patches(&tree, &[json!({"work_status":"x"})]).await;
        let all = tree.read_all().await;
        // 没有 agent_id 的 patch 被跳过 —— agents 子树应为空或不存在。
        let agents = all["state"]["agents"].as_object();
        assert!(agents.is_none() || agents.unwrap().is_empty());
    }

    #[tokio::test]
    async fn upsert_patches_fallback_to_agent_number() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil()));
        upsert_agent_patches(
            &tree,
            &[json!({"agent_number":"001","work_status":{"Idle":{}}})],
        )
        .await;
        let all = tree.read_all().await;
        let agent = &all["state"]["agents"]["001"];
        assert_eq!(agent["work_status"], json!({"Idle": {}}));
    }
}
