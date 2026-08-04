# plana

Scaffolding for a typed bidirectional sync protocol. This repo is split into
three publishable crates:

| Crate | Purpose |
|-------|---------|
| `plana-types` | Shared wire types: JSON-RPC envelopes, health/network descriptors, identity, region policy, RBAC, model metadata, and per-tool MCP I/O structs — every type `Serialize`/`Deserialize` with JSON Schema and TypeScript-bindings support. |
| `plana-jsonrpc` | JSON-RPC 2.0 wire machinery: request/response/notification framing, a method registry, Unix-socket and HTTP/WebSocket transports, and a typed bridge between method strings and serde-serializable messages. |
| `plana` | Umbrella crate: re-exports `plana-types` at the crate root (`plana::http::...`, `plana::protocol::...`) and `plana-jsonrpc` as the `jsonrpc` module (`plana::jsonrpc::...`), plus server-side session management (SSE, events) behind the `rpc-server` feature. |

## Usage

Add to your `Cargo.toml` — either the umbrella crate:

```toml
[dependencies]
plana = { version = "0.1", features = ["rpc-server"] }
```

or just the parts you need:

```toml
[dependencies]
plana-types = { version = "0.1" }
plana-jsonrpc = { version = "0.1" }
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

| Feature | Default | What it enables |
|---------|---------|-----------------|
| (none) | yes | `plana-types` re-exported at the crate root and `plana-jsonrpc` as the `jsonrpc` module — always available. |
| `rpc-server` | no | Server-side session management (`rpc_server::SessionManager`), SSE event streaming, request network/geo detection (`rpc_server::detect_network`). |
| `tracing-helpers` (in `plana-types`) | no | `plana_types::tracing_helpers` module with a `ShortTimer` formatting type. |

## Stability

Pre-1.0: the API is subject to change. The wire formats (serde layouts,
method naming conventions) are treated carefully — changes that affect
on-the-wire compatibility are always considered breaking.

## Not a general-purpose RPC framework

`plana` is the scaffold for one specific typed bidirectional sync protocol,
not a general-purpose RPC framework. The wire types and JSON-RPC machinery
are shaped by that protocol's needs; for a generic RPC stack, look at
established frameworks instead.
