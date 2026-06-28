+++
title = "錯誤代碼參考"
description = """> 重要提示：以下記錄的「錯誤代碼」（例如 `DB_CONNECT_FAILED`、"""
lang = "zht"
category = "design"
subcategory = "core"
+++

# 錯誤代碼參考

> **重要提示**：以下記錄的「錯誤代碼」（例如 `DB_CONNECT_FAILED`、
> `LLM_CALL_FAILED`）是從原始碼提取的**基於字串的訊息模式** —
> 它們是非正式的便利標籤，而非正式的錯誤分類法。**權威的結構化錯誤型別**
> 是 [`packages/shared/core/src/errors.rs`](../packages/shared/core/src/errors.rs) 中定義的列舉：
> `AgentErrorCode`（第 8 行）、`StructuredAgentError`、`CoreError`、`CredentialError`、
> `PromptLoadError`、`SoulLoadError` 及其變體。在以程式化方式整合或
> 報告錯誤時，請使用這些列舉，而非此處列出的字串模式。

本文件編目了 Entelecheia Rust 程式碼庫中使用的錯誤模式。
結構化錯誤代碼仍在進行中；大多數錯誤目前使用 `anyhow`/`thiserror`
搭配描述性訊息。

## 錯誤類別

### 資料庫（`DB_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `DB_CONNECT_FAILED` | `database connection failed: {}` | `scepter/src/app/setup.rs` |
| `DB_MIGRATE_FAILED` | `database migration failed: {}` | `scepter/src/app/setup.rs` |
| `DB_TABLE_CHECK_FAILED` | `failed to check table existence: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_INIT_SCRIPT_FAILED` | `failed to execute initialization script: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_SAVE_FAILED` | `failed to save agent info: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_UPDATE_FAILED` | `failed to update agent status: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_LOG_FAILED` | `failed to log entry: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_CLEANUP_FAILED` | `failed to clean up old logs: {}` | `packages/shared/infra_services/src/persistence.rs` |

### 配置（`CFG_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `CFG_CREDENTIAL_INIT_FAILED` | `credential storage initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_PROVIDER_INIT_FAILED` | `provider config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_MODEL_INIT_FAILED` | `model config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_USER_INIT_FAILED` | `user config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_KEY_STORE_INIT_FAILED` | `key storage service initialization failed: {}` | `scepter/src/app/setup.rs` |

### 狀態（`ST_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `ST_SERIALIZE_FAILED` | `state serialization failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_WRITE_FAILED` | `temp file write failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_READ_FAILED` | `state file read failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_PARSE_FAILED` | `state file parse failed: {}` | `scepter/src/state/state_persistence.rs` |

### WebSocket（`WS_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `WS_SEND_FAILED` | `failed to send message: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_TIMEOUT` | `response wait timeout or channel closed` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_PARSE_FAILED` | `failed to parse agent list: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_NOT_CONNECTED` | `websocket connection not established` | `packages/shared/infra_services/src/ws_transport.rs` |

### 代理（`AG_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `AG_CONNECT_FAILED` | `connection failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_SEND_FAILED` | `send failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_CHANNEL_NOT_INIT` | `send channel not initialized` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_REGISTRATION_FAILED` | `failed to send registration message: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_RE_REGISTER_FAILED` | `internal agent re-registration failed` | `scepter/src/state/state_restoration.rs` |

### LLM（`LLM_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `LLM_CALL_FAILED` | `LLM call failed: {}` | `scepter/src/state_machine/llm_chat/chat_loop.rs` |

### 第二層代理（`L2_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `L2_INIT_FAILED` | `layer2 agent config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `L2_SKILLS_VALIDATE_FAILED` | `layer2 agent skills validation failed: {}` | `scepter/src/app/setup.rs` |

### 技能（`SK_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `SK_PROMPT_LOAD_FAILED` | `prompt loader error` | `packages/shared/prompt/src/prompt_loader.rs` |
| `SK_TOML_PARSE_FAILED` | `TOML parse failed: {}` | `packages/shared/prompt/src/prompt_loader.rs` |

### 執行時期（`RT_*`）

| 代碼 | 訊息模式 | 來源 |
| --- | --- | --- |
| `RT_ARC_UNWRAP_DOMAIN` | `Arc::try_unwrap failed for llm_domain/agent_domain` | `scepter/src/state_machine/mod.rs` |
| `RT_UNDO_NO_ACTIVE_SKILL` | `active_streaming_skill is None, defaulting to HubRis` | `scepter/src/state_machine/mod.rs` |

---

> **注意**：這是一個盡力的編目。Entelecheia 正在遷移到
> 具有唯一代碼的結構化錯誤型別。歡迎擴充此
> 參考的貢獻。
