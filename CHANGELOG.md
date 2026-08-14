# Changelog

All notable changes to plana, the Celestia Island shared infrastructure and
protocol types, are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version lines

plana carries two independent version lines:

- **Rust crates (`plana-*`)** — the `0.1.x` line, tracked by the cargo
  workspace version. Master is at `0.1.15`, which is the version the entries
  below document. Git tags `v0.1.2` … `v0.1.16` mark releases; `v0.1.16` was
  cut on a commit that still carried `0.1.15`, so the workspace baseline
  remains `0.1.15`.
- **npm packages** — independently versioned, not a single line:
  `@celestia-island/plana-celestia-types` 0.1.0,
  `@celestia-island/plana-rpc-client` 0.1.3.

## [Unreleased]

### Removed

- `packages/ui` (`@celestia-island/plana-ui`) is retired. All components moved
  to hikari (generic admin shell, auth card, theme/locale pickers, status bar,
  admin table, vite plugins, pow utils) and shittim-chest (business-coupled
  components). erp.celestia.world and e.celestia.world now consume hikari only.

## [0.1.15] - 2026-08-14

### Added

- Add positional markers (`head` / `tail` / `any`) to image content via
  `LlmImageContent.position`, with a `with_position` builder. (#132)

### Changed

- Bump the workspace to 0.1.15, carrying the positional image wire surface. (#133)

### Fixed

- Remove an unused `schemars::JsonSchema` import that broke the clippy gate. (#130)

## [0.1.14] - 2026-08-04

### Added

- Add a receive-timeout guard (`ENGINE_BINARY_RECEIVE_TIMEOUT_SECS = 60`) and
  document resynchronisation and failure handling for binary transfers. (#129)
- Add an optional `refreshToken` callback so the RPC client refreshes once on
  401 before declaring auth lost. (#128)

### Changed

- Send the WS JWT as a `?token=` URL query parameter instead of a subprotocol,
  so the WS tier can actually connect. (#128)

### Fixed

- Fall back to a local random id when `crypto.randomUUID` is unavailable in
  non-secure contexts. (#127)

## [0.1.13] - 2026-08-03

### Added

- Add announced binary-frame transfer to CEP: a `BinaryStart` announcement,
  chunked raw WS binary frames (≤ 256 KiB), and `BinaryEnd`/`BinaryAbort`
  notifications with MIME and optional SHA-256 checksum. (#126)

### Changed

- Bump `ENGINE_PROTOCOL_VERSION` 2 → 3. (#126)

## [0.1.12] - 2026-08-03

### Added

- Let engines declare capabilities in the `EngineHandshakeResult` ack, so the
  gateway learns modalities and content types before any request. (#125)

### Changed

- Keep the capabilities field optional (skipped when `None`) to stay
  compatible with older engines. (#125)

## [0.1.11] - 2026-08-03

### Added

- Generalize CEP to capability-driven multimodal engines: negotiate
  input/output modalities and content types, and route by declaration. (#124)
- Add multimodal messages (`EngineMessage.content` as content parts) and a
  generic `Engine.Invoke` channel for custom engine operations. (#124)

### Changed

- Bump `ENGINE_PROTOCOL_VERSION` 1 → 2. (#124)

## [0.1.10] - 2026-08-03

### Added

- Add the Celestia Engine Protocol (CEP) interchange types: Handshake,
  Chat/ChatStart streaming, Embeddings, Models, Stats telemetry and Shutdown
  over WebSocket + JSON-RPC 2.0. (#123)
- Export the CEP TypeScript bindings (`engine.ts`, `index.ts`, `JsonValue`)
  via ts-rs. (#123)

Versions before 0.1.10 predate this changelog.
