# arona — Issues & Action Plan

Generated 2026-06-30 from deep code audit.

arona is the shared protocol crate (v0.1.0) that glues entelecheia and shittim-chest via JSON-RPC types and TypeScript bindings. It also serves as the ecosystem documentation hub.

## Critical

### 1. `branch = "dev"` git binding is intentional — do NOT pin to a SHA (won't-do)
- entelecheia's **Cargo.toml**: `arona = { git = "...", branch = "dev" }`
- **Decision (won't-do)**: cross-repo deps resolve **local-first**. A cargo
  `[patch]` (`~/.cargo/config.toml` or the consuming repo's `Cargo.toml`) or an
  env var (`ARONA_ROOT`) overrides the git source to a local checkout; only when
  none of those exist does cargo fall back to the `branch = "dev"` git source.
  Pinning to a commit SHA would contradict this design, so `branch = "dev"` is
  kept on purpose.

## High

### 2. Minimal test coverage
- Only 5 unit tests (in `external_mcp.rs` — TOML parsing)
- No tests for JSON-RPC serialization/deserialization round-trips
- No tests for WebSocket message round-tripping
- No integration tests with entelecheia or shittim-chest consumers
- **Impact**: Protocol drift between Rust and TypeScript consumers could cause silent bugs.
- **Fix**: Add:
  - JSON-RPC message ser/de round-trip tests
  - TypeScript binding generation verification tests
  - Snapshot tests for key type definitions

### 3. No version negotiation in handshake
- `handshake.rs` defines `ConnectHandshake` but has no protocol version field
- Consumers cannot detect incompatibility at connection time
- **Fix**: Add `protocol_version: u32` to handshake and version gating logic.

### 4. All types are data-only — logic divergence risk
- arona defines no traits or behavior, only `#[derive(Serialize, Deserialize, TS, JsonSchema)]`
- Any behavior/logic must be implemented identically in both Rust (entelecheia/scepter) and TypeScript (shittim-chest/webui)
- **Fix**: Consider adding shared validation functions (e.g., `is_valid_agent()`) or documentation contracts about which side owns which logic.

## Medium

### 5. `Agent` enum has 17 variants but some may be aspirational
- Verify that all 17 agent variants correspond to implemented agents
- Remove or document variants that are not yet implemented

### 6. JSON-RPC `Id` supports UUID v7 generation
- **jsonrpc.rs**: `Id::new_v7()` generates UUID v7
- Ensure both sides agree on ID format expectations

## Strengths (for reference)
- Excellent documentation: 8 language translations, architecture/design/guides/meta docs
- TypeScript bindings auto-generated via `ts-rs` — full-stack type safety
- Clean separation: domain vocab enums, WS message params, HTTP API types, JSON-RPC core
- Consistent serde rename conventions with backward-compatible `Option` fields
