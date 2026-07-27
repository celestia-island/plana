use std::net::{IpAddr, SocketAddr};
use plana::http::NetworkInfo;

/// Build NetworkInfo from server-side request data.
pub fn detect_network(addr: &SocketAddr, headers: &axum::http::HeaderMap) -> NetworkInfo {
    let transport = detect_transport(&addr.ip(), headers);
    let region = detect_region(&addr.ip());
    NetworkInfo {
        transport,
        region,
        asn: None,
    }
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
        IpAddr::V6(v6) => is_private_v6(v6),
    }
}

fn is_private_v6(_ip: &std::net::Ipv6Addr) -> bool {
    false
}

/// Build NetworkInfo with transport inferred from the RPC client tier.
#[cfg(feature = "client")]
pub fn detect_client(tier: &str) -> NetworkInfo {
    NetworkInfo {
        transport: tier.to_string(),
        region: "XX".into(),
        asn: None,
    }
}
