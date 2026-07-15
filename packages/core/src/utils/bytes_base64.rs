use base64::Engine;
use bytes::Bytes;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S: Serializer>(data: &Bytes, s: S) -> Result<S::Ok, S::Error> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
    s.serialize_str(&encoded)
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Bytes, D::Error> {
    let s = String::deserialize(d)?;
    base64::engine::general_purpose::STANDARD
        .decode(&s)
        .map(Bytes::from)
        .map_err(serde::de::Error::custom)
}
