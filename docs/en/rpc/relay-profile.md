# The PLANA relay extension — chains over the service profile

Status: **active** (core in `plana-rpc-client::relay` + JS twin in
`@celestia-island/plana-rpc-client`; Tauri battery in `plana-tauri`.
中文版：[relay-profile.md](../../zh-Hans/rpc/relay-profile.md))

This document extends — never replaces —
[the service profile](service-profile.md). The service profile defines
one hop: a client talking to a server. The relay extension defines what
happens when parties must communicate **through** one another: a webview
through its host process, a host through a gateway, a gateway through
the next relay. Chains of any length compose; the profile below is the
single-hop contract every link implements.

## 1. Topology

A chain A₀ → A₁ → … → Aₙ (n ≥ 2). Roles are positional:

- **Edge** (A₀): the UI/client — webview, egui surface, CLI. Speaks pure
  JSON-RPC and addresses the far end *as if direct* (reverse-proxy
  illusion).
- **Relay** (any interior Aᵢ): a host process — Tauri backend, egui
  host, daemon, gateway. Each relay independently routes every frame:
  local / forward / deny.
- **Upstream** (Aₙ): a service-profile server — **or another relay**.
  That substitution is what makes 4-, 5- and N-party chains work: the
  extension standardizes only the hop; chains compose.

## 2. The two channels

**Edge channel** (any adjacent pair): exactly two transport functions —
`bridge_call(method, params, id?, relay?)` for request/response and one
notification lane `bridge_event {method, params}`. Frames are canonical
JSON-RPC 2.0 (`plana::jsonrpc`), identical in shape on edge and upstream
hops, so a frame stays inspectable end to end. The channel is
deliberately transport-agnostic: Tauri invoke/events, egui channels,
stdio and in-process connections are interchangeable adapters over one
contract (`EdgeTransport` in the Rust core, same-named interface in the
JS twin).

**Upstream channel** (relay → server): the service profile verbatim —
WebSocket-primary + HTTP fallback, heartbeat, close codes. The relay
extension adds no server-side grammar.

## 3. Routing — per hop, three outcomes

Every inbound frame routes by its method's leading dotted namespace:

1. **Local** — the `relay` namespace (reserved) or an app-mounted local
   namespace (e.g. `flasher.*` privileged disk I/O, which never rides a
   forwarding path).
2. **Forward** — longest-prefix **whitelist** match maps the namespace
   onto a named endpoint opened via `relay.conn.open`. Ties resolve to
   the latest edit.
3. **Deny** — unlisted namespaces answer `-32601`. Relays never guess an
   endpoint.

Together these are the reverse proxy (the edge sees the server's method
surface unchanged) and the forward proxy (the relay applies its own
whitelist and proxy policy to outbound traffic) the extension is named
for.

## 4. The reserved `relay.*` namespace

Structurally unforwardable: a routing table rejects any entry whose
first segment is `relay`, so no configuration edit can pull
system-control methods onto a server connection. Pre-standardized
surface (mount what applies; latest mount wins):

| Method | Purpose |
|---|---|
| `relay.net.set_proxy` | Outbound proxy — `scheme` (`http`/`https`/`socks5`), `host`, optional `username`/`password`; empty host = explicitly direct. |
| `relay.net.configure_entrypoint` | Standard login-handshake entrypoint `{url}` — the public front door; inner-layer forwarding is the server's job. |
| `relay.conn.open` | Open/reuse the WebSocket or HTTP long-poll connection to an endpoint; returns the handle forwarding maps onto. |
| `relay.fwd.map` | Add one whitelist entry `{namespacePrefix, endpoint}`. |
| `relay.fwd.list` | List endpoints and the whitelist. |
| `relay.window.minimize` / `relay.window.maximize_toggle` / `relay.window.close` | Host window controls (any host framework implements). |
| `relay.window.state` | Window label + maximized flag for chrome syncing. |

This is why an app's first-run surface is exactly *entrypoint + proxy*:
every other connection fact derives from these primitives.

## 5. Multi-hop envelope

- End-to-end correlation: the JSON-RPC `id` passes verbatim through all
  hops.
- Loop guard: the extension member `relay: {hops, via?}` counts hops;
  each forwarding relay increments it and refuses (with an error) any
  frame already at its limit (default 4). `via` is a diagnostic trace,
  never load-bearing.
- No hop knows its absolute position — each applies the same contract.

## 6. Proxy policy

Per hop, one decision: relay settings override → `PLANA_PROXY` /
`PLANA_PROXY_SCHEME` / `PLANA_PROXY_USERNAME` / `PLANA_PROXY_PASSWORD`
environment family → **direct**. Ambient `HTTP(S)_PROXY` is never
honored: traffic policy is explicit configuration. Every dialer on the
hop (WebSocket and HTTP alike) consumes the same decision.

## 7. Packaging

- Protocol core (framework-agnostic): `plana-rpc-client::relay` on
  crates.io and the matching module of
  `@celestia-island/plana-rpc-client` on npm — both ride the packages'
  existing release pipelines.
- Tauri adapter + standard battery (`bridge_call` command, event lane,
  ready-mounted handlers incl. window controls): `plana-tauri`
  (crates.io; `bridge` feature default, droppable for battery-only
  hosts).
- Framework adapters beyond Tauri (egui, stdio) live with their hosts;
  they implement `EdgeTransport` and, where applicable, the same
  battery hooks.

## 8. Conformance notes

- A relay MUST answer every request exactly once (id correlation).
- A relay MUST deny rather than guess (`-32601` for unroutable frames).
- A relay MUST NOT forward `relay.*` methods under any configuration.
- A forwarding relay MUST enforce the hop limit.
- Edge envelopes MAY carry the `relay` extension member; servers on the
  service profile ignore unknown members, so upstream strictness is
  unaffected.
