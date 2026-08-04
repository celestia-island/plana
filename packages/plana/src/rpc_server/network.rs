use crate::http::NetworkInfo;
use std::net::{IpAddr, SocketAddr};
use std::sync::OnceLock;

static GEOIP_READER: OnceLock<Option<maxminddb::Reader<Vec<u8>>>> = OnceLock::new();
static ASN_READER: OnceLock<Option<maxminddb::Reader<Vec<u8>>>> = OnceLock::new();

fn geoip_reader() -> Option<&'static maxminddb::Reader<Vec<u8>>> {
    GEOIP_READER
        .get_or_init(|| {
            let path = std::env::var("GEOIP_DB_PATH")
                .unwrap_or_else(|_| "/usr/share/GeoIP/GeoLite2-Country.mmdb".into());
            match maxminddb::Reader::open_readfile(&path) {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::debug!("GeoIP database not available: {e}");
                    None
                }
            }
        })
        .as_ref()
}

fn asn_reader() -> Option<&'static maxminddb::Reader<Vec<u8>>> {
    ASN_READER
        .get_or_init(|| {
            let path = std::env::var("ASN_DB_PATH")
                .unwrap_or_else(|_| "/usr/share/GeoIP/GeoLite2-ASN.mmdb".into());
            match maxminddb::Reader::open_readfile(&path) {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::debug!("ASN database not available: {e}");
                    None
                }
            }
        })
        .as_ref()
}

/// Build NetworkInfo from server-side request data.
/// Reads X-Forwarded-For to get real client IP when behind a proxy.
pub fn detect_network(addr: &SocketAddr, headers: &axum::http::HeaderMap) -> NetworkInfo {
    let client_ip = client_ip_from_headers(headers).unwrap_or(addr.ip());
    let transport = detect_transport(&client_ip, headers);
    let region = detect_region(&client_ip);
    let asn = detect_asn(&client_ip);
    NetworkInfo {
        transport,
        region,
        asn,
    }
}

fn client_ip_from_headers(headers: &axum::http::HeaderMap) -> Option<IpAddr> {
    headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
}

fn detect_transport(ip: &IpAddr, headers: &axum::http::HeaderMap) -> String {
    let is_proxied = headers.contains_key("X-Forwarded-For") || headers.contains_key("X-Real-IP");
    if headers.contains_key("Sec-WebSocket-Key") {
        return "ws".into();
    }
    if headers
        .get("Accept")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("")
        .contains("text/event-stream")
    {
        return "sse".into();
    }
    if !is_proxied && (ip.is_loopback() || is_private(ip)) {
        return "local".into();
    }
    "poll".into()
}

fn detect_region(ip: &IpAddr) -> String {
    if ip.is_loopback() || is_private(ip) {
        return "XX".into();
    }
    if let Some(reader) = geoip_reader() {
        if let Ok(result) = reader.lookup(*ip) {
            if let Ok(Some(country)) = result.decode::<maxminddb::geoip2::Country>() {
                if let Some(iso) = country.country.iso_code {
                    return iso.to_string();
                }
            }
        }
    }
    "XX".into()
}

fn detect_asn(ip: &IpAddr) -> Option<u32> {
    if ip.is_loopback() || is_private(ip) {
        return None;
    }
    if let Some(reader) = asn_reader() {
        if let Ok(result) = reader.lookup(*ip) {
            if let Ok(Some(asn)) = result.decode::<maxminddb::geoip2::Asn>() {
                return asn.autonomous_system_number;
            }
        }
    }
    None
}

fn is_private(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private(),
        IpAddr::V6(_v6) => false,
    }
}
