//! Core domain types of the world-state store.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Stable identifier of a world entity.
///
/// Convention: `<scope>.<...>` dotted paths, e.g. `station.19`,
/// `point.modbus.19.pressure_1`, `node.nanopi-r3s-01`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityId(pub String);

impl EntityId {
    /// Wrap a raw id string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Broad classification of a world entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    /// A physical site (factory, tank farm, …).
    Facility,
    /// An addressable industrial station (PLC, remote I/O, robot cell).
    Station,
    /// A single sensed quantity (register/DB point, joint encoder, …).
    SensorPoint,
    /// A controllable element (valve, motor, coil).
    Actuator,
    /// A compute node (edge gateway, server, robot brain).
    Node,
    /// Anything not covered above.
    Custom(String),
}

/// Data quality of an entity's current value set, following the
/// industrial `Good`/`Stale`/`Error` semantics used by evernight telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Quality {
    /// Fresh, trustworthy data.
    Good,
    /// Older than the expected update cadence.
    Stale,
    /// The producer reported a read/communication failure.
    Error,
    /// No quality information available.
    Unknown,
}

impl Quality {
    /// Map a wire-quality string (as used by `IndustrialSensorReading`) to
    /// the typed quality. Unrecognised values degrade to [`Quality::Unknown`].
    pub fn from_str_lossy(raw: &str) -> Self {
        match raw {
            "Good" => Self::Good,
            "Stale" => Self::Stale,
            "Error" => Self::Error,
            _ => Self::Unknown,
        }
    }
}

/// A single attribute value on an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    /// Numeric value (scaled engineering value, rate, …).
    Number(f64),
    /// Free text (unit symbol, protocol name, …).
    Text(String),
    /// Boolean state (open/closed, running/stopped).
    Bool(bool),
    /// Structured payload that does not fit the above.
    Json(serde_json::Value),
}

/// A typed entity in the world model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldEntity {
    /// Stable identifier.
    pub id: EntityId,
    /// Broad classification.
    pub kind: EntityKind,
    /// Current attribute values (ordered map for deterministic snapshots).
    #[serde(default)]
    pub attributes: BTreeMap<String, AttributeValue>,
    /// Data quality of the current attribute set.
    pub quality: Quality,
    /// Wall-clock time of the last update (display / audit).
    pub updated_wall: DateTime<Utc>,
    /// Monotonic capture timestamp in nanoseconds, when the producer has one
    /// (ordering / sensor fusion). `None` for sources without a monotonic
    /// clock — callers must not fabricate one.
    pub updated_mono_ns: Option<u64>,
}

/// Directed relation between two entities (e.g. `station.19` —`has_point`→
/// `point.modbus.19.pressure_1`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldRelation {
    /// Subject entity.
    pub from: EntityId,
    /// Object entity.
    pub to: EntityId,
    /// Relation kind, free-form but conventional (`has_point`, `hosted_on`,
    /// `measures`, `controls`, …).
    pub kind: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_wire_strings_match_industrial_protocol() {
        assert_eq!(serde_json::to_string(&Quality::Good).unwrap(), "\"Good\"");
        assert_eq!(serde_json::to_string(&Quality::Stale).unwrap(), "\"Stale\"");
        assert_eq!(serde_json::to_string(&Quality::Error).unwrap(), "\"Error\"");
        assert_eq!(
            serde_json::from_str::<Quality>("\"Good\"").unwrap(),
            Quality::Good
        );
        assert_eq!(Quality::from_str_lossy("Stale"), Quality::Stale);
        assert_eq!(Quality::from_str_lossy("garbage"), Quality::Unknown);
    }

    #[test]
    fn attribute_value_untagged_roundtrip() {
        let cases = [
            AttributeValue::Number(4.0),
            AttributeValue::Text("MPa".to_string()),
            AttributeValue::Bool(true),
            AttributeValue::Json(serde_json::json!({ "a": 1 })),
        ];
        for value in cases {
            let s = serde_json::to_string(&value).unwrap();
            let back: AttributeValue = serde_json::from_str(&s).unwrap();
            assert_eq!(value, back);
        }
        // Number round-trips as a bare JSON scalar.
        assert_eq!(
            serde_json::to_string(&AttributeValue::Number(4.0)).unwrap(),
            "4.0"
        );
    }

    #[test]
    fn entity_id_ordering_is_lexicographic() {
        assert!(EntityId::new("point.a") < EntityId::new("point.b"));
    }
}
