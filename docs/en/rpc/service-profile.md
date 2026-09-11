# The PLANA service profile — strict WebSocket JSON-RPC 2.0

Status: **draft** (implemented by `plana-rpc-server`, conformance suite in
`packages/rpc-server/tests/conformance.rs`)

This document defines the wire profile that hosted celestia services expose
and that third parties may reimplement. The protocol and the
[`plana-rpc-server`] framework are open; hosted instances (for example
`gateway.celestia.world`) are celestia's own deployments of it.

The profile is a *narrow* dialect of JSON-RPC 2.0: everything a client can
do is a request/response call or the heartbeat notification. There are no
client-initiated notification handlers, no batch input, no server-initiated
streams other than handler-pushed notifications on an established
connection.

## 1. Envelope

- The envelope is JSON-RPC 2.0 as canonically defined in
  `plana::jsonrpc` (`packages/plana/src/jsonrpc/types.rs`) — the single canonical
  definition across the fleet. `"jsonrpc": "2.0"` is required on every
  frame.
- One JSON object per WebSocket **text** frame. Binary frames are a
  protocol violation (close code `1003`).
- Request `id`: opaque to the server; string-form UUID v7 is the canonical
  client format (time-ordered, lexicographically sortable). Numeric ids are
  legal JSON-RPC and must be echoed verbatim.
- A request with `id` MUST be answered exactly once by a response echoing
  the id (success carries `result`; failure carries `error` and no
  `result`).
- Error responses to unparseable frames carry `"id": null`
  (JSON-RPC 2.0 mandate).

## 2. Transports

| Transport | Address | Notes |
|---|---|---|
| WebSocket | `GET {path}` upgrade | primary; one logical session per connection |
| HTTP POST | same `{path}` | fallback for degraded environments (mirrors `@celestia-island/plana-rpc-client`); single request object in, single response object out; handler notifications are dropped |

HTTP status mapping on the POST fallback: `200` for every dispatched
result (including JSON-RPC application errors), `400` for unparseable
bodies or non-request objects, `401` when the connection auth hook denies,
`429` when the connection cap is reached.

## 3. Allowed HTTP surface (everything else is RPC)

Only these are exempt from the RPC-only rule:

1. **Probes** — `GET /api/health` (engine health body), healthz/readyz/ping,
   network descriptors. Load balancers and malkuth info-landings speak
   HTTP, not RPC.
2. **Credential-forwarding login** — OAuth `authorize`/`callback` browser
   redirects (HTTP 302 flows cannot ride a WS the browser hasn't opened
   yet). The callback hands the SPA a one-shot ticket; the SPA opens the
   socket and mints its session via an RPC (e.g. `rescue.open_session`).
   Credentials never appear in JS-readable bodies or URLs beyond the
   single-use ticket.
3. **Special protocols** — MQTT or other non-HTTP listeners occupy
   separate ports/paths by explicit deployment decision; they are not part
   of this profile.

## 4. Method naming

`lowercase.dotted` namespaces for service RPC (`rescue.open_session`,
`device.register`, `enrollment.mint`). The `Sync.*` / `Base.*` PascalCase
catalog belongs to the entelecheia workspace-sync dialect and is not used
for new service surfaces. The only reserved notification names are
`Base.Heartbeat` / `Base.HeartbeatAck` (kept for client-compatibility with
`plana-rpc-client`'s default protocol).

## 5. Heartbeat and liveness

- Client sends `{"jsonrpc":"2.0","method":"Base.Heartbeat"}` (no id) at
  its cadence (plana-rpc-client default: 15s).
- The server answers `{"jsonrpc":"2.0","method":"Base.HeartbeatAck"}` on
  the **control lane** — a priority queue that overtakes any response
  backlog, so a saturated data lane cannot read as a dead connection.
- Any inbound frame resets the idle timer. No inbound frame within the
  idle window (default 45s ≈ 3× cadence) ⇒ server closes with code
  `4000`.

## 6. Close codes

| Code | Meaning |
|---|---|
| 1000 | normal closure |
| 1003 | binary frame on a text-only profile |
| 1008 | connection-level policy violation |
| 1011 | unexpected server failure |
| 4000 | idle/heartbeat timeout |

## 7. Error codes

Standard JSON-RPC (`-32700/-32600/-32601/-32602/-32603`) and the plana
fleet extension codes (`plana::jsonrpc::error_codes`, notably `-32005`
AUTH_ERROR). This profile adds:

| Code | Meaning |
|---|---|
| `-32050` | connection cap reached (HTTP 429 body) |
| `-32051` | reserved: a dedicated code for a cancelled dispatch. The current implementation answers the stall as `-32603` + `data.stalled=true` (see §8), so clients must detect stalls by `data.stalled`, not by this code |
| `-32052` | deferred op id is unknown: never issued, or evicted by the server's retention cap (`-32052` can therefore arrive **before** the id's window elapses) |
| `-32053` | deferred op id was issued and its validity window has elapsed |
| `-32054` | deferred-op registry at capacity (pending cap reached) |
| `-32055` | a worker honoured `ops.cancel` and abandoned the operation (deferred outcome error) |

Application errors SHOULD carry a stable machine-readable string in
`error.data.code` (e.g. `"quota_exhausted"`) alongside the human message.

## 8. Limits, stalls, cancellation

- Max concurrent connections: 100 by default (scepter parity); excess
  upgrades refused with HTTP 429 + `-32050` before the handshake.
- Frame/message budgets: 1 MiB / 4 MiB by default.
- A handler running longer than the stall limit (default 8s, below the
  client's 10s ack window) is **cancelled** and answered with a structured
  stall error — `-32603` + `data.stalled=true` today (`-32051` in the table
  above is reserved for a dedicated code that no branch emits yet, so detect
  stalls by `data.stalled`). The late result never reaches the wire.
  ⇒ **Handlers must be cancellation-safe.**
- **The stall limit is a liveness guard measured in seconds, never an
  upstream budget.** It kills a wedged or runaway dispatch; it does not
  bound how long an upstream may take, and raising it is not the fix for a
  slow upstream. Any handler whose upstream can outlive it — payment and
  settlement requests, provider callbacks, LLM calls, anything metered or
  state-changing — MUST answer immediately with a deferred op reference
  (§8.1). A stall during a state-changing dispatch is the worst case of
  all: the upstream can complete **after** the dispatch was cancelled, so
  money or quota is spent with nothing returned to the caller.
- Batch input (JSON arrays) is rejected with `-32600` +
  `data.reason="batch_not_supported"` (v1).

### 8.1 Deferred operations

The profile's answer to upstream latency it does not own: a handler answers
**now** and the caller collects later.

- **Immediate answer.** A deferred-capable handler answers
  `{"op_id": "<opaque random id>", "expires_in": <seconds>}`. `op_id` is a
  random UUID — unguessable, never a counter — because anyone holding it can
  collect the outcome.
- **Validity window.** `expires_in` is the **remaining** validity measured
  from creation, and the intended band for client-facing ids is **10–30
  minutes** (default 30). The window is anchored at creation, not at
  settlement, so a client that reconnects inside the window still collects.
- **Collection.** `ops.result {op_id}` → the outcome
  `{"op_id", "status", "method", "expires_in", "cancel_requested", "result"?,
  "error"?}` with `status` one of `pending` / `completed` / `failed`; exactly
  one of `result` / `error` is present once terminal. Collection is
  **non-destructive**: a settled outcome stays collectable until the window
  elapses, so a lost response frame cannot cost the caller the outcome. An id
  that was never issued answers `-32052`, as does one evicted early by the
  server's retention cap; an id whose window elapsed answers `-32053` (and is
  then gone).
- **Cancellation.** `ops.cancel {op_id}` → `{"op_id", "status",
  "cancel_requested"}`. Best-effort by design: it records a flag the worker
  may observe (it needs no scheduler), and a worker that honours it settles
  the operation with `-32055`. Cancelling an already settled operation is a
  valid answer with `cancel_requested: false`.
- **Settlement notification.** `ops.settled {op_id, status}` is pushed on the
  data lane when the originating connection is still open. It is
  **advisory — never the authoritative path**: it carries no payload, is
  enqueued without waiting, and is dropped silently when the connection is
  gone or its data lane is saturated (a slow client must never be able to
  stall the worker that settled the operation). Clients therefore always
  fall back to `ops.result`.
- **Worker failure is still an answer.** A worker that returns an error or
  panics settles the operation as `failed` (`-32603` with a message naming
  the panic); a deferred operation never lingers in `pending` because its
  worker died.
- **Boundness.** A server caps pending operations (`-32054` when the cap is
  reached) and prunes expired entries, so neither the registry nor a
  disconnected client can grow without limit. Two caveats belong to the
  contract: the bound is on **entries**, not bytes (a settled outcome keeps its
  payload until collected or expired), and the cap is **server-global** — a
  caller that wants per-caller fairness uses the request guard. When the
  retention cap is reached the oldest settled entries are evicted, which is the
  one case where an uncollected outcome can vanish before its window elapses
  (the id then answers `-32052`).

Both methods are served by the framework itself
(`plana-rpc-server`'s built-in method map) and are available on the WS and
HTTP POST transports alike; a service may override either name in its own
method map or deny it through the request guard. The wire shapes are the
`plana::jsonrpc::deferred` types; `plana-rpc-client`'s `RpcClient::await_op`
collects by the `ops.settled` notification when it arrives and by polling
`ops.result` otherwise, towards a caller-supplied deadline.
- Client-sent `Response` frames are ignored with a warning.

## 9. Authentication model

Three composable layers, all optional per deployment:

1. **Connection auth** (upgrade-time, HTTP 401 on denial): the hook sees
   the raw headers/URI — bearer tokens, one-shot tickets, mTLS identities.
   Produces an opaque per-connection context.
2. **Request guard** (pre-dispatch, JSON-RPC error on denial): method-level
   authorization against the connection context. Fleet convention:
   `-32005` for auth denials.
3. **Connection-bound sessions** (recommended for strictly-interactive
   services): session handles live in server memory keyed by connection,
   never serialized to the client — no bearer tokens to leak, no tokens in
   URLs. Reconnect ⇒ re-authenticate. Long-lived credentials belong in
   layer 1, not in session handles.

## 10. Conformance

An implementation claiming this profile passes the suite in
`packages/rpc-server/tests/conformance.rs` (14 cases): id echo incl.
UUIDv7, `-32601`/`-32602` mapping, parse error keeps the connection with
`id:null`, batch rejection, heartbeat ack (including control-lane
overtake of a busy data lane), stall cancellation, idle close `4000`,
upgrade refusal 401, guard denial `-32005` without dispatch, HTTP POST
fallback on the same method map, and handler-pushed notifications.

The deferred-operation contract (§8.1) is covered by
`packages/rpc-server/tests/deferred.rs` (12 cases: immediate answer under a
stall limit far below the upstream's duration, repeated collection after
settlement, collection from a second connection and over the HTTP transport,
structured `-32052`/`-32053` and `-32602`, the advisory `ops.settled`
notification, expiry pruning, `-32054` capacity refusal, the `ops.cancel` flag,
a service overriding the built-in collection method, a panicking worker, and the
stall guard itself as a negative control) plus `RpcClient::await_op`'s own suite
in `packages/rpc-client/tests/deferred.rs` (9 cases, including collection
across a reconnect, a settled failure returned as a value, an unreadable
`ops.result` reported as a protocol error, an absurd deadline, and a
degenerate poll cadence that must not become a busy loop).

## 11. Reference implementations

- **TypeScript client** — `@celestia-island/plana-rpc-client` (npm): layered
  transport with WS + HTTP fallback, pluggable heartbeat (default
  `Base.Heartbeat` notify mode), token refresh on 401, exponential backoff
  reconnect.
- **Rust client** — `plana-rpc-client` (this workspace): persistent WS
  connection with id correlation (UUIDv7), per-call timeouts, heartbeat
  watchdog, exponential-backoff reconnect with a fail-fast `Failed` state,
  notification subscriptions, and a one-shot HTTP POST fallback
  (`http::post_rpc`).
- **Rust server** — `plana-rpc-server` (this workspace): the framework and
  conformance suite backing §10.

## 12. Private deployment (supported capability)

The profile is deliberately endpoint-agnostic: **no implementation may
hardwire an official host**. Every reference service derives its
endpoints from configuration, so a fully private fleet (intranet-only
factory, air-gapped lab, on-prem servers) composes its own stack:

- an identity source of its choosing behind the connection-auth hook
  (bearer token, one-shot ticket, mTLS — §9);
- `plana-rpc-server` (or any conformant third-party server) at any
  internal address, fronted by plain TCP or an internal reverse proxy;
- `plana-rpc-client` / `@celestia-island/plana-rpc-client` pointed at it
  by URL — ws://, wss://, http(s)://, raw pod addresses all conform.

Living reference: the celestia demo enrollment gateway — an
`evernight-gateway` instance on an intranet pod, whose chest-token
verifier secret is aligned with the demo chest, fronted by an nginx path
lane; the flasher reaches it via `?token=` carriage with no code
differences from the hosted configuration. See
`evernight/packages/gateway` and
`evernight-appliance/packages/flasher/README.md` for the full
self-hosting configuration surface.
