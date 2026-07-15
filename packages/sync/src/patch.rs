//! JSON-Merge-Patch 合并 + diff 生成（状态树增量同步的核心）。
//!
//! 状态树是「服务端唯一写者」模型：所有变更都由服务端在树实例上发起，
//! 然后对前后两版做前缀 diff 生成一组 `PatchOp`，广播给订阅了对应视口的
//! 客户端。客户端只负责 apply，不需要处理并发冲突。
//!
//! 合并语义采用 RFC 7396（JSON Merge Patch）的变体：
//! - `set` 深合并 —— 对象键逐项合并（同键以新值覆盖），非对象直接替换。
//! - `del` —— 删除指定路径的键。
//!
//! 这比完整 RFC 6902 JSON-Patch（add/remove/replace/move/copy/test）简单得
//! 多，且本场景无并发写者、无重命名需求，复杂度收益不对等。增量之外，
//! `ws_bridge` 还会周期性对每个活跃视口推一次全量快照作为兜底（防丢补
//! 漏、最终一致），所以即便 patch 丢失也能自愈。

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// 单个 patch 操作。`path` 用点分隔（`state.agents.hubris`）。
///
/// 序列化后即 `Tui.StatePatch` 通知的 params 形态。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchOp {
    pub op: PatchKind,
    pub path: String,
    /// `set` 时为要写入的值；`del` 时恒为 None。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchKind {
    /// 深合并（RFC 7396）：对象逐键合并，标量/数组直接覆盖。用于「增量
    /// 合并」语义，如完整 agent 对象的部分字段更新。
    Set,
    /// 直接替换（不深合并）：整个值覆盖旧值。用于枚举/tagged-union 等需
    /// 要整体替换的场景（如 work_status 从 {Running:{}} → {Completed:{}}）。
    Replace,
    Del,
}

impl PatchOp {
    pub fn set(path: impl Into<String>, value: Value) -> Self {
        Self {
            op: PatchKind::Set,
            path: path.into(),
            value: Some(value),
        }
    }

    pub fn replace(path: impl Into<String>, value: Value) -> Self {
        Self {
            op: PatchKind::Replace,
            path: path.into(),
            value: Some(value),
        }
    }

    pub fn del(path: impl Into<String>) -> Self {
        Self {
            op: PatchKind::Del,
            path: path.into(),
            value: None,
        }
    }
}

/// 把点分路径拆成段。`state.agents.hubris` → `["state","agents","hubris"]`。
///
/// 空字符串返回空切片（表示「根」）。不支持转义 —— 状态树的键约定不含点
/// （agent_id / 路径都是 UUID / 已知的标识符）。
pub fn split_path(path: &str) -> Vec<&str> {
    if path.is_empty() {
        Vec::new()
    } else {
        path.split('.').collect()
    }
}

/// 在 `root` 上应用单个 `PatchOp`（就地修改）。
///
/// - `set` 深合并：沿路径创建/进入对象，叶子处用 RFC 7396 merge。
/// - `replace` 直接替换：叶子处的值整体替换（不深合并），用于枚举/
///   tagged-union 等需要整体替换的场景。
/// - `del`：沿路径进入，删除最后一个键。
///
/// 空路径的 `set` 视为「用 merge_patch 替换根」；空路径的 `replace`
/// 直接用新值替换根。
pub fn apply(root: &mut Value, op: &PatchOp) {
    let segments = split_path(&op.path);
    match op.op {
        PatchKind::Set => {
            let Some(new_val) = op.value.clone() else {
                return;
            };
            if segments.is_empty() {
                *root = merge_patch(std::mem::take(root), new_val);
                return;
            }
            let target = descend_mut(root, &segments);
            *target = merge_patch(std::mem::take(target), new_val);
        }
        PatchKind::Replace => {
            let Some(new_val) = op.value.clone() else {
                return;
            };
            if segments.is_empty() {
                *root = new_val;
                return;
            }
            let target = descend_mut(root, &segments);
            *target = new_val;
        }
        PatchKind::Del => {
            if segments.is_empty() {
                // 删根 = 清空（保持一个空对象，而不是 Value::Null，
                // 否则后续路径创建会失败）。
                *root = Value::Object(Map::new());
                return;
            }
            let (parent_segs, leaf) = segments.split_at(segments.len() - 1);
            let parent = descend_mut(root, parent_segs);
            if let Value::Object(map) = parent {
                map.remove(leaf[0]);
            }
        }
    }
}

/// 批量 apply（按顺序）。用于客户端收到一组 patch 时的合并执行。
pub fn apply_all(root: &mut Value, ops: &[PatchOp]) {
    for op in ops {
        apply(root, op);
    }
}

/// RFC 7396 merge patch 合并：`target` 是当前值，`patch` 是新值。
///
/// - 都是对象 → 逐键合并（patch 中 null 表示删除 target 的键）。
/// - 否则 → patch 覆盖 target。
pub fn merge_patch(target: Value, patch: Value) -> Value {
    match (target, patch) {
        (Value::Object(mut t), Value::Object(p)) => {
            for (k, v) in p {
                match v {
                    // null = 删除该键（RFC 7396 语义）。
                    Value::Null => {
                        t.remove(&k);
                    }
                    pv => {
                        let merged = match t.remove(&k) {
                            Some(tv) => merge_patch(tv, pv.clone()),
                            None => pv,
                        };
                        t.insert(k, merged);
                    }
                }
            }
            Value::Object(t)
        }
        // 任一非对象 → patch 直接覆盖（包括 patch=Null 的情况，表示删除）。
        (_, p) => p,
    }
}

/// 对前后两版状态做 diff，生成一组 `PatchOp`。
///
/// 采用「对象递归 diff + 叶子 set/del」策略：
/// - 两边都是对象 → 对并集逐键递归；新增/变更的键生成 set，消失的键生成 del。
/// - 值不同（至少一边非对象）→ 生成 set（前缀 + 当前路径）。
/// - 值相同 → 不生成任何 op。
///
/// 生成的 op 路径都以 `prefix` 开头（通常是 `state`）。
pub fn diff(prefix: &str, before: &Value, after: &Value) -> Vec<PatchOp> {
    let mut ops = Vec::new();
    diff_into(prefix, before, after, &mut ops);
    ops
}

fn diff_into(prefix: &str, before: &Value, after: &Value, ops: &mut Vec<PatchOp>) {
    match (before, after) {
        (Value::Object(b), Value::Object(a)) => {
            // 处理删除的键。
            for (k, bv) in b {
                if !a.contains_key(k) {
                    ops.push(PatchOp::del(join(prefix, k)));
                    continue;
                }
                // 相同的键递归比较。
                let _ = bv; // 仅用于遍历；比较在下面。
            }
            // 处理新增/变更的键。
            for (k, av) in a {
                let path = join(prefix, k);
                match b.get(k) {
                    None => ops.push(PatchOp::set(path, av.clone())),
                    Some(bv) => diff_into(&path, bv, av, ops),
                }
            }
        }
        (b, a) if b == a => {
            // 值完全相同 —— 不产生 op。
        }
        (_, a) => {
            // 值不同（至少一边非对象）→ set。
            ops.push(PatchOp::set(prefix.to_string(), a.clone()));
        }
    }
}

/// 沿路径段下降，沿途缺失的对象键自动创建（空对象）。永远返回叶子处的
/// `&mut Value`（根/中间节点若是非对象，会被替换成空对象继续下降）。
fn descend_mut<'a>(root: &'a mut Value, segments: &[&str]) -> &'a mut Value {
    let mut cur = root;
    for seg in segments {
        // 当前节点若不是对象，先变成空对象（否则无法继续 descend）。
        if !cur.is_object() {
            *cur = Value::Object(Map::new());
        }
        let map = cur.as_object_mut().expect("just promoted to object");
        cur = map
            .entry((*seg).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
    }
    cur
}

fn join(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

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
