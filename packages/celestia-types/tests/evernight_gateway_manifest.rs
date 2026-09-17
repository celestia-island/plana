//! TOML round-trip tests for the evernight gateway manifest types.
//!
//! The manifest is a **hand-writable TOML document** (design invariant);
//! these tests pin the exact wire shape so a schema change is caught by
//! CI before it reaches either consumer (chest writer, evernight reader).

use plana_celestia_types::evernight::*;

const MINIMAL: &str = r#"
version = 1
id = "s3-front"
generated_by = "hand-written"
exit_node = "ev-tier3-01"

[[servers]]
host = ["s3.celestia.world"]

  [[servers.routes]]
  id = "artifacts"
  action = { type = "proxy", upstream = ["http://192.0.2.1:3009"] }

  [[servers.routes]]
  id = "healthz"
  match = { path = ["/healthz"] }
  action = { type = "static", status = 200, body = "ok" }
"#;

#[test]
fn minimal_manifest_round_trips_through_toml() {
    let parsed: GatewayManifest = toml::from_str(MINIMAL).expect("minimal manifest must parse");
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.exit_node, "ev-tier3-01");
    assert_eq!(parsed.servers.len(), 1);
    assert_eq!(parsed.servers[0].host, vec!["s3.celestia.world"]);
    assert_eq!(parsed.servers[0].listen, "0.0.0.0:443"); // default
    assert!(parsed.servers[0].redirect_http); // default
    assert_eq!(parsed.servers[0].routes.len(), 2);

    // Route 1: proxy with defaults (preserve_host defaults true).
    assert!(matches!(
        &parsed.servers[0].routes[0].action,
        GatewayAction::Proxy { upstream, preserve_host, .. }
            if upstream == &vec!["http://192.0.2.1:3009".to_string()] && *preserve_host
    ));

    // Route 2: static respond.
    assert!(matches!(
        &parsed.servers[0].routes[1].action,
        GatewayAction::Static { status: 200, body } if body == "ok"
    ));

    // Round-trip: serialize back to TOML, re-parse, compare.
    let back = toml::to_string(&parsed).expect("serialize back");
    let re: GatewayManifest = toml::from_str(&back).expect("re-parse");
    assert_eq!(parsed, re);
}

#[test]
fn full_manifest_round_trips_with_tls_and_middleware() {
    let toml_str = r#"
version = 1
id = "full-example"
exit_node = "ev-hub-01"

[[servers]]
host = ["s3.celestia.world", "s3.example.com"]
listen = "0.0.0.0:8443"
redirect_http = false

  [servers.tls]
  type = "static"
  cert = "door:s3.celestia.world"
  key = "door:s3.celestia.world"

  [[servers.routes]]
  id = "presigned"
  match = { path_prefix = ["/bucket-a/", "/bucket-b/"], methods = ["GET", "HEAD", "PUT"] }
  action = { type = "proxy", upstream = ["http://192.0.2.1:3009", "http://192.0.2.2:3009"], lb = "round_robin", preserve_host = true }
  body_max = "4GiB"
  timeout = "600s"

  [servers.routes.ratelimit]
  requests = 100
  per = "1m"
  key = "ip"

  [[servers.routes]]
  id = "redirect-old"
  match = { path_prefix = ["/legacy/"] }
  action = { type = "redirect", to = "https://s3.celestia.world/", status = 308 }

  [[servers.routes]]
  id = "deny-admin"
  match = { path_prefix = ["/admin/"] }
  action = { type = "reject", status = 403 }
"#;
    let parsed: GatewayManifest = toml::from_str(toml_str).expect("full manifest must parse");
    assert_eq!(parsed.servers[0].host.len(), 2);
    assert_eq!(parsed.servers[0].listen, "0.0.0.0:8443");
    assert!(!parsed.servers[0].redirect_http);
    assert!(parsed.servers[0].tls.is_some());

    let tls = parsed.servers[0].tls.as_ref().unwrap();
    assert_eq!(tls.mode, "static");
    assert_eq!(tls.cert, "door:s3.celestia.world");

    // Rate limit
    let rl = parsed.servers[0].routes[0].ratelimit.as_ref().unwrap();
    assert_eq!(rl.requests, 100);

    // Round-trip
    let back = toml::to_string(&parsed).expect("serialize");
    let re: GatewayManifest = toml::from_str(&back).expect("re-parse");
    assert_eq!(parsed, re);
}

#[test]
fn json_round_trip_for_rpc_wire() {
    // The same types travel as JSON inside the relay channel (Config.Apply
    // is a JSON-RPC method, not a TOML file).
    let manifest = GatewayManifest {
        version: 1,
        id: "test".into(),
        generated_by: "chest".into(),
        exit_node: "ev-1".into(),
        servers: vec![GatewayServer {
            host: vec!["x.example.com".into()],
            listen: "0.0.0.0:443".into(),
            tls: None,
            redirect_http: true,
            routes: vec![GatewayRoute {
                id: "r1".into(),
                r#match: GatewayMatch {
                    path_prefix: vec!["/api/".into()],
                    ..Default::default()
                },
                action: GatewayAction::Proxy {
                    upstream: vec!["http://127.0.0.1:3005".into()],
                    lb: "round_robin".into(),
                    preserve_host: false,
                },
                use_: vec![],
                ratelimit: None,
                body_max: None,
                timeout: None,
            }],
        }],
    };
    let json = serde_json::to_string(&manifest).unwrap();
    let back: GatewayManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(manifest, back);
}

#[test]
fn config_apply_params_round_trip() {
    let params = ConfigApplyParams {
        manifest: GatewayManifest {
            version: 1,
            id: "m1".into(),
            generated_by: String::new(),
            exit_node: "ev-1".into(),
            servers: vec![],
        },
        config_version: 42,
    };
    let json = serde_json::to_string(&params).unwrap();
    let back: ConfigApplyParams = serde_json::from_str(&json).unwrap();
    assert_eq!(back.config_version, 42);
    assert_eq!(back.manifest.id, "m1");
}

#[test]
fn the_middleware_list_round_trips_under_the_natural_toml_key() {
    // Hand-written manifests use `use = [...]` (not `use_`); without the
    // serde rename the field was silently dropped. Pinned by R2.
    let toml_str = r#"
version = 1
id = "mw-example"
exit_node = "ev-1"

[[servers]]
host = ["x.example.com"]

  [[servers.routes]]
  id = "r1"
  match = { path_prefix = ["/api/"] }
  action = { type = "proxy", upstream = ["http://127.0.0.1:3005"] }
  use = ["auth-required"]
"#;
    let parsed: GatewayManifest = toml::from_str(toml_str).expect("must parse");
    assert_eq!(
        parsed.servers[0].routes[0].use_,
        vec!["auth-required".to_string()],
        "the natural TOML key must populate use_"
    );
    // Serializing back must emit `use` (not `use_`).
    let back = toml::to_string(&parsed).unwrap();
    assert!(
        back.contains("use = ["),
        "serialized key must be 'use': {back}"
    );
    assert!(!back.contains("use_"), "no underscore leak: {back}");
}
