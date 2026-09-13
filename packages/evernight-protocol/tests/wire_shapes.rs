//! Wire-shape contract tests for the Tier 3 `protocol.*` DTOs.
//!
//! Compiled as an *external* crate, so these tests prove the surface a
//! plugin author actually resolves: the method name constants, the
//! internally tagged JSON shapes (including the `rename_all` spellings),
//! and serde round-trips for every type. The JSON literals pinned here are
//! the contract — changing one is a wire break that must be coordinated
//! with evernight's anchor tests and the Tier 3 guide's interface table.

use plana_evernight_protocol::{
    ConnectParamsDto, ConnectResultDto, DataAddressDto, PROTOCOL_CONNECT_METHOD,
    PROTOCOL_PING_METHOD, PROTOCOL_PROBE_METHOD, PROTOCOL_READ_METHOD, PROTOCOL_WRITE_METHOD,
    PingParamsDto, PingResultDto, ProbeParamsDto, ProbeResultDto, ReadParamsDto, ReadResultDto,
    TransportInfoDto, WriteParamsDto, WriteResultDto, WriteVerificationDto,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

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
