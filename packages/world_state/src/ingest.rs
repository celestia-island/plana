//! Telemetry ingestion: map `plana` industrial readings onto
//! world-state entities.
//!
//! The first producer on the embodied-AI roadmap is industrial telemetry
//! (`Sync.IndustrialTelemetryPush`, whose payload types live in
//! `plana` and mirror the entelecheia wire protocol 1:1).
//! [`apply_telemetry_batch`] turns one batch into:
//!
//! - one `station.<id>` entity ([`EntityKind::Station`]),
//! - one `point.<protocol>.<station>.<name>` entity per reading
//!   ([`EntityKind::SensorPoint`]) with the reading's value set as
//!   attributes and its wire quality mapped to [`Quality`],
//! - a `has_point` relation from the station to each point.

use chrono::{DateTime, Utc};
use plana::ws::services::industrial::{IndustrialSensorReading, IndustrialTelemetryBatch};

use crate::store::WorldStateStore;
use crate::types::{AttributeValue, EntityId, EntityKind, Quality, WorldEntity, WorldRelation};

/// Well-known attribute keys used by the telemetry ingestion.
pub mod attrs {
    /// Engineering value after scaling.
    pub const SCALED_VALUE: &str = "scaled_value";
    /// Raw producer value.
    pub const RAW_VALUE: &str = "raw_value";
    /// Unit symbol (e.g. "MPa").
    pub const UNIT: &str = "unit";
    /// Producer address (e.g. "HR:16", "DB1.DBW4").
    pub const ADDRESS: &str = "address";
    /// Producing protocol (e.g. "modbus_rtu").
    pub const PROTOCOL: &str = "protocol";
}

fn parse_wall(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

fn point_entity_id(reading: &IndustrialSensorReading) -> EntityId {
    EntityId::new(format!(
        "point.{}.{}.{}",
        reading.protocol, reading.station_id, reading.name
    ))
}

fn station_entity_id(station_id: &str) -> EntityId {
    EntityId::new(format!("station.{station_id}"))
}

/// Map one reading to a point entity. `fallback_wall` is used when the
/// reading carries no parseable timestamp — callers pass the stored entity's
/// previous wall time so steady-state producers with broken timestamps stay
/// idempotent (a fresh `Utc::now()` would manufacture a change per apply).
fn reading_to_entity(
    reading: &IndustrialSensorReading,
    fallback_wall: DateTime<Utc>,
) -> WorldEntity {
    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert(
        attrs::SCALED_VALUE.to_string(),
        AttributeValue::Number(reading.scaled_value),
    );
    attributes.insert(
        attrs::RAW_VALUE.to_string(),
        AttributeValue::Number(reading.raw_value),
    );
    attributes.insert(
        attrs::UNIT.to_string(),
        AttributeValue::Text(reading.unit.clone()),
    );
    attributes.insert(
        attrs::ADDRESS.to_string(),
        AttributeValue::Text(reading.address.clone()),
    );
    attributes.insert(
        attrs::PROTOCOL.to_string(),
        AttributeValue::Text(reading.protocol.clone()),
    );

    WorldEntity {
        id: point_entity_id(reading),
        kind: EntityKind::SensorPoint,
        attributes,
        quality: Quality::from_str_lossy(&reading.quality),
        updated_wall: parse_wall(&reading.timestamp).unwrap_or(fallback_wall),
        updated_mono_ns: None,
    }
}

/// Ingest one telemetry batch into the store. Returns the number of store
/// changes applied (entities actually inserted/updated + relations added).
///
/// Protocol assumption: one batch carries one station; relations attach to
/// the batch-level station id.
pub fn apply_telemetry_batch(store: &WorldStateStore, batch: &IndustrialTelemetryBatch) -> usize {
    let mut changes = 0;

    let station_id = station_entity_id(&batch.station_id);
    let station_wall = parse_wall(&batch.timestamp).unwrap_or_else(|| {
        store
            .get(&station_id)
            .map(|existing| existing.updated_wall)
            .unwrap_or_else(Utc::now)
    });
    let station = WorldEntity {
        id: station_id.clone(),
        kind: EntityKind::Station,
        attributes: std::collections::BTreeMap::new(),
        quality: Quality::Unknown,
        updated_wall: station_wall,
        updated_mono_ns: None,
    };
    if store.upsert(station) {
        changes += 1;
    }

    for reading in &batch.readings {
        let point_id = point_entity_id(reading);
        let fallback_wall = store
            .get(&point_id)
            .map(|existing| existing.updated_wall)
            .unwrap_or_else(Utc::now);
        let point = reading_to_entity(reading, fallback_wall);
        if store.upsert(point) {
            changes += 1;
        }
        if store.relate(WorldRelation {
            from: station_id.clone(),
            to: point_id,
            kind: "has_point".to_string(),
        }) {
            changes += 1;
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use plana::ws::services::industrial::IndustrialSensorReading;

    fn reading(name: &str, value: f64, quality: &str, ts: &str) -> IndustrialSensorReading {
        IndustrialSensorReading {
            station_id: "19".to_string(),
            protocol: "modbus_rtu".to_string(),
            address: "HR:16".to_string(),
            name: name.to_string(),
            raw_value: value * 10.0,
            scaled_value: value,
            unit: "MPa".to_string(),
            quality: quality.to_string(),
            timestamp: ts.to_string(),
        }
    }

    fn batch(readings: Vec<IndustrialSensorReading>) -> IndustrialTelemetryBatch {
        IndustrialTelemetryBatch {
            station_id: "19".to_string(),
            timestamp: "2026-08-02T00:00:00Z".to_string(),
            readings,
        }
    }

    #[test]
    fn apply_creates_station_points_and_relations() {
        let store = WorldStateStore::new(64);
        let batch = batch(vec![
            reading("pressure_1", 4.0, "Good", "2026-08-02T00:00:01Z"),
            reading("temp_1", 40.0, "Stale", "2026-08-02T00:00:01Z"),
        ]);

        let changes = apply_telemetry_batch(&store, &batch);
        // 1 station + 2 points upserts + 2 relations.
        assert_eq!(changes, 5);

        let station = store.get(&EntityId::new("station.19")).unwrap();
        assert_eq!(station.kind, EntityKind::Station);

        let point = store
            .get(&EntityId::new("point.modbus_rtu.19.pressure_1"))
            .unwrap();
        assert_eq!(point.kind, EntityKind::SensorPoint);
        assert_eq!(point.quality, Quality::Good);
        assert_eq!(
            point.attributes.get(attrs::SCALED_VALUE),
            Some(&AttributeValue::Number(4.0))
        );
        assert_eq!(
            point.attributes.get(attrs::UNIT),
            Some(&AttributeValue::Text("MPa".to_string()))
        );

        let stale = store
            .get(&EntityId::new("point.modbus_rtu.19.temp_1"))
            .unwrap();
        assert_eq!(stale.quality, Quality::Stale);

        let rels = store.relations_from(&EntityId::new("station.19"));
        assert_eq!(rels.len(), 2);
        assert!(rels.iter().all(|r| r.kind == "has_point"));
    }

    #[test]
    fn repeated_apply_is_idempotent() {
        let store = WorldStateStore::new(64);
        let batch = batch(vec![reading(
            "pressure_1",
            4.0,
            "Good",
            "2026-08-02T00:00:01Z",
        )]);

        let first = apply_telemetry_batch(&store, &batch);
        assert_eq!(first, 3); // station + point + relation
        let second = apply_telemetry_batch(&store, &batch);
        assert_eq!(second, 0); // nothing changed
    }

    #[test]
    fn unknown_quality_degrades_to_unknown() {
        let store = WorldStateStore::new(64);
        let batch = batch(vec![reading(
            "pressure_1",
            4.0,
            "Questionable",
            "2026-08-02T00:00:01Z",
        )]);
        apply_telemetry_batch(&store, &batch);
        let point = store
            .get(&EntityId::new("point.modbus_rtu.19.pressure_1"))
            .unwrap();
        assert_eq!(point.quality, Quality::Unknown);
    }

    #[test]
    fn unparsable_timestamp_reuses_stored_wall_time() {
        let store = WorldStateStore::new(64);
        let before = Utc::now();
        let bad = batch(vec![reading("pressure_1", 4.0, "Good", "not-a-timestamp")]);
        apply_telemetry_batch(&store, &bad);
        let first = store
            .get(&EntityId::new("point.modbus_rtu.19.pressure_1"))
            .unwrap();
        assert!(first.updated_wall >= before);

        // Re-applying the same bad-timestamp reading stays idempotent: the
        // stored wall time is reused instead of a fresh Utc::now().
        let changes = apply_telemetry_batch(&store, &bad);
        assert_eq!(changes, 0);
        let second = store
            .get(&EntityId::new("point.modbus_rtu.19.pressure_1"))
            .unwrap();
        assert_eq!(second.updated_wall, first.updated_wall);
    }
}
