//! `StateTree` —— 单个隔离状态树实例。
//!
//! 一棵树对应一个 `ScopeKey`（workspace 全局 或 user 私有），内部是一
//! 份 `serde_json::Value` 树 + 单调递增版本号 + 最近访问时间戳 + 变更广
//! 播。所有写操作都经过 `tree.write()` —— 它在内存里 apply 之后做前后
//! diff，生成 `PatchOp` 并广播给订阅者（ws_bridge 的 state-tree writer
//! 任务订阅它，把落在视口内的 patch 发给客户端）。
//!
//! 这是「联机游戏式」状态同步的服务端核心：服务端是唯一写者，客户端无
//! 状态只 apply。懒加载 + 空闲回收由 `Registry` 负责，落库由 `Store`
//! 负责（见 store.rs）。

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use serde_json::Value;
use tokio::sync::{Mutex, RwLock, broadcast};
use tracing::trace;
use uuid::Uuid;

use crate::patch::{self, PatchOp};
use crate::snapshot;

/// 树的所有者：workspace 树 或 user 私有树。
///
/// 三级隔离：「游戏房间」式强同步——同一 `(group_id, workspace_id)` 下
/// 的所有用户共享 workspace 树，各自有独立 user 树。直接成员（未通过
/// 组授权访问）以 `user_id` 作为个人伪组 ID。
///
/// `group_id` 始终存在，绝不为 `None`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeKey {
    pub workspace_id: Uuid,
    pub group_id: Uuid,
    pub owner: ScopeOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeOwner {
    Workspace,
    User(Uuid),
}

impl ScopeKey {
    pub fn workspace(workspace_id: Uuid, group_id: Uuid) -> Self {
        Self {
            workspace_id,
            group_id,
            owner: ScopeOwner::Workspace,
        }
    }

    pub fn user(workspace_id: Uuid, user_id: Uuid, group_id: Uuid) -> Self {
        Self {
            workspace_id,
            group_id,
            owner: ScopeOwner::User(user_id),
        }
    }
}

/// 广播给订阅者的变更事件。ws_bridge 据此把 patch 发给视口命中的客户端。
#[derive(Debug, Clone)]
pub struct PatchEvent {
    pub scope: ScopeKey,
    /// 本批变更的 patch（已 apply 到树上，客户端只 apply 即可）。
    pub ops: Vec<PatchOp>,
    /// 应用这批 patch 之后的树版本（单调递增）。
    pub version: u64,
}

/// 单个隔离状态树实例。
pub struct StateTree {
    scope: ScopeKey,
    inner: RwLock<Value>,
    /// 单调递增版本号 —— 每次 write 自增。客户端可用它做去重/顺序保证。
    version: AtomicU64,
    /// 最近一次访问（读或写）的 monotonic 纳秒时间戳 —— 供 Registry 的
    /// 空闲回收 reaper 判断是否可卸载。
    last_access_ns: AtomicU64,
    /// 变更广播。容量 256，与 UpstreamConn.fanout_tx 一致；订阅者落后
    /// 会收到 Lagged（ws_bridge 端 continue 跳过，等下一次周期快照兜底）。
    events_tx: broadcast::Sender<PatchEvent>,
    /// 写锁 —— 串行化所有写操作，保证 diff 的 before/after 一致性。
    write_lock: Mutex<()>,
    /// 最近 64 条 patch 的环形缓冲区 —— 新订阅者 replay 用。
    recent_patches: Mutex<Vec<PatchEvent>>,
}

impl StateTree {
    pub fn new(scope: ScopeKey) -> Arc<Self> {
        let (events_tx, _) = broadcast::channel(256);
        Arc::new(Self {
            scope,
            inner: RwLock::new(Value::Object(serde_json::Map::new())),
            version: AtomicU64::new(0),
            last_access_ns: AtomicU64::new(now_ns()),
            events_tx,
            write_lock: Mutex::new(()),
            recent_patches: Mutex::new(Vec::with_capacity(64)),
        })
    }

    pub fn scope(&self) -> ScopeKey {
        self.scope
    }

    /// Subscribe to change broadcasts.
    pub fn subscribe_events(&self) -> broadcast::Receiver<PatchEvent> {
        self.events_tx.subscribe()
    }

    /// 最近访问时间戳（monotonic 纳秒）。
    pub fn last_access_ns(&self) -> u64 {
        self.last_access_ns.load(Ordering::Relaxed)
    }

    /// 当前活跃订阅者数（ws_bridge 的 state-tree writer 每个持一个 receiver）。
    /// reaper 用它判断树实例是否还在被跟踪 —— 0 才允许回收。
    pub fn subscriber_count(&self) -> usize {
        self.events_tx.receiver_count()
    }

    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    pub fn touch(&self) {
        self.last_access_ns.store(now_ns(), Ordering::Relaxed);
    }

    /// 读取整棵树（快照副本）。会更新 last_access。
    pub async fn read_all(&self) -> Value {
        self.touch();
        self.inner.read().await.clone()
    }

    /// 读取视口子树快照（裁剪）。会更新 last_access。
    pub async fn read_viewport(&self, viewport: &[String]) -> Value {
        self.touch();
        let guard = self.inner.read().await;
        snapshot::snapshot(&guard, viewport)
    }

    /// 写一个 patch op —— 内部先 apply 再广播。返回应用后的版本号。
    /// 通常用 `write_ops` 批量写以减少广播次数。
    pub async fn write(&self, op: PatchOp) -> u64 {
        self.write_ops(vec![op]).await
    }

    /// 批量写 —— 串行化 apply 全部 op，**直接广播传入的 ops**（保留
    /// set/replace/del 语义），而非重新 diff。
    ///
    /// 为什么不重新 diff：调用方传入的 ops 已有明确语义（如 replace 表示
    /// 整体替换），重新 diff(before, after) 会把 replace 退化成 set/del
    /// 组合，丢失语义（例如 work_status 从 {Running}→{Completed} 会被
    /// diff 成 del Running + set Completed，而非 replace work_status）。
    /// 直接广播原始 ops 让客户端 apply 时保留正确语义。
    ///
    /// 幂等检测：仍比较 before/after，若实际无变化则不自增版本、不广播
    /// （避免重复推送无意义的 op）。
    pub async fn write_ops(&self, ops: Vec<PatchOp>) -> u64 {
        if ops.is_empty() {
            return self.version();
        }
        let _guard = self.write_lock.lock().await;

        let before = {
            let g = self.inner.read().await;
            g.clone()
        };
        let mut after = before.clone();
        patch::apply_all(&mut after, &ops);

        // 幂等检测：实际无变化则跳过。
        if after == before {
            return self.version();
        }

        // 写回 after。
        {
            let mut g = self.inner.write().await;
            *g = after;
        }
        let new_version = self.version.fetch_add(1, Ordering::Relaxed) + 1;
        self.touch();

        let event = PatchEvent {
            scope: self.scope,
            ops,
            version: new_version,
        };
        let ops_count = event.ops.len();
        // 广播 —— 没有订阅者时 send 返回 Err，忽略（树仍是权威源）。
        let _ = self.events_tx.send(event.clone());
        // Record in ring buffer for late subscribers to replay.
        {
            let mut buf = self.recent_patches.lock().await;
            if buf.len() >= 64 {
                buf.remove(0);
            }
            buf.push(event);
        }
        trace!(
            scope = ?self.scope,
            version = new_version,
            ops = ops_count,
            "state tree patched"
        );
        new_version
    }

    /// 用一个完整的根值替换整棵树（用于懒加载后填充 / store 载入）。
    /// 会做 diff 广播。不做深合并 —— 调用方应确保语义正确。
    pub async fn replace_root(&self, new_root: Value) -> u64 {
        self.write_ops(vec![PatchOp::set("", new_root)]).await
    }
}

fn now_ns() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn write_and_read_roundtrip() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil(), Uuid::nil()));
        tree.write(PatchOp::set(
            "state.agents.hubris",
            json!({"status":"idle"}),
        ))
        .await;
        let all = tree.read_all().await;
        assert_eq!(
            all,
            json!({"state":{"agents":{"hubris":{"status":"idle"}}}})
        );
    }

    #[tokio::test]
    async fn write_broadcasts_patch_event() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil(), Uuid::nil()));
        let mut rx = tree.subscribe_events();
        tree.write(PatchOp::set("state.x", json!(1))).await;
        let event = rx.recv().await.expect("should receive event");
        // 空树首次写：diff 把整个新子树合并成分支级 op（set("state", {x:1})）
        // 而非叶子级（set("state.x", 1)）—— 更高效，客户端深合并后结果一致。
        assert!(!event.ops.is_empty());
        assert_eq!(event.version, 1);
        // 验证应用后状态正确（不管 op 粒度）。
        let all = tree.read_all().await;
        assert_eq!(all, json!({"state":{"x":1}}));
    }

    #[tokio::test]
    async fn idempotent_write_does_not_increment_version() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil(), Uuid::nil()));
        tree.write(PatchOp::set("state.x", json!(1))).await;
        let v1 = tree.version();
        // 再写一次相同值 —— diff 应为空，版本不变、不广播。
        tree.write(PatchOp::set("state.x", json!(1))).await;
        assert_eq!(tree.version(), v1);
    }

    #[tokio::test]
    async fn read_viewport_crops() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil(), Uuid::nil()));
        tree.write(PatchOp::set("state.agents.hubris", json!(1)))
            .await;
        tree.write(PatchOp::set("state.devices.n1", json!(2))).await;
        let snap = tree.read_viewport(&["state.agents".into()]).await;
        assert_eq!(snap, json!({"state":{"agents":{"hubris":1}}}));
    }

    #[tokio::test]
    async fn batch_write_single_broadcast() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil(), Uuid::nil()));
        let mut rx = tree.subscribe_events();
        tree.write_ops(vec![
            PatchOp::set("state.a", json!(1)),
            PatchOp::set("state.b", json!(2)),
        ])
        .await;
        let event = rx.recv().await.expect("should receive one batched event");
        // 一次广播（write_ops 合并成一次 diff + 一次广播）。op 数取决于 diff
        // 粒度（空树首次写可能合并成分支级 op）。
        assert!(!event.ops.is_empty());
        // 应用后状态正确。
        let all = tree.read_all().await;
        assert_eq!(all, json!({"state":{"a":1,"b":2}}));
    }

    #[tokio::test]
    async fn replace_root_diffs_against_empty() {
        let tree = StateTree::new(ScopeKey::workspace(Uuid::nil(), Uuid::nil()));
        let mut rx = tree.subscribe_events();
        tree.replace_root(json!({"state":{"x":1}})).await;
        let event = rx.recv().await.expect("should receive event");
        assert!(!event.ops.is_empty());
        let all = tree.read_all().await;
        assert_eq!(all, json!({"state":{"x":1}}));
    }

    #[test]
    fn scope_key_isolation() {
        let ws = ScopeKey::workspace(Uuid::nil(), Uuid::nil());
        let u = ScopeKey::user(Uuid::nil(), Uuid::nil(), Uuid::nil());
        assert_ne!(ws, u);
        assert_eq!(ws.owner, ScopeOwner::Workspace);
        assert_eq!(u.owner, ScopeOwner::User(Uuid::nil()));
    }
}
