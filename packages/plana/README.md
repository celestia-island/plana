# plana

**PLANA** - *Protocol for Live Agent Network Automation*: a typed
application-layer protocol for real-time state synchronization and control
between a client shell and a backend service runtime, built on JSON-RPC 2.0
(the way HTTP is built on TCP). Not a general-purpose RPC framework.

The workspace has seven crates without `publish = false`: the protocol
foundation (`plana`), the domain profile (`plana-celestia-types`) and the
service-side crates built on them (`plana-rpc-server`, `plana-rpc-client`,
`plana-tauri`, `plana-evernight-client`, `plana-celestia-config`); every other
member is `publish = false` internal infrastructure. The `v*` release job in
`.github/workflows/publish.yml` currently ships five of them: `plana`,
`plana-rpc-server`, `plana-rpc-client`, `plana-tauri` and
`plana-celestia-types`.

| Crate | Role in the stack |
|-------|-------------------|
| `plana` | **The protocol foundation** - the JSON-RPC 2.0 wire layer (`plana::jsonrpc`) and the generic protocol core (`plana::protocol_core`) live here directly, plus the server-side axum mounting module behind the `rpc-server` feature. |
| `plana-celestia-types` | **Celestia domain profile** - the celestia-island platform's agent, task, panel, industrial and tool domain messages, built on the generic core. It depends on `plana`, never the reverse. |
| `plana-protocol-core` | **Re-export shim** (`publish = false`) - a one-line `pub use plana::protocol_core::*`, kept so git pins naming the former standalone crate keep compiling. |
| `plana-jsonrpc` | **Re-export shim** (`publish = false`) - a one-line `pub use plana::jsonrpc::*`, same reason. |

## Architecture: foundation + domain profiles

PLANA is layered: the foundation crate owns the wire layer and the generic
protocol core, and a domain profile is a *separate crate* that depends on it
and plugs its own message vocabulary in.

```text
plana (foundation: plana::jsonrpc + plana::protocol_core)
  ├── plana-protocol-core / plana-jsonrpc   (re-export shims, not published)
  ├── plana-rpc-server / plana-rpc-client / plana-tauri   (service + client frameworks)
  ├── plana-evernight-client                (terminal-route dispatch client)
  └── plana-celestia-types                  (domain profile; depends on plana)
```

- **The generic core** - `plana::protocol_core` owns the platform-independent
  message set: handshake/version/identity negotiation, base protocol messages,
  health and network descriptors, RBAC, and region policy. It knows nothing
  about agents, tasks, panels or any specific platform. The generic JSON-RPC
  2.0 envelope lives in `plana::jsonrpc` as the single canonical definition -
  a former copy in the core drifted and was removed.
- **Registering a domain** - two mechanisms, one for the protocol's own
  families and one for third-party profiles:
  - *Built-in method families* declare their namespaces with the `namespace!`
    macro (`plana::namespace`, `#[macro_export]`ed at the foundation crate
    root), which generates the typed `Method` variants whose wire names are
    derived from the enum path (`Sync.Ping`, `Base.Heartbeat`, …). The
    `Method` catalog itself is a closed set — extending it is a change to the
    protocol crate.
  - *Third-party profiles* do not fork or patch the protocol: they register
    their own method names directly into `plana::jsonrpc::RpcMethodMap`
    (`RpcMethodMap::empty().method("my.domain.op", handler)`, string-keyed
    dynamic dispatch over HTTP/WS via `rpc_axum_router`), so any
    `"client shell <-> backend runtime"` scenario can speak the same
    JSON-RPC 2.0 framing without the protocol knowing its methods.
- **The celestia profile** - `plana-celestia-types` is the celestia-island
  platform's domain profile, built on the generic core of `plana`. It is its
  own crate, not a feature of the foundation: consumers that need the domain
  vocabulary depend on `plana-celestia-types` directly. The `plana` feature
  named `celestia` is a **no-op compatibility feature** kept only so pins that
  still declare it keep resolving (see [Feature flags](#feature-flags)).
- **Shared module names** - `http` and `enums` exist in both crates, and there
  is no merged umbrella path between them: `plana::http` / `plana::enums` carry
  the generic descriptors only, while the domain DTOs (`AgentItem`,
  `ModelInfo`, `TierDefinition`, …) live under `plana_celestia_types::http` /
  `plana_celestia_types::enums`. `plana::http::AgentItem` does not resolve.
- **Why it matters** - the protocol is usable beyond its origin: any
  "client shell <-> backend runtime" state-synchronization scenario can
  implement its own profile crate without forking the protocol.

## Usage

Add to your `Cargo.toml` — either the foundation crate:

```toml
[dependencies]
plana = { version = "0.2", features = ["rpc-server"] }
```

or the foundation plus the celestia domain profile:

```toml
[dependencies]
plana = "0.2"
plana-celestia-types = "0.1"
```

`plana-jsonrpc` and `plana-protocol-core` are `publish = false` re-export
shims; do not depend on them by version.

### Registering and calling a JSON-RPC method

Built-in method families are declared with the `namespace!` macro (defined in
`plana::jsonrpc::pending`) and dispatched through the typed `Method`
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
| `celestia` | no | **No-op compatibility feature.** The domain profile is the separate `plana-celestia-types` crate, which depends on `plana` (never the reverse), so there is nothing left to gate or re-export. Kept so pins that declare `features = ["celestia"]` keep resolving; `plana::celestia` does not exist. |
| `rpc-server` | no | Server-side SSE event streaming and request network/geo detection (`rpc_server::detect_network`). Transport sessions for SSE live in `plana::jsonrpc::session`. |
| `jsonrpc` | no | No-op compatibility feature — the JSON-RPC layer is an always-on module of this crate now (`plana::jsonrpc`), so there is nothing to gate. Kept so consumers that declare it keep resolving. |
| `tracing-helpers` | no | Enables `plana::tracing_helpers`, the `ShortTimer` formatting type re-exported from `plana::protocol_core::tracing_helpers`. |

`default = []`: no feature is on unless a consumer asks for it.

### Consumer migration (post re-pin)

The feature surface of the foundation changed when the generic core was
extracted and again when the `celestia` facade was removed. Current facts:

- `celestia` — no-op now. It no longer re-exports anything: the domain
  vocabulary is the `plana-celestia-types` crate, so declare that crate
  directly instead of `features = ["celestia"]`.
- `jsonrpc` — no-op now (the JSON-RPC layer is always on). A no-op compat
  feature is kept so existing declarations keep resolving; do not rely on it
  to disable anything.
- `types` — the pre-split name of the domain feature; no such feature exists
  in any form now. The domain vocabulary is the `plana-celestia-types` crate.
- `tracing-helpers` — enables `plana::tracing_helpers`.

Declarations observed in this workspace (verified against each consumer's
`Cargo.toml`; keep them in that shape when re-pinning):

| Consumer | Features to declare on `plana` | Domain types |
|----------|-------------------------------|--------------|
| arona | `rpc-server` | — |
| shittim-chest | `jsonrpc`, `tracing-helpers` | direct `plana-celestia-types` dependency |
| evernight | `jsonrpc` | — (uses the generic `plana::http` descriptors) |
| entelecheia | `rpc-server` | direct `plana-celestia-types` dependency |

## TypeScript bindings

Generated TS bindings live in **two** places, both regenerated by
`just gen bindings`:

- `packages/plana/bindings/` — the generic core types (health, RBAC,
  handshake, region policy, base messages) and the deferred-operation wire
  shapes, generated by the foundation crate itself (the `plana-protocol-core`
  shim owns no types). The JSON-RPC request/response envelope is Rust-only;
  only the error object (`JsonRpcError`) is generated into `bindings/ops.ts`.
- `packages/celestia-types/bindings/` — the celestia domain profile types
  (agent, task, industrial, tool, malkuth supervision, …), shipped as the
  `@celestia-island/plana-types` npm package.

The generic types (e.g. `HealthResponse`, `RbacUser`, `ConnectionStatus`,
`BaseHeartbeatParams`, `RegionPolicy`) are generated **only** into
`packages/plana/bindings/`; `packages/celestia-types/bindings/` additionally
carries two vendored snapshots of the foundation's generated bindings
(`protocol-core-httpTypes.ts` and `ops.ts`) so the published npm tarball is
self-contained.

> **`HealthDetailed` was removed in the core split** (dead in-repo, unused by
> in-repo consumers). If you consumed it from the `@celestia-island/plana-types`
> npm bindings, migrate to the structured health fields (`HealthResponse` /
> `ServiceStatus` / `ConnectionStatus`).

## Stability

Pre-1.0: the API is subject to change. The wire formats (serde layouts,
method naming conventions) are treated carefully — changes that affect
on-the-wire compatibility are always considered breaking.

## Not a general-purpose RPC framework

`plana` is the scaffold for one specific typed bidirectional sync protocol,
not a general-purpose RPC framework. The wire types and JSON-RPC machinery
are shaped by that protocol's needs; for a generic RPC stack, look at
established frameworks instead.
