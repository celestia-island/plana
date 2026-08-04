# plana

**PLANA** - *Protocol for Live Agent Network Automation*: a typed
application-layer protocol for real-time state synchronization and control
between a client shell and a backend service runtime, built on JSON-RPC 2.0
(the way HTTP is built on TCP). Not a general-purpose RPC framework.

This repo is split into three publishable crates:

| Crate | Role in the stack |
|-------|-------------------|
| `plana-celestia-types` | **Celestia domain profile** - the celestia-island platform's typed envelopes and per-domain messages, plugged into PLANA via the macro registration mechanism. |
| `plana-jsonrpc` | **Framing & transport base** - JSON-RPC 2.0 correlation, typed method routing, and Unix-socket / HTTP / WebSocket bindings. |
| `plana` | **The protocol layer** - re-exports the model and the framing, and adds server-side session management (SSE, events) behind the `rpc-server` feature. |



## Architecture: generic core + domain profiles

PLANA is a general protocol: the core (`plana` + `plana-jsonrpc`) defines
framing, routing, and the registration mechanism, and knows nothing about any
specific domain. Platforms plug their message vocabularies in as **domain
profiles**:

- **Registering a domain** - a domain crate uses the `namespace!` macro
  (re-exported as `plana::jsonrpc::namespace`) to declare its typed method
  namespaces; the core router dispatches on them without knowing their
  semantics.
- **The celestia profile** - `plana-celestia-types` is the celestia-island
  platform's domain profile. It ships by default (feature `celestia`) because
  the celestia platform is PLANA's primary consumer; other consumers can
  disable it (`default-features = false`) and register their own profiles.
- **Why it matters** - the protocol is usable beyond its origin: any
  "client shell <-> backend runtime" state-synchronization scenario can
  implement its own profile without forking the protocol.

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
| (none) | yes | `plana-celestia-types` re-exported at the crate root and `plana-jsonrpc` as the `jsonrpc` module — always available. |
| `rpc-server` | no | Server-side session management (`rpc_server::SessionManager`), SSE event streaming, request network/geo detection (`rpc_server::detect_network`). |
| `tracing-helpers` (in `plana-celestia-types`) | no | `plana_types::tracing_helpers` module with a `ShortTimer` formatting type. |

## Stability

Pre-1.0: the API is subject to change. The wire formats (serde layouts,
method naming conventions) are treated carefully — changes that affect
on-the-wire compatibility are always considered breaking.

## Not a general-purpose RPC framework

`plana` is the scaffold for one specific typed bidirectional sync protocol,
not a general-purpose RPC framework. The wire types and JSON-RPC machinery
are shaped by that protocol's needs; for a generic RPC stack, look at
established frameworks instead.
