use std::net::{IpAddr, SocketAddr};
use plana::http::NetworkInfo;

/// Build NetworkInfo from server-side request data.
/// Reads X-Forwarded-For to get real client IP when behind a proxy.
pub fn detect_network(addr: &SocketAddr, headers: &axum::http::HeaderMap) -> NetworkInfo {
    let client_ip = client_ip_from_headers(headers).unwrap_or(addr.ip());
    let transport = detect_transport(&client_ip, headers);
    let region = detect_region(&client_ip);
    NetworkInfo {
        transport,
        region,
        asn: None,
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
    if ip.is_loopback() || is_private(ip) {
        return "local".into();
    }
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
    "poll".into()
}

fn detect_region(ip: &IpAddr) -> String {
    if ip.is_loopback() || is_private(ip) {
        return "XX".into();
    }
    "XX".into()
}

fn is_private(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private(),
        IpAddr::V6(_v6) => false,
    }
}
