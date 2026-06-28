+++
title = "Referencia de Códigos de Error"
description = """> IMPORTANTE: Los "códigos de error" documentados a continuación (ej. `DB_CONNECT_FAILED`,"""
lang = "es"
category = "design"
subcategory = "core"
+++

# Referencia de Códigos de Error

> **IMPORTANTE**: Los "códigos de error" documentados a continuación (ej. `DB_CONNECT_FAILED`,
> `LLM_CALL_FAILED`) son **patrones de mensaje basados en cadenas** extraídos del
> código fuente — son etiquetas de conveniencia informales, no una taxonomía de
> errores formal. Los **tipos de error estructurados autoritativos** son los enums definidos
> en [`packages/shared/core/src/errors.rs`](../packages/shared/core/src/errors.rs):
> `AgentErrorCode` (línea 8), `StructuredAgentError`, `CoreError`, `CredentialError`,
> `PromptLoadError`, `SoulLoadError` y sus variantes. Al integrar o
> reportar errores programáticamente, use esos enums, no los patrones de cadena
> listados aquí.

Este documento cataloga los patrones de error utilizados en el código base Rust de Entelecheia.
Los códigos de error estructurados están en progreso; la mayoría de los errores actualmente usan `anyhow`/`thiserror`
con mensajes descriptivos.

## Categorías de Error

### Base de Datos (`DB_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `DB_CONNECT_FAILED` | `database connection failed: {}` | `scepter/src/app/setup.rs` |
| `DB_MIGRATE_FAILED` | `database migration failed: {}` | `scepter/src/app/setup.rs` |
| `DB_TABLE_CHECK_FAILED` | `failed to check table existence: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_INIT_SCRIPT_FAILED` | `failed to execute initialization script: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_SAVE_FAILED` | `failed to save agent info: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_UPDATE_FAILED` | `failed to update agent status: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_LOG_FAILED` | `failed to log entry: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_CLEANUP_FAILED` | `failed to clean up old logs: {}` | `packages/shared/infra_services/src/persistence.rs` |

### Configuración (`CFG_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `CFG_CREDENTIAL_INIT_FAILED` | `credential storage initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_PROVIDER_INIT_FAILED` | `provider config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_MODEL_INIT_FAILED` | `model config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_USER_INIT_FAILED` | `user config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_KEY_STORE_INIT_FAILED` | `key storage service initialization failed: {}` | `scepter/src/app/setup.rs` |

### Estado (`ST_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `ST_SERIALIZE_FAILED` | `state serialization failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_WRITE_FAILED` | `temp file write failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_READ_FAILED` | `state file read failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_PARSE_FAILED` | `state file parse failed: {}` | `scepter/src/state/state_persistence.rs` |

### WebSocket (`WS_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `WS_SEND_FAILED` | `failed to send message: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_TIMEOUT` | `response wait timeout or channel closed` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_PARSE_FAILED` | `failed to parse agent list: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_NOT_CONNECTED` | `websocket connection not established` | `packages/shared/infra_services/src/ws_transport.rs` |

### Agente (`AG_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `AG_CONNECT_FAILED` | `connection failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_SEND_FAILED` | `send failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_CHANNEL_NOT_INIT` | `send channel not initialized` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_REGISTRATION_FAILED` | `failed to send registration message: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_RE_REGISTER_FAILED` | `internal agent re-registration failed` | `scepter/src/state/state_restoration.rs` |

### LLM (`LLM_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `LLM_CALL_FAILED` | `LLM call failed: {}` | `scepter/src/state_machine/llm_chat/chat_loop.rs` |

### Agentes de Capa 2 (`L2_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `L2_INIT_FAILED` | `layer2 agent config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `L2_SKILLS_VALIDATE_FAILED` | `layer2 agent skills validation failed: {}` | `scepter/src/app/setup.rs` |

### Habilidades (`SK_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `SK_PROMPT_LOAD_FAILED` | `prompt loader error` | `packages/shared/prompt/src/prompt_loader.rs` |
| `SK_TOML_PARSE_FAILED` | `TOML parse failed: {}` | `packages/shared/prompt/src/prompt_loader.rs` |

### Runtime (`RT_*`)

| Código | Patrón de Mensaje | Origen |
| --- | --- | --- |
| `RT_ARC_UNWRAP_DOMAIN` | `Arc::try_unwrap failed for llm_domain/agent_domain` | `scepter/src/state_machine/mod.rs` |
| `RT_UNDO_NO_ACTIVE_SKILL` | `active_streaming_skill is None, defaulting to HubRis` | `scepter/src/state_machine/mod.rs` |

---

> **Nota**: Este es un catálogo de mejor esfuerzo. Entelecheia está migrando hacia
> tipos de error estructurados con códigos únicos. Las contribuciones para expandir esta
> referencia son bienvenidas.
