+++
title = "Справочник Кодов Ошибок"
description = """> ВАЖНО: «Коды ошибок», документированные ниже (например, `DB_CONNECT_FAILED`,"""
lang = "ru"
category = "design"
subcategory = "core"
+++

# Справочник Кодов Ошибок

> **ВАЖНО**: «Коды ошибок», документированные ниже (например, `DB_CONNECT_FAILED`,
> `LLM_CALL_FAILED`), являются **строковыми шаблонами сообщений**, извлечёнными из
> исходного кода — это неформальные удобные метки, а не формальная
> таксономия ошибок. **Авторитетные структурированные типы ошибок** — это перечисления, определённые
> в [`packages/shared/core/src/errors.rs`](../packages/shared/core/src/errors.rs):
> `AgentErrorCode` (строка 8), `StructuredAgentError`, `CoreError`, `CredentialError`,
> `PromptLoadError`, `SoulLoadError` и их варианты. При интеграции или
> программной отчётности об ошибках используйте эти перечисления, а не строковые шаблоны,
> перечисленные здесь.

Этот документ каталогизирует шаблоны ошибок, используемые в кодовой базе Entelecheia на Rust.
Структурированные коды ошибок находятся в разработке; большинство ошибок в настоящее время используют `anyhow`/`thiserror`
с описательными сообщениями.

## Категории Ошибок

### База Данных (`DB_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `DB_CONNECT_FAILED` | `database connection failed: {}` | `scepter/src/app/setup.rs` |
| `DB_MIGRATE_FAILED` | `database migration failed: {}` | `scepter/src/app/setup.rs` |
| `DB_TABLE_CHECK_FAILED` | `failed to check table existence: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_INIT_SCRIPT_FAILED` | `failed to execute initialization script: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_SAVE_FAILED` | `failed to save agent info: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_UPDATE_FAILED` | `failed to update agent status: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_LOG_FAILED` | `failed to log entry: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_CLEANUP_FAILED` | `failed to clean up old logs: {}` | `packages/shared/infra_services/src/persistence.rs` |

### Конфигурация (`CFG_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `CFG_CREDENTIAL_INIT_FAILED` | `credential storage initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_PROVIDER_INIT_FAILED` | `provider config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_MODEL_INIT_FAILED` | `model config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_USER_INIT_FAILED` | `user config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_KEY_STORE_INIT_FAILED` | `key storage service initialization failed: {}` | `scepter/src/app/setup.rs` |

### Состояние (`ST_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `ST_SERIALIZE_FAILED` | `state serialization failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_WRITE_FAILED` | `temp file write failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_READ_FAILED` | `state file read failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_PARSE_FAILED` | `state file parse failed: {}` | `scepter/src/state/state_persistence.rs` |

### WebSocket (`WS_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `WS_SEND_FAILED` | `failed to send message: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_TIMEOUT` | `response wait timeout or channel closed` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_PARSE_FAILED` | `failed to parse agent list: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_NOT_CONNECTED` | `websocket connection not established` | `packages/shared/infra_services/src/ws_transport.rs` |

### Агент (`AG_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `AG_CONNECT_FAILED` | `connection failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_SEND_FAILED` | `send failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_CHANNEL_NOT_INIT` | `send channel not initialized` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_REGISTRATION_FAILED` | `failed to send registration message: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_RE_REGISTER_FAILED` | `internal agent re-registration failed` | `scepter/src/state/state_restoration.rs` |

### LLM (`LLM_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `LLM_CALL_FAILED` | `LLM call failed: {}` | `scepter/src/state_machine/llm_chat/chat_loop.rs` |

### Агенты Уровня 2 (`L2_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `L2_INIT_FAILED` | `layer2 agent config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `L2_SKILLS_VALIDATE_FAILED` | `layer2 agent skills validation failed: {}` | `scepter/src/app/setup.rs` |

### Навыки (`SK_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `SK_PROMPT_LOAD_FAILED` | `prompt loader error` | `packages/shared/prompt/src/prompt_loader.rs` |
| `SK_TOML_PARSE_FAILED` | `TOML parse failed: {}` | `packages/shared/prompt/src/prompt_loader.rs` |

### Время Выполнения (`RT_*`)

| Код | Шаблон Сообщения | Источник |
| --- | --- | --- |
| `RT_ARC_UNWRAP_DOMAIN` | `Arc::try_unwrap failed for llm_domain/agent_domain` | `scepter/src/state_machine/mod.rs` |
| `RT_UNDO_NO_ACTIVE_SKILL` | `active_streaming_skill is None, defaulting to HubRis` | `scepter/src/state_machine/mod.rs` |

---

> **Примечание**: Это каталог, составленный по мере возможности. Entelecheia мигрирует к
> структурированным типам ошибок с уникальными кодами. Вклад в расширение этого
> справочника приветствуется.
