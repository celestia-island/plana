//! `StateTreeRegistry` —— 按 scope 隔离的树实例表，懒加载 + 空闲回收。
//!
//! 仿照 `report_cache` / `chunk_history` / `containers::ContainerSlotRegistry`
//! 的既有模式（`DashMap<ScopeKey, Arc<...>>`），但多了两件事：
//!
//! 1. **懒加载**：`get_or_load(scope)` 未命中时，从 `StateStore` 载入初
//!    始状态填充树实例，再返回。载入是 per-scope 串行化的（init lock，
//!    仿 `UpstreamPool::acquire` 的 `init_locks`），避免并发首次访问重
//!    复载入。
//! 2. **实时回收**：`spawn_reaper` 后台任务周期扫描 `last_access_ns`，
//!    超过 `idle_ttl` 且当前无订阅者的树实例，把内存树 flush 到 store
//!    后从表里移除（"部分旧数据会在合适的时候从内存回收并确保它已写进
//!    数据库存储住"）。下次访问会重新懒加载。
//!
//! 订阅者计数由 `StateTree` 内部 broadcast sender 的 `receiver_count()`
//! 反映 —— ws_bridge 的 state-tree writer 持有一个 receiver，断开即 drop。

use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use tokio::sync::Mutex;
use tracing::info;

use crate::store::StateStore;
use crate::tree::{ScopeKey, StateTree};

/// 默认空闲回收阈值：30 分钟无访问且无订阅者 → 回收。
const DEFAULT_IDLE_TTL: Duration = Duration::from_secs(30 * 60);
/// 默认 reaper 扫描周期：5 分钟一次。
const DEFAULT_REAP_INTERVAL: Duration = Duration::from_secs(5 * 60);

#[derive(Clone)]
pub struct StateTreeRegistry {
    trees: Arc<DashMap<ScopeKey, Arc<StateTree>>>,
    init_locks: Arc<DashMap<ScopeKey, Arc<Mutex<()>>>>,
    store: Arc<dyn StateStore>,
    /// 软上限：内存中同时驻留的树实例数。超出时 reaper 会更激进地回收。
    /// 0 = 不限（仅按 idle_ttl 回收）。骨架阶段默认不限。
    soft_capacity: usize,
}

impl StateTreeRegistry {
    pub fn new(store: Arc<dyn StateStore>) -> Self {
        Self {
            trees: Arc::new(DashMap::new()),
            init_locks: Arc::new(DashMap::new()),
            store,
            soft_capacity: 0,
        }
    }

    pub fn with_soft_capacity(mut self, cap: usize) -> Self {
        self.soft_capacity = cap;
        self
    }

    /// 已驻留的树实例数（监控/测试用）。
    pub fn len(&self) -> usize {
        self.trees.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trees.is_empty()
    }

    /// 取（或懒加载）一棵树。命中即返回并 touch；未命中则从 store 载入
    /// 后返回。per-scope 串行化初始化（仿 UpstreamPool::acquire）。
    pub async fn get_or_load(&self, scope: ScopeKey) -> Arc<StateTree> {
        // Fast path。
        if let Some(t) = self.trees.get(&scope) {
            return t.clone();
        }
        // Slow path：per-scope 串行。
        let lock = self
            .init_locks
            .entry(scope)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        let _guard = lock.lock().await;
        // Double-check。
        if let Some(t) = self.trees.get(&scope) {
            return t.clone();
        }
        let tree = StateTree::new(scope);
        // 从 store 载入初始状态（若有）。
        let loaded = self.store.load(scope).await;
        if !loaded.as_object().map(|m| m.is_empty()).unwrap_or(true) {
            tree.replace_root(loaded).await;
        }
        self.trees.insert(scope, tree.clone());
        drop(_guard);
        self.try_cleanup_init_lock(&scope);
        tree
    }

    /// 已驻留树实例的快照（reaper 用）。
    fn snapshot_keys(&self) -> Vec<ScopeKey> {
        self.trees.iter().map(|e| *e.key()).collect()
    }

    /// 当 init_lock 无竞争时移除条目，防止 DashMap 无限增长
    /// （仿 UpstreamPool::try_cleanup_init_lock）。
    fn try_cleanup_init_lock(&self, scope: &ScopeKey) {
        if let Some(lock) = self.init_locks.get(scope) {
            if lock.try_lock().is_ok() {
                drop(lock);
                self.init_locks.remove(scope);
            }
        }
    }

    /// 启动后台空闲回收任务。返回 join handle（通常 fire-and-forget）。
    ///
    /// 回收条件：`last_access` 距今 > idle_ttl **且** broadcast 无订阅者
    /// （`receiver_count() == 0`，说明没有 ws_bridge writer 正在跟踪它）。
    /// 回收时先把整棵树 flush 到 store（`replace_root` 的逆操作），再从
    /// 表里移除。下次 `get_or_load` 会重新从 store 载入。
    pub fn spawn_reaper(self, idle_ttl: Option<Duration>) -> tokio::task::JoinHandle<()> {
        let idle_ttl = idle_ttl.unwrap_or(DEFAULT_IDLE_TTL);
        let interval = DEFAULT_REAP_INTERVAL;
        let me = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                me.reap_once(idle_ttl).await;
            }
        })
    }

    async fn reap_once(&self, idle_ttl: Duration) {
        let now = now_ns();
        let keys = self.snapshot_keys();
        let mut reaped = 0usize;
        for scope in keys {
            let Some(tree) = self.trees.get(&scope) else {
                continue;
            };
            let idle_ns = now.saturating_sub(tree.last_access_ns());
            // 仍有订阅者 → 不回收（活跃连接在跟踪它）。
            if tree.subscriber_count() > 0 {
                continue;
            }
            if Duration::from_nanos(idle_ns) < idle_ttl {
                continue;
            }
            // flush 整棵树到 store 后移除。
            let snapshot = tree.read_all().await;
            drop(tree); // 释放 dashmap 读锁引用再 remove。
            if !snapshot.as_object().map(|m| m.is_empty()).unwrap_or(true) {
                self.store.put(scope, "", snapshot).await;
            }
            self.trees.remove(&scope);
            reaped += 1;
        }
        if reaped > 0 {
            info!(
                reaped,
                remaining = self.trees.len(),
                "state tree reaper evicted idle trees"
            );
        }
    }

    /// 手动驱逐一棵树（测试 / 显式卸载用）。不 flush —— 调用方负责。
    pub fn evict(&self, scope: &ScopeKey) {
        self.trees.remove(scope);
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
    use crate::patch::PatchOp;
    use crate::store;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn get_or_load_creates_then_reuses() {
        let reg = StateTreeRegistry::new(Arc::new(store::NoopStore));
        let scope = ScopeKey::workspace(Uuid::nil());
        let t1 = reg.get_or_load(scope).await;
        let t2 = reg.get_or_load(scope).await;
        assert!(Arc::ptr_eq(&t1, &t2), "second get should reuse same tree");
        assert_eq!(reg.len(), 1);
    }

    #[tokio::test]
    async fn lazy_load_from_store() {
        // 预置 store 里有数据 —— get_or_load 应载入。
        let mem = store::MemoryStore::default();
        let scope = ScopeKey::workspace(Uuid::nil());
        mem.put(scope, "state.x", json!(1)).await;
        let reg = StateTreeRegistry::new(Arc::new(mem));
        let tree = reg.get_or_load(scope).await;
        let all = tree.read_all().await;
        assert_eq!(all, json!({"state":{"x":1}}));
    }

    #[tokio::test]
    async fn reaper_evicts_idle_unsubscribed_tree() {
        let mem = store::MemoryStore::default();
        let reg = StateTreeRegistry::new(Arc::new(mem.clone()));
        let scope = ScopeKey::workspace(Uuid::nil());
        let tree = reg.get_or_load(scope).await;
        tree.write(PatchOp::set("state.x", json!(1))).await;
        assert_eq!(reg.len(), 1);

        // ttl=0 + 无订阅者 → 立即回收。注意：这里没有持有 receiver。
        reg.reap_once(Duration::from_secs(0)).await;
        assert_eq!(reg.len(), 0, "idle tree should be reaped");

        // 被回收的数据应已落 store。
        let reloaded = mem.load(scope).await;
        assert_eq!(reloaded, json!({"state":{"x":1}}));
    }

    #[tokio::test]
    async fn reaper_keeps_tree_with_subscriber() {
        let reg = StateTreeRegistry::new(Arc::new(store::NoopStore));
        let scope = ScopeKey::workspace(Uuid::nil());
        let tree = reg.get_or_load(scope).await;
        // 持有一个 receiver —— 模拟 ws_bridge writer 在订阅。
        let _rx = tree.subscribe_events();
        reg.reap_once(Duration::from_secs(0)).await;
        assert_eq!(
            reg.len(),
            1,
            "tree with active subscriber must not be reaped"
        );
    }

    #[tokio::test]
    async fn reaper_does_not_evict_recently_accessed() {
        let reg = StateTreeRegistry::new(Arc::new(store::NoopStore));
        let scope = ScopeKey::workspace(Uuid::nil());
        let tree = reg.get_or_load(scope).await;
        tree.write(PatchOp::set("state.x", json!(1))).await;
        // ttl 很大 —— 刚访问过的树不会被回收。
        reg.reap_once(Duration::from_secs(3600)).await;
        assert_eq!(reg.len(), 1);
    }
}
