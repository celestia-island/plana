//! 视口快照计算 —— 路径前缀订阅 + 子树裁剪。
//!
//! 客户端声明"我在看哪些路径前缀"（如 `state.agents`），服务端据此裁剪出
//! 全局状态树中属于这些前缀的子树，作为 `Tui.StateSnapshot` 推送。这是
//! "屏幕里看得到的内容"的服务端实现：视口 = 路径前缀集合，与具体列表滚
//! 动位置无关（刻意保持简单、稳定、可预测 —— 大列表的窗口化是后续按域
//! 复用 chunk_history 的虚拟 total 模式的事，不在这层）。
//!
//! 快照的两种用途：
//! 1. **首帧**：客户端 `state.subscribe` 时立即返回当前视口 snapshot
//!    （替代旧的 fetchInitialData 拉取）。
//! 2. **周期兜底**：ws_bridge 每 ~3s 对每个活跃视口推一次完整 snapshot，
//!    防丢补漏、最终一致（即便增量 patch 丢失也能自愈）。

use serde_json::{Map, Value};

use crate::patch::split_path;

/// 判断 `path` 是否落在任一视口前缀之内（双向匹配）。
///
/// 一个 op 落在视口内，当且仅当它和视口前缀之一满足下列之一：
/// - **op 是视口前缀的子代（含相等）**：op=`state.agents.hubris` 命中
///   前缀 `state.agents` 或 `state`。这是常见情形（叶子级增量）。
/// - **op 是视口前缀的祖先**：op=`state`（整体子树 set）命中前缀
///   `state.x`、`state.agents.hubris` —— 因为这个 set 覆盖了视口关心的
///   那部分。这种情况发生在服务端 diff 把整棵新子树合并成一个分支级
///   op 时（比拆成叶子 op 高效），客户端必须收到才能 apply。
///
/// 空字符串前缀视为「根」—— 匹配所有路径。
pub fn path_in_viewport(path: &str, viewport: &[String]) -> bool {
    if viewport.is_empty() {
        return false;
    }
    let segs = split_path(path);
    viewport.iter().any(|p| overlaps(p, &segs))
}

/// `prefix` 与 `path_segs` 是否有覆盖关系（一个是另一个的前缀）。
fn overlaps(prefix: &str, path_segs: &[&str]) -> bool {
    let prefix_segs = split_path(prefix);
    if prefix_segs.is_empty() {
        return true; // 空前缀 = 根 = 匹配一切。
    }
    let n = prefix_segs.len().min(path_segs.len());
    // 较短的前若干段必须一致（双向：谁短谁就是潜在前缀）。
    path_segs[..n]
        .iter()
        .zip(prefix_segs[..n].iter())
        .all(|(p, pre)| p == pre)
}

/// 单向前缀判断：`prefix` 是否是 `path_segs` 的前缀（prefix 更短或等长）。
/// 供 `normalize_prefixes` 用 —— 判断一个前缀是否被另一个（更靠根的）包含。
fn is_prefix(prefix: &str, path_segs: &[&str]) -> bool {
    let prefix_segs = split_path(prefix);
    if prefix_segs.is_empty() {
        return true;
    }
    if prefix_segs.len() > path_segs.len() {
        return false;
    }
    path_segs[..prefix_segs.len()]
        .iter()
        .zip(prefix_segs.iter())
        .all(|(p, pre)| p == pre)
}

/// 从 `root` 中裁剪出 `viewport` 覆盖的子树。
///
/// 返回一个对象，键为各前缀的最后一段（或前缀本身若是多段且不冲突），
/// 值为对应子树。实现上对每个前缀下降取值后合并进结果对象。当多个前缀
/// 有父子关系（如 `state` 和 `state.agents`），较短前缀的结果会包含较
/// 长前缀，故按前缀长度降序处理，让较短前缀先写入、较长的覆盖更精确的
/// 子键 —— 但这会产生重复。为简单起见，本实现用「规范化前缀」：去掉
/// 被其它前缀包含的子前缀（`state.agents` 被 `state` 包含 → 只保留
/// `state`），保证结果对象不重复。
pub fn snapshot(root: &Value, viewport: &[String]) -> Value {
    if viewport.is_empty() {
        return Value::Object(Map::new());
    }
    let normalized = normalize_prefixes(viewport);
    let mut out = Map::new();
    for p in &normalized {
        let sub = descend_const(root, p);
        // 把子树挂到结果对象上：前缀的完整路径都重建出来。
        insert_at_path(&mut out, p, sub);
    }
    Value::Object(out)
}

/// 去掉被其它前缀包含的子前缀（避免重复裁剪）。输入无需有序。
///
/// 例：`["state.agents", "state"]` → `["state"]`（后者包含前者）。
fn normalize_prefixes(prefixes: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for p in prefixes {
        let p_segs = split_path(p);
        // p 被已保留的某个前缀包含 → 跳过。
        let dominated = kept.iter().any(|k| is_prefix(k, &p_segs));
        if dominated {
            continue;
        }
        // 移除已保留但被 p 包含的（p 更短/更靠根）。
        let p_clone = p.clone();
        kept.retain(|k| {
            let k_segs = split_path(k);
            !is_prefix(&p_clone, &k_segs)
        });
        kept.push(p_clone);
    }
    kept
}

/// 沿点分路径在 `root` 中下降取值（只读）。缺失段返回 Null。
fn descend_const(root: &Value, path: &str) -> Value {
    let segs = split_path(path);
    let mut cur = root;
    for seg in segs {
        match cur {
            Value::Object(map) => match map.get(seg) {
                Some(v) => cur = v,
                None => return Value::Null,
            },
            _ => return Value::Null,
        }
    }
    cur.clone()
}

/// 在 `obj` 里沿 `path` 创建对象链，叶子上放 `value`。
fn insert_at_path(obj: &mut Map<String, Value>, path: &str, value: Value) {
    let segs = split_path(path);
    if segs.is_empty() {
        // 根路径：与 value 深合并（value 应是对象）。
        if let Value::Object(v) = value {
            for (k, vv) in v {
                obj.insert(k, vv);
            }
        }
        return;
    }
    let mut cur = obj;
    let (last_segs, leaf) = segs.split_at(segs.len() - 1);
    for seg in last_segs {
        let entry = cur
            .entry((*seg).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        cur = entry.as_object_mut().expect("just promoted");
    }
    cur.insert(leaf[0].to_string(), value);
}

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
        assert_eq!(snap, json!({"state": {"agents": {"a": 1}, "devices": {"d": 2}}}));
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
