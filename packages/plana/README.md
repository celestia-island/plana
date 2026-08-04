# plana

Typed JSON-RPC 2.0 protocol toolkit for Rust: shared wire types, JSON-RPC
2.0 machinery, Unix-socket and HTTP/WebSocket transports, and server-side
session management — split into feature-gated modules so you pull in only
what you need.

The `types` module (on by default) provides the wire types themselves:
`Serialize`/`Deserialize` structs and enums with JSON Schema and
TypeScript-bindings support, covering protocol envelopes, transport-agnostic
message params, per-tool MCP I/O structs, health/network descriptors, and
identity/region/RBAC metadata.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
plana = { version = "0.1", features = ["jsonrpc", "rpc-server"] }
```

### Registering and calling a JSON-RPC method

```rust
use plana::jsonrpc::RpcMethodMap;
use serde_json::{Value, json};

let methods = RpcMethodMap::empty().method("echo", |params: Value| async move {
    Ok(json!({ "echo": params["text"].clone() }))
});

let resp = methods.dispatch(Some(json!({ "text": "hello" })), "echo").await;
assert_eq!(resp.result.unwrap(), json!({ "echo": "hello" }));

let missing = methods.dispatch(None, "nope").await;
assert_eq!(missing.error.unwrap().code, -32601); // METHOD_NOT_FOUND
```

### Using the shared health/network wire types

```rust
use plana::http::{BackendKind, HealthResponse, NetworkInfo, ServiceStatus};

let health = HealthResponse {
    status: ServiceStatus::Ok,
    version: "1.0.0".to_string(),
    kind: BackendKind::Dev,
    uptime: 42,
    network: NetworkInfo {
        transport: "local".to_string(),
        region: "XX".to_string(),
        asn: None,
    },
    build_hash: None,
    engine_version: None,
};
let json = serde_json::to_string(&health)?;
```

### Unix-socket transport

```rust
use plana::jsonrpc::{JsonRpcRequest, TimeoutPolicy, unix_transport::JsonRpcTransport};

let mut transport = JsonRpcTransport::connect("/run/my-app/rpc.sock").await?;
let response = transport.send(&JsonRpcRequest::new("ping", None), TimeoutPolicy::Default).await?;
```

## Feature flags

| Feature        | Default | What it enables                                                                 |
|----------------|---------|---------------------------------------------------------------------------------|
| `types`        | yes     | Shared wire types: protocol envelopes, message params, MCP I/O, http, identity, region, rbac, model. |
| `jsonrpc`      | no      | JSON-RPC 2.0 layer: request/response/notification types, method registry (`RpcMethodMap`), Unix-socket transports, bridge helpers. |
| `rpc-server`   | no      | Server-side session management (`SessionManager`), SSE event streaming, request network/geo detection. Implies `jsonrpc`. |
| `tracing-helpers` | no   | `tracing_helpers` module with a `ShortTimer` formatting type.                  |

## Stability

Pre-1.0: the API is subject to change. The wire formats (serde layouts,
method naming conventions) are treated carefully — changes that affect
on-the-wire compatibility are always considered breaking.
