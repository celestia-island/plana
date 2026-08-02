//! In-memory world-state store: single-writer entities + relations with
//! monotonic versions and a broadcast channel.
//!
//! This is the typed counterpart of `plana_sync`'s `StateTree` (which stores
//! untyped `serde_json` for UI state): writes are serialised, every mutation
//! bumps a monotonic version, and a `tokio::broadcast` channel notifies
//! subscribers — the online-game-style pattern, aimed at physical-world
//! consistency instead of UI.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::broadcast;

use crate::types::{EntityId, EntityKind, WorldEntity, WorldRelation};

/// One store mutation, broadcast to subscribers.
#[derive(Debug, Clone, PartialEq)]
pub enum WorldChange {
    /// An entity was inserted or replaced.
    Upsert {
        /// Entity id.
        id: EntityId,
        /// New store version.
        version: u64,
    },
    /// An entity was removed.
    Remove {
        /// Entity id.
        id: EntityId,
        /// New store version.
        version: u64,
    },
    /// A relation was added.
    Relate {
        /// The relation.
        relation: WorldRelation,
        /// New store version.
        version: u64,
    },
    /// A relation was removed.
    Unrelate {
        /// The relation.
        relation: WorldRelation,
        /// New store version.
        version: u64,
    },
}

impl WorldChange {
    /// Store version after this change.
    pub fn version(&self) -> u64 {
        match self {
            Self::Upsert { version, .. }
            | Self::Remove { version, .. }
            | Self::Relate { version, .. }
            | Self::Unrelate { version, .. } => *version,
        }
    }
}

/// Append-only seam for persisting the world-state change stream (root PLAN
/// §8.9). The store itself is in-memory only by design; implementations may
/// persist to disk, a database, or forward into a pipeline. Appends are
/// best-effort: failures are logged and never block mutations.
pub trait WorldEventLog: Send + Sync {
    /// Append one applied change.
    fn append(&self, change: &WorldChange) -> anyhow::Result<()>;
}

/// In-memory world-state store.
///
/// All mutating calls are single-writer (interior locks) and idempotent:
/// upserting an entity identical to the stored one neither bumps the version
/// nor broadcasts a change.
///
/// Notes on the broadcast channel: it is a `tokio::broadcast`, so a
/// subscriber that falls more than `capacity` changes behind receives
/// `Lagged` and should re-read [`WorldStateStore::snapshot`]. Version
/// ordering of the broadcast stream is only guaranteed for a single writer
/// thread (there is no global write serialisation beyond the per-map locks).
pub struct WorldStateStore {
    entities: RwLock<HashMap<EntityId, WorldEntity>>,
    relations: RwLock<HashSet<WorldRelation>>,
    version: AtomicU64,
    changes_tx: broadcast::Sender<WorldChange>,
    event_log: RwLock<Option<Arc<dyn WorldEventLog>>>,
}

impl WorldStateStore {
    /// Create an empty store whose broadcast channel holds `capacity` changes.
    pub fn new(capacity: usize) -> Self {
        let (changes_tx, _) = broadcast::channel(capacity);
        Self {
            entities: RwLock::new(HashMap::new()),
            relations: RwLock::new(HashSet::new()),
            version: AtomicU64::new(0),
            changes_tx,
            event_log: RwLock::new(None),
        }
    }

    /// Attach (or detach) an append-only event log for the change stream.
    pub fn set_event_log(&self, log: Option<Arc<dyn WorldEventLog>>) {
        *self
            .event_log
            .write()
            .expect("world-state event-log lock poisoned") = log;
    }

    fn record(&self, change: &WorldChange) {
        let log = self
            .event_log
            .read()
            .expect("world-state event-log lock poisoned")
            .clone();
        if let Some(log) = log
            && let Err(e) = log.append(change)
        {
            tracing::warn!(error = %e, "world-state event log append failed");
        }
    }

    /// Current store version (number of applied changes).
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }

    /// Subscribe to the change stream.
    pub fn subscribe(&self) -> broadcast::Receiver<WorldChange> {
        self.changes_tx.subscribe()
    }

    fn bump(&self) -> u64 {
        self.version.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Insert or replace an entity. Returns `true` when the stored state
    /// actually changed (idempotent upserts are silent).
    pub fn upsert(&self, entity: WorldEntity) -> bool {
        let id = entity.id.clone();
        let changed = {
            let mut entities = self
                .entities
                .write()
                .expect("world-state entities lock poisoned");
            match entities.get(&id) {
                Some(existing) if *existing == entity => false,
                _ => {
                    entities.insert(id.clone(), entity);
                    true
                }
            }
        };
        if changed {
            let version = self.bump();
            let change = WorldChange::Upsert { id, version };
            // A send only fails when no subscribers exist — fine.
            let _ = self.changes_tx.send(change.clone());
            self.record(&change);
        }
        changed
    }

    /// Remove an entity. Returns `true` when it existed.
    pub fn remove(&self, id: &EntityId) -> bool {
        let removed = {
            let mut entities = self
                .entities
                .write()
                .expect("world-state entities lock poisoned");
            entities.remove(id).is_some()
        };
        if removed {
            let version = self.bump();
            let change = WorldChange::Remove {
                id: id.clone(),
                version,
            };
            let _ = self.changes_tx.send(change.clone());
            self.record(&change);
        }
        removed
    }

    /// Add a relation. Returns `true` when it was new.
    pub fn relate(&self, relation: WorldRelation) -> bool {
        let added = {
            let mut relations = self
                .relations
                .write()
                .expect("world-state relations lock poisoned");
            relations.insert(relation.clone())
        };
        if added {
            let version = self.bump();
            let change = WorldChange::Relate { relation, version };
            let _ = self.changes_tx.send(change.clone());
            self.record(&change);
        }
        added
    }

    /// Remove a relation. Returns `true` when it existed.
    pub fn unrelate(&self, relation: &WorldRelation) -> bool {
        let removed = {
            let mut relations = self
                .relations
                .write()
                .expect("world-state relations lock poisoned");
            relations.remove(relation)
        };
        if removed {
            let version = self.bump();
            let change = WorldChange::Unrelate {
                relation: relation.clone(),
                version,
            };
            let _ = self.changes_tx.send(change.clone());
            self.record(&change);
        }
        removed
    }

    /// Fetch one entity.
    pub fn get(&self, id: &EntityId) -> Option<WorldEntity> {
        self.entities
            .read()
            .expect("world-state entities lock poisoned")
            .get(id)
            .cloned()
    }

    /// List entities of a kind, sorted by id for determinism.
    pub fn list_by_kind(&self, kind: &EntityKind) -> Vec<WorldEntity> {
        let mut out: Vec<WorldEntity> = self
            .entities
            .read()
            .expect("world-state entities lock poisoned")
            .values()
            .filter(|e| &e.kind == kind)
            .cloned()
            .collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    /// All relations originating at `from`, sorted for determinism.
    pub fn relations_from(&self, from: &EntityId) -> Vec<WorldRelation> {
        let mut out: Vec<WorldRelation> = self
            .relations
            .read()
            .expect("world-state relations lock poisoned")
            .iter()
            .filter(|r| &r.from == from)
            .cloned()
            .collect();
        out.sort_by(|a, b| a.to.cmp(&b.to).then(a.kind.cmp(&b.kind)));
        out
    }

    /// Snapshot all entities, sorted by id.
    pub fn snapshot(&self) -> Vec<WorldEntity> {
        let mut out: Vec<WorldEntity> = self
            .entities
            .read()
            .expect("world-state entities lock poisoned")
            .values()
            .cloned()
            .collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }
}

impl Default for WorldStateStore {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AttributeValue, EntityId, EntityKind, Quality};

    fn entity(id: &str, kind: EntityKind, value: f64) -> WorldEntity {
        let mut attributes = std::collections::BTreeMap::new();
        attributes.insert("v".to_string(), AttributeValue::Number(value));
        WorldEntity {
            id: EntityId::new(id),
            kind,
            attributes,
            quality: Quality::Good,
            updated_wall: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            updated_mono_ns: None,
        }
    }

    #[test]
    fn upsert_inserts_and_broadcasts() {
        let store = WorldStateStore::new(16);
        let mut rx = store.subscribe();

        assert!(store.upsert(entity("station.19", EntityKind::Station, 0.0)));
        assert_eq!(store.version(), 1);

        let change = rx.try_recv().unwrap();
        match change {
            WorldChange::Upsert { id, version } => {
                assert_eq!(id, EntityId::new("station.19"));
                assert_eq!(version, 1);
            }
            other => panic!("expected Upsert, got {other:?}"),
        }
    }

    #[test]
    fn idempotent_upsert_is_silent() {
        let store = WorldStateStore::new(16);
        let mut rx = store.subscribe();

        let e = entity("station.19", EntityKind::Station, 0.0);
        assert!(store.upsert(e.clone()));
        assert!(!store.upsert(e)); // identical: no change
        assert_eq!(store.version(), 1);
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_err()); // exactly one change broadcast

        // A real change bumps the version again.
        assert!(store.upsert(entity("station.19", EntityKind::Station, 1.0)));
        assert_eq!(store.version(), 2);
    }

    #[test]
    fn remove_reports_existence() {
        let store = WorldStateStore::new(16);
        let id = EntityId::new("station.19");
        assert!(!store.remove(&id));
        store.upsert(entity("station.19", EntityKind::Station, 0.0));
        assert!(store.remove(&id));
        assert!(store.get(&id).is_none());
        assert_eq!(store.version(), 2);
    }

    #[test]
    fn relations_roundtrip_sorted() {
        let store = WorldStateStore::new(16);
        let station = EntityId::new("station.19");
        let r1 = WorldRelation {
            from: station.clone(),
            to: EntityId::new("point.modbus.19.b"),
            kind: "has_point".into(),
        };
        let r2 = WorldRelation {
            from: station.clone(),
            to: EntityId::new("point.modbus.19.a"),
            kind: "has_point".into(),
        };
        assert!(store.relate(r1.clone()));
        assert!(store.relate(r2.clone()));
        assert!(!store.relate(r1.clone())); // duplicate: no change

        let rels = store.relations_from(&station);
        assert_eq!(rels.len(), 2);
        assert_eq!(rels[0].to, EntityId::new("point.modbus.19.a")); // sorted
        assert_eq!(rels[1].to, EntityId::new("point.modbus.19.b"));

        assert!(store.unrelate(&r1));
        assert_eq!(store.relations_from(&station).len(), 1);
    }

    #[test]
    fn list_by_kind_and_snapshot_are_sorted() {
        let store = WorldStateStore::new(16);
        store.upsert(entity("point.b", EntityKind::SensorPoint, 1.0));
        store.upsert(entity("station.1", EntityKind::Station, 0.0));
        store.upsert(entity("point.a", EntityKind::SensorPoint, 2.0));

        let points = store.list_by_kind(&EntityKind::SensorPoint);
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].id, EntityId::new("point.a"));
        assert_eq!(points[1].id, EntityId::new("point.b"));

        let snap = store.snapshot();
        assert_eq!(snap.len(), 3);
        assert_eq!(snap[0].id, EntityId::new("point.a"));
        assert_eq!(snap[2].id, EntityId::new("station.1"));
    }

    #[tokio::test]
    async fn subscribe_observes_change_order() {
        let store = WorldStateStore::new(16);
        let mut rx = store.subscribe();

        store.upsert(entity("a", EntityKind::Node, 0.0));
        store.upsert(entity("b", EntityKind::Node, 0.0));

        let first = rx.recv().await.unwrap();
        let second = rx.recv().await.unwrap();
        assert_eq!(first.version(), 1);
        assert_eq!(second.version(), 2);
    }

    #[derive(Default)]
    struct RecordingLog(std::sync::Mutex<Vec<u64>>);

    impl WorldEventLog for RecordingLog {
        fn append(&self, change: &WorldChange) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(change.version());
            Ok(())
        }
    }

    #[test]
    fn event_log_seam_receives_applied_changes() {
        let store = WorldStateStore::new(16);
        let log = Arc::new(RecordingLog::default());
        store.set_event_log(Some(log.clone()));

        store.upsert(entity("a", EntityKind::Node, 0.0));
        store.upsert(entity("a", EntityKind::Node, 0.0)); // idempotent: not recorded
        store.remove(&EntityId::new("a"));

        assert_eq!(*log.0.lock().unwrap(), vec![1, 2]);
    }

    #[test]
    fn detached_event_log_is_inert() {
        let store = WorldStateStore::new(16);
        store.set_event_log(None);
        assert!(store.upsert(entity("a", EntityKind::Node, 0.0)));
    }
}
