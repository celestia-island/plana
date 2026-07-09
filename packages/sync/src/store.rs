//! 叶子持久化 —— 状态树回收落库的适配层。
//!
//! 状态树是「懒加载 + 实时回收」的：空闲超阈值且无活跃订阅者时，reaper
//! 把 dirty 叶子 flush 到 DB、从内存移除树实例，下次访问再重新载入。
//! 落库由 `StateStore` trait 抽象，便于消费方自行实现 DB 持久化。

use async_trait::async_trait;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::tree::ScopeKey;

/// 叶子级 KV 存储。键 = 路径前缀（如 `state.agents.hubris`），值 = 子树。
#[async_trait]
pub trait StateStore: Send + Sync + 'static {
    /// 载入该 scope 下所有已持久化的叶子，合并成一棵树返回。
    /// 没有数据返回空对象。
    async fn load(&self, scope: ScopeKey) -> Value;

    /// 把该 scope 下指定路径的子树写入（upsert）。
    async fn put(&self, scope: ScopeKey, path: &str, value: Value);

    /// 删除该 scope 下指定路径的叶子。
    async fn delete(&self, scope: ScopeKey, path: &str);

    /// 删除该 scope 下所有数据（树实例被整体回收时调用）。
    async fn clear(&self, scope: ScopeKey);
}

/// 进程内 no-op 存储 —— 骨架阶段用，落库总是丢失（重启即空）。
#[derive(Default, Clone)]
pub struct NoopStore;

#[async_trait]
impl StateStore for NoopStore {
    async fn load(&self, _scope: ScopeKey) -> Value {
        Value::Object(serde_json::Map::new())
    }
    async fn put(&self, _scope: ScopeKey, _path: &str, _value: Value) {}
    async fn delete(&self, _scope: ScopeKey, _path: &str) {}
    async fn clear(&self, _scope: ScopeKey) {}
}

/// 简单内存 KV —— 单测可验证 flush/load 往返。生产不用。
#[derive(Default, Clone)]
pub struct MemoryStore {
    inner: Arc<Mutex<HashMap<(ScopeKey, String), Value>>>,
}

#[async_trait]
impl StateStore for MemoryStore {
    async fn load(&self, scope: ScopeKey) -> Value {
        let map = self.inner.lock().unwrap();
        let mut out = serde_json::Map::new();
        for ((s, path), v) in map.iter() {
            if *s == scope {
                insert_path(&mut out, path, v.clone());
            }
        }
        Value::Object(out)
    }
    async fn put(&self, scope: ScopeKey, path: &str, value: Value) {
        self.inner
            .lock()
            .unwrap()
            .insert((scope, path.to_string()), value);
    }
    async fn delete(&self, scope: ScopeKey, path: &str) {
        self.inner
            .lock()
            .unwrap()
            .remove(&(scope, path.to_string()));
    }
    async fn clear(&self, scope: ScopeKey) {
        self.inner.lock().unwrap().retain(|(s, _), _| *s != scope);
    }
}

/// 生成用于 store 的 scope 标识字符串（用于 DB 行的 scope_id）。
/// workspace 全局树 = workspace_id；user 树 = `user:<user_id>@<ws_id>`。
pub fn scope_id_str(scope: ScopeKey) -> String {
    match scope.owner {
        crate::tree::ScopeOwner::Workspace => scope.workspace_id.to_string(),
        crate::tree::ScopeOwner::User(uid) => {
            format!("user:{}@{}", uid, scope.workspace_id)
        }
    }
}

fn insert_path(out: &mut serde_json::Map<String, Value>, path: &str, value: Value) {
    if path.is_empty() {
        if let Value::Object(map) = value {
            for (k, v) in map {
                out.insert(k, v);
            }
        }
        return;
    }
    let segments: Vec<&str> = path.split('.').collect();
    let mut current = out;
    for (i, seg) in segments.iter().enumerate() {
        if i == segments.len() - 1 {
            current.insert(seg.to_string(), value.clone());
        } else {
            let entry = current
                .entry(seg.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(map) = entry {
                current = map;
            } else {
                return;
            }
        }
    }
}
