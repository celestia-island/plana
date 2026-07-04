# arona — 项目状态与计划 (PLAN)

> 本文件由自动化扫描于 **2026-07-04** 生成，记录项目当前状态、近期进展与后续计划。
> 最近一次手动刷新：**2026-07-04**（审计并清理 Vision/MediaPipe 等 aspirational stub 类型，见 R11）。
> 原有详细计划已保留于文末「既有详细计划（存档）」。

## 1. 项目概述

- **名称**：`arona`
- **简介**：celestia-island 共享协议类型库，含 TypeScript 绑定与构建脚本。
- **远程仓库**：https://github.com/celestia-island/arona.git
- **技术栈**：Rust / Node/TypeScript / just
- **类别**：rust-lib

## 2. 当前状态

- **当前分支**：`dev`
- **工作区**：干净（此前未提交的 `src/protocol/jsonrpc.rs` 改动已随 `8dcfdbf` 入库）
- **最近提交时间**：2026-07-04
- **最近提交**：8dcfdbf style: rustfmt jsonrpc notification fallback
- **分支对比**：`dev` 领先 `master` 127 个提交

## 3. 发布元数据补全（本次刷新完成）

本次刷新补全了面向 crates.io 的发布元数据，已通过 `cargo check` / `cargo clippy --all-features -- -D warnings` / `cargo test --lib`（651 项通过）/ `cargo fmt --check`：

- `README.md`：新增官方 docs.rs 徽章（`https://docs.rs/arona/badge.svg`）。
- `Cargo.toml`：补充 `keywords`、`categories`，新增 `[package.metadata.docs.rs]`（`all-features = true`）。
- 关键词：`json-rpc`、`protocol`、`mcp`、`typescript`、`schema`。
- 分类：`api-bindings`、`data-structures`、`web-programming::websocket`、`network-programming`。

## 4. 近期进展（最近提交）

- refactor: audit and clean up aspirational stub types（移除 `ModelCategory::Vision` / `ModelServerKind::MediaPipe`，见 R11）
- style: rustfmt jsonrpc notification fallback
- docs: add PLAN.md current-status snapshot
- feat(model): add ModelCapability enum, extend ModelCategory, add GenerationTier
- docs: simplify description
- chore: normalize dependency versions to caret (^) prefix
- chore: add CI workflow, rust-toolchain pin, relax schemars dep, update README/PLAN

## 5. 后续计划

1. ~~整理并提交当前未提交改动~~ — 已完成（`src/protocol/jsonrpc.rs` 随 `8dcfdbf` 入库）。
2. ~~完善 `crates.io` 发布元数据（rust-version / metadata / docs.rs badge）~~ — 已完成（`rust-version` 既有 `1.85`，本次补齐 keywords/categories 与 `[package.metadata.docs.rs]`，README 已加 docs.rs badge）。
3. 补充单元/集成测试，保持 `just test` 与 clippy `-D warnings` 通过。
4. 定期刷新本 PLAN.md 以反映最新状态。

---

## 既有详细计划（存档）

# arona — Issues & Action Plan

Generated 2026-06-30 from deep code audit. Updated 2026-07-02 (R3). Updated 2026-07-03 (R4). Updated 2026-07-03 (R5). Updated 2026-07-03 (R6). Updated 2026-07-03 (R7). Updated 2026-07-03 (R8). Updated 2026-07-03 (R9). Updated 2026-07-03 (R10).

arona is the shared protocol crate (v0.1.0) that glues entelecheia and shittim-chest via JSON-RPC types and TypeScript bindings. It also serves as the ecosystem documentation hub.

## Resolved (R4)

### 1. `PROTOCOL_VERSION` constant is unused — RESOLVED (R4)
- Defined in `lib.rs` but never referenced. Retained as a canonical declaration of the platform's advertised protocol version.

### 2. Schemars coverage gaps — RESOLVED (R4)
- Added `JsonSchema` derive to: `AgentErrorCode`, `StreamSegment`, `LlmStream`, `RouteInfo`, `StructuredAgentError`, `PeriodType`, `EmbeddingModel`, `YoloTaskTier`.
- Updated `examples/schema_dump.rs` with all types that carry `JsonSchema`.

### 3. Agent enum aspirational variants removed — RESOLVED (R4)
- Removed `ClassicSoftwareEngineering`, `WebUiPanel`, `IndustrialIoT`, `RemoteOperations` — none had corresponding MCP type modules or agent implementations.
- Agent enum now has 13 variants, all backed by `mcp/` modules.

### 4. Handshake version negotiation — RESOLVED (R3)
- `ConnectHandshakeParams` carries `protocol_version: u32` (default v1), satisfying the version-gating requirement.

## Resolved (R5)

### 5. All types are data-only — logic divergence risk — RESOLVED (R5)
- Added shared validation/categorization impl blocks on core types:
  - `Agent::all()` — returns all 13 variants for iteration
  - `ReportType::is_query()`, `is_error()`, `is_pending()`, `is_terminal()` — wire-level classification both sides can replicate
  - `AgentErrorCode::is_llm_error()`, `is_cosmos_error()`, `is_chain_error()`, `is_skill_error()`, `is_model_selection_error()` — error categorization for routing/retry logic
- These functions serve as documentation contracts for behavior that both entelecheia and shittim-chest must agree on.

### 6. JSON-RPC `Id` UUID v7 format agreement — RESOLVED (R5)
- Added doc comment on `Id::new_uuid()` stating UUID v7 is the canonical wire format for the platform.
- Added `Id::new_uuid_v4()` as an opt-in alternative for consumers that prefer random UUIDs.
- TypeScript consumers should use a UUID v7 library for format parity or treat IDs as opaque sortable strings.
- Existing test `id_new_uuid_is_v7_shaped_string` verifies the v7 format.

## Resolved (R6)

### 7. `ExternalToolInfo` missing serialization traits — RESOLVED (R6)
- `ExternalToolInfo` (external_mcp.rs) is a public type representing tool info from external MCP servers, but lacked `Serialize`, `Deserialize`, and `PartialEq` derives.
- Added all three derives so consumers can serialize, deserialize, and compare tool info from external MCP servers.

### 8. `build_notification` ultimate fallback produces invalid JSON-RPC — RESOLVED (R6)
- The absolute `.unwrap_or_else` fallback in `build_notification` produced `{"method":""}` — an empty method string, which violates JSON-RPC 2.0 spec (method MUST be a non-empty string).
- Fixed the fallback to use `"internal.fallback"` as a sentinel method name, ensuring all fallback paths produce valid JSON-RPC messages.

## Resolved (R7)

### 9. R6 fix incomplete — `build_notification` and `build_notification_value` first-level fallbacks still produce potentially-invalid JSON-RPC — RESOLVED (R7)
- R6 #8 only fixed the innermost/ultimate `.unwrap_or_else` fallback. The first-level fallback in both `build_notification` and `build_notification_value` still passed through the raw `method` argument, which could be an empty string (`{"method":""}` — violates JSON-RPC 2.0).
- Fixed both first-level fallbacks to use `"internal.fallback"` as the sentinel method name, matching the ultimate fallback.
- Simplified `build_notification` from 3 levels to 2 (now only: struct serialization → stringified safe JSON), since two consecutive serialization fallbacks were redundant.

## Resolved (R8)

### 10. R7 fix still incomplete — `build_notification`/`build_notification_value` happy path emits invalid JSON-RPC with empty method — RESOLVED (R8)
- R7 #9 only guarded the *fallback paths* against empty/whitespace method strings. If the method is empty and serialization *succeeds* (the happy path), the functions still produce `{"method":""}` — violating JSON-RPC 2.0 (method MUST be non-empty).
- Added explicit method validation at the top of both `build_notification` and `build_notification_value`: if `method.trim().is_empty()`, substitute `"internal.fallback"` before constructing the notification. This closes the final gap in the R6→R7→R8 chain.

## Resolved (R9)

### 11. `JsonRpcMessage::deserialize` misclassifies notification with explicit `"id": null` — RESOLVED (R9)
- JSON-RPC 2.0 allows `id` to be `null` (explicit null, not absent). The classifier used `value.get("id").is_some()` which returns `true` for `"id": null` (key exists), routing `{"jsonrpc":"2.0","id":null,"method":"test"}` to Request instead of Notification.
- Since `"id": null` semantically means "no response expected" (notification behavior), this misclassification breaks downstream handling.
- Fixed `has_id` check to use `value.get("id").map(|v| !v.is_null()).unwrap_or(false)` — treats `null` id as "no id".
- Added test `message_classifies_explicit_null_id_as_notification` to guard against regression.

## Resolved (R10)

### 12. R9 fix rejects spec-mandated `"id": null` responses — RESOLVED (R10)
- R9 made `has_id` return `false` for `"id": null`, which is correct for *request/notification* discrimination (null id = no id = notification). But the same `has_id` gated the *response* branch (`(has_result || has_error) && has_id`), so a Response carrying `"id": null` was rejected as "cannot classify".
- JSON-RPC 2.0 §5.1 mandates `"id": null` on error responses to requests whose id could not be detected (Parse error / Invalid Request). Rejecting them is a protocol bug — relevant for external MCP-server interop, where a malformed inbound request yields exactly such a response.
- Orthogonal to R9: R9 governs method-message routing; responses are identified by `result`/`error`. Dropped `&& has_id` from the response condition (the request branch still requires a non-null id, so R9's notification routing is unchanged).
- Added test `message_classifies_null_id_error_response`; all prior JSON-RPC tests (R9 notification, ambiguous-payload priority, unclassifiable rejection) still pass.

## Resolved (R11)

### 13. Vision / MediaPipe aspirational stub types removed — RESOLVED (R11)
- evernight 审计报告 Medium #4 指出 `model.rs` 中 `ModelCategory::Vision` 与 `ModelServerKind::MediaPipe` 被标注为 stub，属于 aspirational 占位类型。
- 审计结论：两者均为「全息 AR / MediaPipe 姿态识别」这一未实现特性预留，无任何实际实现——evernight（唯一部署方）自身的 `ModelServerKind` 仅含 `Ollama`/`WhisperCpp`/`Vllm`，并无 `MediaPipe`；entelecheia / shittim-chest / evernight 均未导入 `arona::model` 的类型（消费方各自持有镜像类型，或仅在 shittim-chest 的 `mock-mode` 开发桩里以裸 JSON 字符串形式出现）。
- 已实现的「视觉」能力（MediaFlow 视觉评审，glm-5v-turbo）由 `ModelCategory::MultiModal` + `ModelCapability::{ImageInput, VideoInput, VisualCritique}` 表达，不在本次清理范围内。
- 处理：从公开 API 中移除 `ModelCategory::Vision` 与 `ModelServerKind::MediaPipe` 两个枚举变体，并同步更新模块级文档表格与 `bindings/model.ts`（`cargo test` 自动再生成）。沿用 R4「移除 Agent 枚举 aspirational 变体」的既定惯例。
- 验证：`cargo check --all-features`、`cargo clippy --all-features -- -D warnings`、`cargo test --all-features`（651 项通过）均通过。
- 关联：evernight 仓库 PLAN.md Medium #4（本仓库内完成，evernight 侧无需改动）。

## Strengths (for reference)
- Excellent documentation: 8 language translations, architecture/design/guides/meta docs
- TypeScript bindings auto-generated via `ts-rs` — full-stack type safety
- Clean separation: domain vocab enums, WS message params, HTTP API types, JSON-RPC core
- Consistent serde rename conventions with backward-compatible `Option` fields
- 642 auto-generated TS binding ser/de tests + hand-written unit tests for JSON-RPC and TOML parsing
- `JsonSchema` derive on all core enum/struct types for schema-aware consumers

