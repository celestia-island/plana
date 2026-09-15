//! Wire-shape contract tests for the Tier 3 `protocol.*` DTOs.
//!
//! Compiled as an *external* crate, so these tests prove the surface a
//! plugin author actually resolves: the method name constants, the
//! internally tagged JSON shapes (including the `rename_all` spellings),
//! and serde round-trips for every type. The JSON literals pinned here are
//! the contract — changing one is a wire break that must be coordinated
//! with evernight's anchor tests and the Tier 3 guide's interface table.

use plana_celestia_types::evernight::{
    ConnectParamsDto, ConnectResultDto, DataAddressDto, PingParamsDto, PingResultDto,
    ProbeParamsDto, ProbeResultDto, ReadParamsDto, ReadResultDto, TransportInfoDto, WriteParamsDto,
    WriteResultDto, WriteVerificationDto, PROTOCOL_CONNECT_METHOD, PROTOCOL_PING_METHOD,
    PROTOCOL_PROBE_METHOD, PROTOCOL_READ_METHOD, PROTOCOL_WRITE_METHOD,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};

/// Serialize → assert the exact JSON → deserialize → re-serialize → assert
/// the shape survived the round-trip unchanged.
fn round_trip<T>(value: &T, want: Value)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let wire = serde_json::to_value(value).unwrap();
    assert_eq!(wire, want, "serialization must pin the wire shape");
    let back: T = serde_json::from_value(want.clone()).unwrap();
    assert_eq!(&back, value, "deserialization must reconstruct the value");
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        want,
        "the round-tripped value must serialize identically"
    );
}

#[test]
fn method_constants_spell_the_contract() {
    assert_eq!(PROTOCOL_CONNECT_METHOD, "protocol.connect");
    assert_eq!(PROTOCOL_READ_METHOD, "protocol.read");
    assert_eq!(PROTOCOL_WRITE_METHOD, "protocol.write");
    assert_eq!(PROTOCOL_PING_METHOD, "protocol.ping");
    assert_eq!(PROTOCOL_PROBE_METHOD, "protocol.probe");
}

#[test]
fn transport_info_tcp_is_internally_tagged() {
    round_trip(
        &TransportInfoDto::Tcp {
            host: "192.0.2.10".into(),
            port: 502,
        },
        json!({"kind": "tcp", "host": "192.0.2.10", "port": 502}),
    );
}

#[test]
fn transport_info_serial_carries_an_optional_baud_hint() {
    round_trip(
        &TransportInfoDto::Serial {
            port: "/dev/ttyUSB0".into(),
            baud: Some(115_200),
        },
        json!({"kind": "serial", "port": "/dev/ttyUSB0", "baud": 115_200}),
    );

    // An absent hint serializes as `null` (the field is always present)…
    round_trip(
        &TransportInfoDto::Serial {
            port: "/dev/ttyUSB1".into(),
            baud: None,
        },
        json!({"kind": "serial", "port": "/dev/ttyUSB1", "baud": null}),
    );

    // …but a payload that omits the field entirely still deserializes.
    let back: TransportInfoDto =
        serde_json::from_value(json!({"kind": "serial", "port": "/dev/ttyUSB1"})).unwrap();
    assert_eq!(
        back,
        TransportInfoDto::Serial {
            port: "/dev/ttyUSB1".into(),
            baud: None,
        },
        "a missing baud field must default to None, not error"
    );
}

#[test]
fn data_address_modbus_is_internally_tagged() {
    round_trip(
        &DataAddressDto::Modbus {
            station: 1,
            fc: 3,
            address: 0,
            count: 2,
        },
        json!({"kind": "modbus", "station": 1, "fc": 3, "address": 0, "count": 2}),
    );
}

#[test]
fn data_address_s7_keeps_the_db_number_rename() {
    round_trip(
        &DataAddressDto::S7 {
            db_number: 5,
            offset: 0,
            length: 4,
        },
        json!({"kind": "s7", "db_number": 5, "offset": 0, "length": 4}),
    );
}

#[test]
fn data_address_mc_head_address_is_a_u32() {
    // Field type transcribed from the evernight anchor: `head_address: u32`.
    round_trip(
        &DataAddressDto::Mc {
            device: "D".into(),
            head_address: 100,
            count: 2,
        },
        json!({"kind": "mc", "device": "D", "head_address": 100, "count": 2}),
    );
}

#[test]
fn data_address_raw_size_is_a_usize() {
    round_trip(
        &DataAddressDto::Raw {
            address: "hold[0..2]".into(),
            size: 4,
        },
        json!({"kind": "raw", "address": "hold[0..2]", "size": 4}),
    );
}

#[test]
fn connect_params_and_result_pin_their_shapes() {
    round_trip(
        &ConnectParamsDto {
            transport: TransportInfoDto::Tcp {
                host: "192.0.2.10".into(),
                port: 102,
            },
        },
        json!({"transport": {"kind": "tcp", "host": "192.0.2.10", "port": 102}}),
    );
    round_trip(
        &ConnectResultDto { connected: true },
        json!({"connected": true}),
    );
}

#[test]
fn read_params_and_result_pin_their_shapes() {
    round_trip(
        &ReadParamsDto {
            address: DataAddressDto::Modbus {
                station: 1,
                fc: 3,
                address: 0,
                count: 2,
            },
        },
        json!({"address": {"kind": "modbus", "station": 1, "fc": 3, "address": 0, "count": 2}}),
    );
    round_trip(
        &ReadResultDto {
            raw: vec![0, 0, 1, 92],
            latency_us: 500,
        },
        json!({"raw": [0, 0, 1, 92], "latency_us": 500}),
    );
}

#[test]
fn write_params_pin_their_shape() {
    round_trip(
        &WriteParamsDto {
            address: DataAddressDto::S7 {
                db_number: 5,
                offset: 0,
                length: 2,
            },
            data: vec![0x30, 0x39],
        },
        json!({
            "address": {"kind": "s7", "db_number": 5, "offset": 0, "length": 2},
            "data": [48, 57]
        }),
    );
}

#[test]
fn write_result_verification_renames_are_snake_case() {
    round_trip(
        &WriteResultDto {
            confirmed: true,
            verification: WriteVerificationDto::Confirmed,
        },
        json!({"confirmed": true, "verification": "confirmed"}),
    );
    round_trip(
        &WriteResultDto {
            confirmed: false,
            verification: WriteVerificationDto::Unconfirmed,
        },
        json!({"confirmed": false, "verification": "unconfirmed"}),
    );
    round_trip(
        &WriteResultDto {
            confirmed: false,
            verification: WriteVerificationDto::Unknown,
        },
        json!({"confirmed": false, "verification": "unknown"}),
    );
}

#[test]
fn write_result_verification_defaults_to_unknown() {
    assert_eq!(
        WriteVerificationDto::default(),
        WriteVerificationDto::Unknown
    );

    // A legacy `{ "confirmed": bool }` payload (no `verification` field)
    // still deserializes, defaulting the new field to `unknown`.
    let legacy: WriteResultDto = serde_json::from_value(json!({"confirmed": false})).unwrap();
    assert!(!legacy.confirmed);
    assert_eq!(legacy.verification, WriteVerificationDto::Unknown);
}

#[test]
fn ping_params_is_the_empty_object() {
    round_trip(&PingParamsDto {}, json!({}));
    round_trip(
        &PingResultDto { reachable: true },
        json!({"reachable": true}),
    );
}

#[test]
fn ping_params_tolerates_the_omitted_params_form() {
    // JSON-RPC 2.0 may omit `params` for a no-argument method (the
    // TypeScript sibling client does when called without an argument), and
    // plana's router hands such a handler `Value::Null` — the DTO must
    // treat that as the same empty params, not an error. Serialization
    // stays the canonical `{}` pinned above.
    let from_null: PingParamsDto = serde_json::from_value(Value::Null).unwrap();
    assert_eq!(from_null, PingParamsDto {});
    let from_text: PingParamsDto = serde_json::from_str("null").unwrap();
    assert_eq!(from_text, PingParamsDto {});
    assert_eq!(
        serde_json::to_value(PingParamsDto {}).unwrap(),
        json!({}),
        "serialization must stay the canonical empty object"
    );
}

#[test]
fn read_result_raw_covers_the_empty_and_large_array_edges() {
    // An empty read serializes as an empty JSON array (`[]`, not `null`).
    round_trip(
        &ReadResultDto {
            raw: vec![],
            latency_us: 0,
        },
        json!({"raw": [], "latency_us": 0}),
    );

    // A large block (plus the u64 latency ceiling) must survive the
    // Value round-trip byte-for-byte; pinned without a giant literal.
    let big = ReadResultDto {
        raw: (0..=255u8).cycle().take(4096).collect(),
        latency_us: u64::MAX,
    };
    let wire = serde_json::to_value(&big).unwrap();
    assert_eq!(wire["raw"].as_array().map(Vec::len), Some(4096));
    let back: ReadResultDto = serde_json::from_value(wire).unwrap();
    assert_eq!(back, big);
}

#[test]
fn probe_params_and_result_pin_their_shapes() {
    round_trip(
        &ProbeParamsDto {
            transport: TransportInfoDto::Serial {
                port: "/dev/ttyUSB0".into(),
                baud: Some(9_600),
            },
        },
        json!({"transport": {"kind": "serial", "port": "/dev/ttyUSB0", "baud": 9600}}),
    );
    round_trip(
        &ProbeResultDto {
            protocol: "modbus_tcp".into(),
            // Exactly representable in f32, so the widened f64 the JSON
            // number carries compares equal (0.85 would surface f32
            // precision as 0.8500000238418579).
            confidence: 0.75,
        },
        json!({"protocol": "modbus_tcp", "confidence": 0.75}),
    );
}

#[test]
fn probe_result_confidence_precision_is_pinned_across_both_paths() {
    // A non-dyadic confidence surfaces the two serialization paths
    // differently, and both are legal wire forms:
    //
    // - `to_value` widens f32 → f64 (serde_json numbers carry f64), so the
    //   Value-based wire — what `RpcClient::call(method, to_value(dto))`
    //   actually sends — carries the widened repr 0.9900000095367432;
    // - direct text serialization formats the f32 itself, emitting the
    //   shortest round-tripping decimal "0.99".
    //
    // Both deserialize back to the same f32, which is the actual contract.
    let dto = ProbeResultDto {
        protocol: "modbus_tcp".into(),
        confidence: 0.99,
    };

    let widened = json!({"protocol": "modbus_tcp", "confidence": f64::from(0.99f32)});
    assert_eq!(serde_json::to_value(&dto).unwrap(), widened);
    let from_widened: ProbeResultDto = serde_json::from_value(widened).unwrap();
    assert_eq!(from_widened, dto);

    let text = serde_json::to_string(&dto).unwrap();
    assert_eq!(text, r#"{"protocol":"modbus_tcp","confidence":0.99}"#);
    let from_text: ProbeResultDto = serde_json::from_str(&text).unwrap();
    assert_eq!(from_text, dto);
}
