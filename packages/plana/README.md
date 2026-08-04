# plana

**plana implements the Sync protocol** - a typed application-layer protocol for
**real-time state synchronization and control between a client shell and a
backend service runtime**.

## What the Sync protocol is

A UI that stands in front of a live backend - a chat client, a control panel,
a terminal - needs more than point-to-point RPC. It needs to:

- **Connect and identify** - version handshake, identity exchange, mismatch
  handling.
- **Stay in sync** - the backend pushes *snapshots and patches* of its world
  state (agents, containers, tasks, VMs, models, providers) so the client
  mirrors it in real time.
- **Carry typed bidirectional events** - agent streaming responses, thinking
  steps, tool calls, reports and transfers; task and container lifecycle;
  skill-chain execution; system messages.
- **Involve humans** - ask-human and review flows.
- **Synchronize catalogs** - models and providers discovered from files or a
  registry, pushed to every client.
- **Speak per-domain messages** - auth, industrial telemetry, file browsing,
  logs, panels and workspaces, device control (e.g. YOLO inference engines).

The Sync protocol defines all of this as **typed messages**, shared by both
sides through one message model with JSON Schema and TypeScript-bindings
generation.

## Layering

The Sync protocol is built on **JSON-RPC 2.0** the same way HTTP is built on
TCP: JSON-RPC supplies generic framing and transports; Sync defines what the
messages *mean* and how state is synchronized. If you only need generic remote
calls, use a plain JSON-RPC framework (e.g. jsonrpsee); if you need a real
synchronization protocol, use plana.

This repo is split into three publishable crates:

| Crate | Role in the stack |
|-------|-------------------|
| `plana-types` | **Message model** - the Sync protocol's typed envelopes and per-domain messages, plus health/network descriptors, identity, RBAC, region policy, model metadata and MCP I/O structs. |
| `plana-jsonrpc` | **Framing & transport base** - JSON-RPC 2.0 correlation, typed method routing, and Unix-socket / HTTP / WebSocket bindings. |
| `plana` | **The protocol layer** - re-exports the model and the framing, and adds server-side session management (SSE, events) behind the `rpc-server` feature. |

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
