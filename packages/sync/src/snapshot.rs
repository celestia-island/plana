//! 视口快照计算 —— yuuka 兼容重导出层。
//!
//! 实现已迁至 yuuka（`celestia-island/yuuka` 0.9+ 的 viewport 模块），
//! 语义与迁移前的本仓实现逐字节等价。本模块只做名称保持，下游
//! `plana_sync::snapshot::*` 的使用路径零变化：
//!
//! | plana 侧名称        | yuuka 来源                       |
//! |---------------------|----------------------------------|
//! | `path_in_viewport`  | `yuuka::viewport::path_in_viewport` |
//! | `snapshot`          | `yuuka::viewport::snapshot`      |
//!
//! 视口模型不变：客户端声明「我在看哪些路径前缀」（如 `state.agents`），
//! 服务端据此裁剪出全局状态树中属于这些前缀的子树，作为
//! `Sync.StateSnapshot` 推送（首帧 + 周期兜底双用途）—— 详见 yuuka
//! `viewport` 模块的文档。
//!
//! 下方行为级单测全部保留（只调 pub API），经重导出路径运行 —— 它们
//! 是重导出正确性的保险。迁移前的测试本就只走 pub 表面（无引用
//! `normalize_prefixes` 等私有 helper 的测试需要删除），yuuka 侧另有
//! 等价与边界加强覆盖。

pub use yuuka::viewport::{path_in_viewport, snapshot};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn prefix_match_basic() {
        let vp = vec!["state.agents".to_string()];
        assert!(path_in_viewport("state.agents.hubris", &vp));
        assert!(path_in_viewport("state.agents", &vp));
        assert!(!path_in_viewport("state.devices", &vp));
    }

    #[test]
    fn prefix_match_ancestor_op_hits_descendant_viewport() {
        // 关键：服务端 diff 可能产出分支级 op（如 set("state", {agents:{...}})，
        // 当整棵子树是新的时）。客户端订阅了子路径 state.agents.hubris，
        // 这个分支级 op 覆盖了视口关心的那部分 → 必须命中（双向匹配）。
        let vp = vec!["state.agents.hubris".to_string()];
        assert!(
            path_in_viewport("state", &vp),
            "ancestor op (state) must hit descendant viewport (state.agents.hubris)"
        );
        assert!(path_in_viewport("state.agents", &vp));
        assert!(path_in_viewport("state.agents.hubris", &vp));
        assert!(path_in_viewport("state.agents.hubris.status", &vp));
        // 完全不相干的路径仍不命中。
        assert!(!path_in_viewport("devices.n1", &vp));
    }

    #[test]
    fn prefix_match_root_matches_all() {
        let vp = vec!["".to_string()];
        assert!(path_in_viewport("anything.here", &vp));
    }

    #[test]
    fn empty_viewport_matches_nothing() {
        let vp: Vec<String> = vec![];
        assert!(!path_in_viewport("state.agents", &vp));
    }

    #[test]
    fn snapshot_crops_subtree() {
        let root = json!({
            "state": {
                "agents": {"hubris": {"status": "idle"}},
                "devices": {"node1": {"online": true}}
            }
        });
        let snap = snapshot(&root, &["state.agents".to_string()]);
        assert_eq!(
            snap,
            json!({"state": {"agents": {"hubris": {"status": "idle"}}}})
        );
        // devices 不在视口内，不应出现。
        assert!(snap.get("devices").is_none() || snap.get("state.devices").is_none());
    }

    #[test]
    fn snapshot_multiple_prefixes() {
        let root = json!({
            "state": {
                "agents": {"hubris": 1},
                "devices": {"n1": 2},
                "reports": {"r1": 3}
            }
        });
        let snap = snapshot(&root, &["state.agents".into(), "state.devices".into()]);
        assert_eq!(
            snap,
            json!({"state": {"agents": {"hubris": 1}, "devices": {"n1": 2}}})
        );
    }

    #[test]
    fn snapshot_normalizes_nested_prefixes() {
        let root = json!({"state": {"agents": {"a": 1}, "devices": {"d": 2}}});
        // state.agents 被 state 包含 → 只保留 state。
        let snap = snapshot(&root, &["state".into(), "state.agents".into()]);
        assert_eq!(
            snap,
            json!({"state": {"agents": {"a": 1}, "devices": {"d": 2}}})
        );
    }

    #[test]
    fn snapshot_empty_viewport_returns_empty() {
        let root = json!({"state": {"a": 1}});
        let snap = snapshot(&root, &[]);
        assert!(snap.as_object().map(|m| m.is_empty()).unwrap_or(true));
    }

    #[test]
    fn snapshot_root_prefix_returns_whole_tree() {
        let root = json!({"state": {"a": 1}, "meta": {"b": 2}});
        let snap = snapshot(&root, &["".to_string()]);
        assert_eq!(snap, root);
    }
}
