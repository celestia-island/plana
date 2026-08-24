//! Wire-format lock for the `Sync.StatePatch` notification params.
//!
//! `PatchOp` is the serde shape of the `Sync.StatePatch` JSON-RPC
//! notification: field order (`op`, `path`, `value`), the lowercase op
//! tags and the `value`-omitted `del` form are all locked byte for byte.
//! These goldens were captured from the pre-yuuka in-tree implementation
//! and must never drift — clients and servers of different versions
//! interoperate through this exact byte shape.

use plana_sync::patch::{PatchKind, PatchOp};
use serde_json::json;

#[test]
fn golden_set_op_serializes_like_pre_migration_plana() {
    let op = PatchOp::set("state.agents.hubris", json!({"status":"idle"}));
    // Field order (op, path, value) and the lowercase op tag are locked.
    assert_eq!(
        serde_json::to_string(&op).unwrap(),
        r#"{"op":"set","path":"state.agents.hubris","value":{"status":"idle"}}"#
    );
    assert_eq!(
        serde_json::to_value(&op).unwrap(),
        json!({"op":"set","path":"state.agents.hubris","value":{"status":"idle"}})
    );
}

#[test]
fn golden_replace_op_serializes_like_pre_migration_plana() {
    let op = PatchOp::replace("state.work_status", json!({"Running":{"code":1}}));
    assert_eq!(
        serde_json::to_string(&op).unwrap(),
        r#"{"op":"replace","path":"state.work_status","value":{"Running":{"code":1}}}"#
    );
    assert_eq!(
        serde_json::to_value(&op).unwrap(),
        json!({"op":"replace","path":"state.work_status","value":{"Running":{"code":1}}})
    );
}

#[test]
fn golden_del_op_omits_the_value_key() {
    let op = PatchOp::del("state.agents.kalos");
    // `value` is skipped entirely — the wire object has exactly two keys.
    assert_eq!(
        serde_json::to_string(&op).unwrap(),
        r#"{"op":"del","path":"state.agents.kalos"}"#
    );
    let as_value = serde_json::to_value(&op).unwrap();
    assert_eq!(as_value.as_object().unwrap().len(), 2);
    assert!(as_value.get("value").is_none());
}

#[test]
fn golden_patch_kind_tags_are_lowercase() {
    assert_eq!(serde_json::to_value(PatchKind::Set).unwrap(), json!("set"));
    assert_eq!(
        serde_json::to_value(PatchKind::Replace).unwrap(),
        json!("replace")
    );
    assert_eq!(serde_json::to_value(PatchKind::Del).unwrap(), json!("del"));
}

#[test]
fn ops_deserialize_like_pre_migration_plana() {
    let set: PatchOp = serde_json::from_str(r#"{"op":"set","path":"p","value":42}"#).unwrap();
    assert_eq!(set, PatchOp::set("p", json!(42)));

    let replace: PatchOp =
        serde_json::from_str(r#"{"op":"replace","path":"p","value":{"a":1}}"#).unwrap();
    assert_eq!(replace, PatchOp::replace("p", json!({"a":1})));

    let del: PatchOp = serde_json::from_str(r#"{"op":"del","path":"p"}"#).unwrap();
    assert_eq!(del, PatchOp::del("p"));
    assert_eq!(del.value, None);
    assert_eq!(del.op, PatchKind::Del);

    // An explicit JSON null for `value` also lands as `None` (the skip
    // only affects serialization).
    let del_null: PatchOp =
        serde_json::from_str(r#"{"op":"del","path":"p","value":null}"#).unwrap();
    assert_eq!(del_null, PatchOp::del("p"));
}

#[test]
fn ops_roundtrip_through_serde_value() {
    for op in [
        PatchOp::set("state.a.b", json!({"x":[1,2],"y":"z"})),
        PatchOp::replace("state.c", json!("done")),
        PatchOp::del("state.d.e"),
    ] {
        let back: PatchOp = serde_json::from_value(serde_json::to_value(&op).unwrap()).unwrap();
        assert_eq!(back, op);
    }
}
