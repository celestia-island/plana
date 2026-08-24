//! JSON-Merge-Patch 合并 + diff 生成 —— yuuka 兼容重导出层。
//!
//! 实现已迁至 yuuka（`celestia-island/yuuka` 0.9+ 的 patch / merge /
//! diff / path 模块），语义与迁移前的本仓实现逐字节等价（yuuka 侧
//! 72000+36000 项差分实证零分歧），serde wire 表示也逐字节一致
//! （`Sync.StatePatch` 线上格式由 yuuka `tests/patch_compat.rs` 与本
//! crate `tests/wire_compat.rs` 的 golden 测试双向锁定）。
//!
//! 本模块只做名称保持，下游 `plana_sync::patch::*` 的使用路径零变化：
//!
//! | plana 侧名称  | yuuka 来源                  |
//! |---------------|-----------------------------|
//! | `PatchOp`     | `yuuka::patch::PatchOp`     |
//! | `PatchKind`   | `yuuka::patch::PatchKind`   |
//! | `apply`       | `yuuka::patch::apply`       |
//! | `apply_all`   | `yuuka::patch::apply_all`   |
//! | `merge_patch` | `yuuka::merge::merge_patch` |
//! | `diff`        | `yuuka::diff::diff`         |
//! | `split_path`  | `yuuka::path::split`        |
//!
//! 合并语义仍是 RFC 7396（JSON Merge Patch）变体：`set` 深合并、
//! `replace` 整体替换、`del` 删键 —— 详见 yuuka 对应模块的文档。
//!
//! 下方行为级单测全部保留（只调 pub API），经重导出路径运行 —— 它们
//! 是重导出正确性的保险。迁移前的测试本就只走 pub 表面（无引用私有
//! helper 的测试需要删除），yuuka 侧另有等价与加强覆盖。

pub use yuuka::diff::diff;
pub use yuuka::merge::merge_patch;
pub use yuuka::patch::{PatchKind, PatchOp, apply, apply_all};
pub use yuuka::path::split as split_path;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn set_creates_nested_objects() {
        let mut root = json!({});
        apply(
            &mut root,
            &PatchOp::set("state.agents.hubris", json!({"status":"idle"})),
        );
        assert_eq!(
            root,
            json!({"state":{"agents":{"hubris":{"status":"idle"}}}})
        );
    }

    #[test]
    fn set_deep_merges_objects() {
        let mut root = json!({"state":{"agents":{"hubris":{"status":"idle","model":"gpt"}}}});
        apply(
            &mut root,
            &PatchOp::set("state.agents.hubris", json!({"status":"busy"})),
        );
        // model 键保留（深合并），status 被覆盖。
        assert_eq!(
            root,
            json!({"state":{"agents":{"hubris":{"status":"busy","model":"gpt"}}}})
        );
    }

    #[test]
    fn del_removes_key() {
        let mut root = json!({"state":{"agents":{"hubris":{},"kalos":{}}}});
        apply(&mut root, &PatchOp::del("state.agents.kalos"));
        assert_eq!(root, json!({"state":{"agents":{"hubris":{}}}}));
    }

    #[test]
    fn replace_overrides_without_deep_merge() {
        // replace 用于枚举/tagged-union：整体替换，不深合并。
        let mut root = json!({"state":{"agents":{"hubris":{"work_status":{"Running":{}}}}}});
        apply(
            &mut root,
            &PatchOp::replace("state.agents.hubris.work_status", json!({"Completed": {}})),
        );
        // 整体替换 —— 不应残留 Running。
        assert_eq!(
            root,
            json!({"state":{"agents":{"hubris":{"work_status":{"Completed":{}}}}}})
        );
    }

    #[test]
    fn replace_creates_path() {
        let mut root = json!({});
        apply(
            &mut root,
            &PatchOp::replace("state.agents.hubris", json!({"x": 1})),
        );
        assert_eq!(root, json!({"state":{"agents":{"hubris":{"x":1}}}}));
    }

    #[test]
    fn merge_patch_object_recursive() {
        let t = json!({"a":{"x":1,"y":2},"b":3});
        let p = json!({"a":{"y":20,"z":3},"c":4});
        assert_eq!(
            merge_patch(t, p),
            json!({"a":{"x":1,"y":20,"z":3},"b":3,"c":4})
        );
    }

    #[test]
    fn merge_patch_null_deletes_key() {
        let t = json!({"a":{"x":1,"y":2}});
        let p = json!({"a":{"x":null}});
        assert_eq!(merge_patch(t, p), json!({"a":{"y":2}}));
    }

    #[test]
    fn diff_detects_add_change_remove() {
        let before = json!({"hubris":{"status":"idle"},"kalos":{"status":"busy"}});
        let after = json!({"hubris":{"status":"busy","model":"glm"},"seia":{"status":"idle"}});
        let ops = diff("state.agents", &before, &after);
        // kalos 被删、hubris.status 变 + hubris.model 增、seia 新增。
        let has_del_kalos = ops
            .iter()
            .any(|o| o.op == PatchKind::Del && o.path == "state.agents.kalos");
        let has_set_hubris_status = ops.iter().any(|o| {
            o.op == PatchKind::Set
                && o.path == "state.agents.hubris.status"
                && o.value == Some(json!("busy"))
        });
        let has_set_hubris_model = ops.iter().any(|o| {
            o.op == PatchKind::Set
                && o.path == "state.agents.hubris.model"
                && o.value == Some(json!("glm"))
        });
        let has_set_seia = ops.iter().any(|o| {
            o.op == PatchKind::Set
                && o.path == "state.agents.seia"
                && o.value == Some(json!({"status":"idle"}))
        });
        assert!(has_del_kalos, "missing del kalos: {ops:?}");
        assert!(has_set_hubris_status, "missing set hubris.status: {ops:?}");
        assert!(has_set_hubris_model, "missing set hubris.model: {ops:?}");
        assert!(has_set_seia, "missing set seia: {ops:?}");
    }

    #[test]
    fn diff_identical_emits_nothing() {
        let v = json!({"a":{"b":1}});
        assert!(diff("state", &v, &v).is_empty());
    }

    #[test]
    fn apply_then_diff_roundtrip() {
        // before --apply(ops)--> after；diff(before, after) 应产生能重建
        // after 的 op 集（再 apply 回 before 应得到 after）。
        let before = json!({"agents":{"hubris":{"status":"idle"}}});
        let mut root = before.clone();
        apply_all(
            &mut root,
            &[
                PatchOp::set("agents.hubris.status", json!("busy")),
                PatchOp::set("agents.seia", json!({"status":"idle"})),
            ],
        );
        let after = root.clone();
        let ops = diff("", &before, &after);
        let mut rebuilt = before;
        apply_all(&mut rebuilt, &ops);
        assert_eq!(rebuilt, after);
    }
}
