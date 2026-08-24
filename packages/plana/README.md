# plana

**PLANA** - *Protocol for Live Agent Network Automation*: a typed
application-layer protocol for real-time state synchronization and control
between a client shell and a backend service runtime, built on JSON-RPC 2.0
(the way HTTP is built on TCP). Not a general-purpose RPC framework.

This repo is split into four publishable crates:

| Crate | Role in the stack |
|-------|-------------------|
| `plana-protocol-core` | **Generic protocol core** - handshake and identity negotiation, base protocol messages, health/network descriptors, RBAC, and region policy. Independent of any specific platform domain. |
| `plana-celestia-types` | **Celestia domain profile** - the celestia-island platform's agent, task, panel, industrial and tool domain messages, built on the generic `plana-protocol-core` message set. |
| `plana-jsonrpc` | **Framing & transport base** - JSON-RPC 2.0 correlation, typed method routing, and Unix-socket / HTTP / WebSocket bindings. |
| `plana` | **The protocol layer** - re-exports the core, the domain profile and the framing, and adds server-side session management (SSE, events) behind the `rpc-server` feature. |



## Architecture: generic core + domain profiles

PLANA is layered: the generic protocol core sits at the bottom, a domain
profile plugs its message vocabulary in on top of it, and the umbrella crate
re-exports both for consumers.

```text
plana-protocol-core (generic message set)  ←  plana-celestia-types (domain profile)  ←  plana (umbrella)
      └────────────────────────────────────────── plana-jsonrpc (framing; namespace! macro machinery)
```

- **The generic core** - `plana-protocol-core` owns the platform-independent
  message set: handshake/version/identity negotiation, base protocol messages,
  health and network descriptors, RBAC, and region policy. It knows nothing
  about agents, tasks, panels or any specific platform. The generic JSON-RPC
  2.0 envelope lives with the framing crate (`plana-jsonrpc`) as the single
  canonical definition - a former copy here drifted and was removed.
- **Registering a domain** - two mechanisms, one for the protocol's own
  families and one for third-party profiles:
  - *Built-in method families* declare their namespaces with the `namespace!`
    macro (re-exported as `plana::jsonrpc::namespace`), which generates the
    typed `Method` variants whose wire names are derived from the enum path
    (`Sync.Ping`, `Base.Heartbeat`, …). The `Method` catalog itself is a
    closed set — extending it is a change to the protocol crate.
  - *Third-party profiles* do not fork or patch the protocol: they register
    their own method names directly into `plana::jsonrpc::RpcMethodMap`
    (`RpcMethodMap::empty().method("my.domain.op", handler)`, string-keyed
    dynamic dispatch over HTTP/WS via `rpc_axum_router`), so any
    `"client shell <-> backend runtime"` scenario can speak the same
    JSON-RPC 2.0 framing without the protocol knowing its methods.
- **The celestia profile** - `plana-celestia-types` is the celestia-island
  platform's domain profile, built on `plana-protocol-core`. It ships by
  default (feature `celestia`) because the celestia platform is PLANA's
  primary consumer; other consumers can disable it
  (`default-features = false`) and register their own profiles.
- **Shared module names** - `http` and `enums` exist in both the core and the
  profile; the umbrella merges them under one path (`plana::http::*`), with
  generic and domain types side by side. The full domain surface is also
  reachable at `plana::celestia`.
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
plana-protocol-core = { version = "0.2" }
plana-celestia-types = { version = "0.1" }
plana-jsonrpc = { version = "0.1" }
```

### Registering and calling a JSON-RPC method

Built-in method families are declared with the `namespace!` macro (defined in
`plana-jsonrpc`, see `pending.rs`) and dispatched through the typed `Method`
enum. Third-party method names register directly into `RpcMethodMap` — no
enum extension needed:

```rust
use plana::jsonrpc::{JsonRpcRequest, RpcMethodMap, UnixMethod, serialize_to_jsonrpc};
use serde_json::{Value, json};

fn main() -> Result<(), serde_json::Error> {
    // A typed method name from the declaration site of your domain profile:
    let req = JsonRpcRequest::new(UnixMethod::ToolListTools, None);

    // Serialize to the wire (JSON-RPC 2.0 envelope):
    let json = serialize_to_jsonrpc(&req, false)?;

    // Third-party methods: string-keyed dynamic dispatch, no forking.
    let methods = RpcMethodMap::empty()
        .method("my.domain.op", |params: Value| async move {
            Ok(json!({ "echo": params["text"].clone() }))
        });
    assert!(json.starts_with("{\"jsonrpc\":\"2.0\","));

    // Health/network descriptors from the generic core:
    let health = plana::http::HealthResponse::ok(
        "1.0.0",
        plana::http::BackendKind::Dev,
        42,
        plana::http::NetworkInfo::unknown(),
    );
    serde_json::to_string(&health).map(|_| ())
}
```

### Using the shared health/network wire types

```rust
use plana::http::{BackendKind, HealthResponse, NetworkInfo, ServiceStatus};

fn main() -> Result<(), serde_json::Error> {
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
    serde_json::to_string(&health).map(|_| ())
}
```

### Unix-socket transport

```rust
use std::path::Path;
use plana::jsonrpc::{JsonRpcRequest, TimeoutPolicy, unix_transport::JsonRpcTransport};

let mut transport = JsonRpcTransport::connect(Path::new("/run/my-app/rpc.sock")).await?;
let response = transport.send(&JsonRpcRequest::new_raw("ping", None), TimeoutPolicy::Default).await?;
```

## Feature flags

| Feature | Default | What it enables |
|---------|---------|-----------------|
| `celestia` | yes | `plana-celestia-types` re-exported at the crate root and at `plana::celestia`; `plana-protocol-core` is always re-exported at the root. |
| `rpc-server` | no | Server-side SSE event streaming and request network/geo detection (`rpc_server::detect_network`). Transport sessions for SSE live in `plana::jsonrpc::session`. |
| `jsonrpc` | no | No-op compatibility feature — `plana-jsonrpc` is an always-on dependency now, so there is nothing to gate. Kept so consumers that declare it keep resolving. |
| `tracing-helpers` | no | Forwards `plana-protocol-core/tracing-helpers`: `plana::tracing_helpers` module with a `ShortTimer` formatting type. |

### Consumer migration (post re-pin)

The feature surface of `plana` changed when the generic core was extracted:

- `jsonrpc` — no-op now (plana-jsonrpc is always on). A no-op compat feature is
  kept so existing declarations keep resolving; do not rely on it to disable
  anything.
- `types` — renamed to `celestia`. The domain types now live in
  `plana-celestia-types`; enable `celestia` for the domain vocabulary.
- `tracing-helpers` — forwarded to `plana-protocol-core/tracing-helpers`.

After re-pinning, declare the following features:

| Consumer | Features to declare |
|----------|---------------------|
| arona | `rpc-server` |
| shittim-chest | `jsonrpc`, `tracing-helpers`, `celestia` |
| evernight | `jsonrpc` |
| entelecheia | `rpc-server`, `celestia` (aspirational — see note below) |
| scriptum | `jsonrpc` (aspirational — see note below) |

> **entelecheia and scriptum rows are re-pin guidance, not current fact.**
> entelecheia's manifest still pins the *dead* `plana-types` crate name
> (`plana = { package = "plana-types", git = "…/plana", tag = "v0.1.9" }`),
> and scriptum's Cargo.toml still pins the dead `plana-types` crate name as a
> git dependency on live `master`; neither has been migrated to the split
> `plana`/`plana-celestia-types` crates yet. The rows above are the feature
> sets they should declare once re-pinned.

## TypeScript bindings

Generated TS bindings live in **two** places, both regenerated by
`just gen bindings`:

- `packages/protocol-core/bindings/` — the generic core types (health,
  RBAC, handshake, region policy, base messages). The JSON-RPC envelope
  (owned by `plana-jsonrpc`) is Rust-only and has no TypeScript export.
  In-repo for now;
  an npm home for the generic bindings package is a follow-up decision.
- `packages/celestia-types/bindings/` — the celestia domain profile types
  (agent, task, industrial, tool, malkuth supervision, …), shipped as
  `@celestia-island/plana-celestia-types`.

The generic types (e.g. `HealthResponse`, `RbacUser`, `ConnectionStatus`,
`BaseHeartbeatParams`, `RegionPolicy`) are exported **only** from
`packages/protocol-core/bindings/`; the celestia package ships domain types
only.

> **`HealthDetailed` was removed in the core split** (dead in-repo, unused by
> in-repo consumers). If you consumed it from the old
> `@celestia-island/plana-types` npm bindings, migrate to the structured
> health fields (`HealthResponse` / `ServiceStatus` / `ConnectionStatus`).

## Stability

Pre-1.0: the API is subject to change. The wire formats (serde layouts,
method naming conventions) are treated carefully — changes that affect
on-the-wire compatibility are always considered breaking.

## Not a general-purpose RPC framework

`plana` is the scaffold for one specific typed bidirectional sync protocol,
not a general-purpose RPC framework. The wire types and JSON-RPC machinery
are shaped by that protocol's needs; for a generic RPC stack, look at
established frameworks instead.
