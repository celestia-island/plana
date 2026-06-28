+++
title = "错误码参考"
description = """> 重要提示：下面记录的"错误码"（例如 `DB_CONNECT_FAILED`、"""
lang = "zhs"
category = "design"
subcategory = "core"
+++

# 错误码参考

> **重要提示**：下面记录的"错误码"（例如 `DB_CONNECT_FAILED`、
> `LLM_CALL_FAILED`）是从源代码中提取的**基于字符串的消息模式**
> ——它们是非正式便利标签，并非正式的错误分类。**权威的结构化错误类型**是
> [`packages/shared/core/src/errors.rs`](../packages/shared/core/src/errors.rs) 中定义的枚举：
> `AgentErrorCode`（第 8 行）、`StructuredAgentError`、`CoreError`、`CredentialError`、
> `PromptLoadError`、`SoulLoadError` 及其变体。在以编程方式集成或
> 报告错误时，请使用这些枚举，而非此处列出的字符串模式。

本文档编目了 Entelecheia Rust 代码库中使用的错误模式。
结构化错误码正在开发中；大多数错误当前使用 `anyhow`/`thiserror`
配合描述性消息。

## 错误类别

### 数据库（`DB_*`）

| 代码 | 消息模式 | 来源 |
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

| 代码 | 消息模式 | 来源 |
| --- | --- | --- |
| `CFG_CREDENTIAL_INIT_FAILED` | `credential storage initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_PROVIDER_INIT_FAILED` | `provider config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_MODEL_INIT_FAILED` | `model config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_USER_INIT_FAILED` | `user config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_KEY_STORE_INIT_FAILED` | `key storage service initialization failed: {}` | `scepter/src/app/setup.rs` |

### 状态（`ST_*`）

| 代码 | 消息模式 | 来源 |
| --- | --- | --- |
| `ST_SERIALIZE_FAILED` | `state serialization failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_WRITE_FAILED` | `temp file write failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_READ_FAILED` | `state file read failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_PARSE_FAILED` | `state file parse failed: {}` | `scepter/src/state/state_persistence.rs` |

### WebSocket（`WS_*`）

| 代码 | 消息模式 | 来源 |
| --- | --- | --- |
| `WS_SEND_FAILED` | `failed to send message: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_TIMEOUT` | `response wait timeout or channel closed` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_PARSE_FAILED` | `failed to parse agent list: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_NOT_CONNECTED` | `websocket connection not established` | `packages/shared/infra_services/src/ws_transport.rs` |

### Agent（`AG_*`）

| 代码 | 消息模式 | 来源 |
| --- | --- | --- |
| `AG_CONNECT_FAILED` | `connection failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_SEND_FAILED` | `send failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_CHANNEL_NOT_INIT` | `send channel not initialized` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_REGISTRATION_FAILED` | `failed to send registration message: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_RE_REGISTER_FAILED` | `internal agent re-registration failed` | `scepter/src/state/state_restoration.rs` |

### LLM（`LLM_*`）

| 代码 | 消息模式 | 来源 |
| --- | --- | --- |
| `LLM_CALL_FAILED` | `LLM call failed: {}` | `scepter/src/state_machine/llm_chat/chat_loop.rs` |

### Layer2 Agent（`L2_*`）

| 代码 | 消息模式 | 来源 |
| --- | --- | --- |
| `L2_INIT_FAILED` | `layer2 agent config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `L2_SKILLS_VALIDATE_FAILED` | `layer2 agent skills validation failed: {}` | `scepter/src/app/setup.rs` |

### 技能（`SK_*`）

| 代码 | 消息模式 | 来源 |
| --- | --- | --- |
| `SK_PROMPT_LOAD_FAILED` | `prompt loader error` | `packages/shared/prompt/src/prompt_loader.rs` |
| `SK_TOML_PARSE_FAILED` | `TOML parse failed: {}` | `packages/shared/prompt/src/prompt_loader.rs` |

### 运行时（`RT_*`）

| 代码 | 消息模式 | 来源 |
| --- | --- | --- |
| `RT_ARC_UNWRAP_DOMAIN` | `Arc::try_unwrap failed for llm_domain/agent_domain` | `scepter/src/state_machine/mod.rs` |
| `RT_UNDO_NO_ACTIVE_SKILL` | `active_streaming_skill is None, defaulting to HubRis` | `scepter/src/state_machine/mod.rs` |

---

> **注意**：这是一个尽最大努力的编目。Entelecheia 正在向带有
> 唯一码的结构化错误类型迁移。欢迎贡献来扩展此
> 参考。
