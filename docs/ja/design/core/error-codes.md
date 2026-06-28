+++
title = "エラーコードリファレンス"
description = """> 重要: 以下に文書化された「エラーコード」（例：DB_CONNECT_FAILED、LLM_CALL_FAILED）は、ソースコードから抽出された文字列ベースのメッ"""
lang = "ja"
category = "design"
subcategory = "core"
+++

# エラーコードリファレンス

> **重要**: 以下に文書化された「エラーコード」（例：`DB_CONNECT_FAILED`、
> `LLM_CALL_FAILED`）は、ソースコードから抽出された**文字列ベースのメッセージパターン**
> です — これらは非公式な便宜上のラベルであり、正式なエラー分類ではありません。
> **権威ある構造化エラータイプ**は
> [`packages/shared/core/src/errors.rs`](../packages/shared/core/src/errors.rs)で定義された列挙型です：
> `AgentErrorCode`（8行目）、`StructuredAgentError`、`CoreError`、`CredentialError`、
> `PromptLoadError`、`SoulLoadError`、およびそれらのバリアント。プログラムでエラーを統合または
> 報告する場合は、ここにリストされた文字列パターンではなく、それらの列挙型を使用してください。

このドキュメントはEntelecheiaのRustコードベース全体で使用されるエラーパターンをカタログ化します。
構造化エラーコードは作業中です；現在ほとんどのエラーは説明的なメッセージを持つ`anyhow`/`thiserror`を使用しています。

## エラーカテゴリ

### データベース (`DB_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `DB_CONNECT_FAILED` | `database connection failed: {}` | `scepter/src/app/setup.rs` |
| `DB_MIGRATE_FAILED` | `database migration failed: {}` | `scepter/src/app/setup.rs` |
| `DB_TABLE_CHECK_FAILED` | `failed to check table existence: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_INIT_SCRIPT_FAILED` | `failed to execute initialization script: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_SAVE_FAILED` | `failed to save agent info: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_UPDATE_FAILED` | `failed to update agent status: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_LOG_FAILED` | `failed to log entry: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_CLEANUP_FAILED` | `failed to clean up old logs: {}` | `packages/shared/infra_services/src/persistence.rs` |

### 設定 (`CFG_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `CFG_CREDENTIAL_INIT_FAILED` | `credential storage initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_PROVIDER_INIT_FAILED` | `provider config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_MODEL_INIT_FAILED` | `model config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_USER_INIT_FAILED` | `user config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `CFG_KEY_STORE_INIT_FAILED` | `key storage service initialization failed: {}` | `scepter/src/app/setup.rs` |

### 状態 (`ST_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `ST_SERIALIZE_FAILED` | `state serialization failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_WRITE_FAILED` | `temp file write failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_READ_FAILED` | `state file read failed: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_PARSE_FAILED` | `state file parse failed: {}` | `scepter/src/state/state_persistence.rs` |

### WebSocket (`WS_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `WS_SEND_FAILED` | `failed to send message: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_TIMEOUT` | `response wait timeout or channel closed` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_PARSE_FAILED` | `failed to parse agent list: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_NOT_CONNECTED` | `websocket connection not established` | `packages/shared/infra_services/src/ws_transport.rs` |

### エージェント (`AG_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `AG_CONNECT_FAILED` | `connection failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_SEND_FAILED` | `send failed: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_CHANNEL_NOT_INIT` | `send channel not initialized` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_REGISTRATION_FAILED` | `failed to send registration message: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_RE_REGISTER_FAILED` | `internal agent re-registration failed` | `scepter/src/state/state_restoration.rs` |

### LLM (`LLM_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `LLM_CALL_FAILED` | `LLM call failed: {}` | `scepter/src/state_machine/llm_chat/chat_loop.rs` |

### Layer2エージェント (`L2_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `L2_INIT_FAILED` | `layer2 agent config initialization failed: {}` | `scepter/src/app/setup.rs` |
| `L2_SKILLS_VALIDATE_FAILED` | `layer2 agent skills validation failed: {}` | `scepter/src/app/setup.rs` |

### スキル (`SK_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `SK_PROMPT_LOAD_FAILED` | `prompt loader error` | `packages/shared/prompt/src/prompt_loader.rs` |
| `SK_TOML_PARSE_FAILED` | `TOML parse failed: {}` | `packages/shared/prompt/src/prompt_loader.rs` |

### ランタイム (`RT_*`)

| コード | メッセージパターン | ソース |
| --- | --- | --- |
| `RT_ARC_UNWRAP_DOMAIN` | `Arc::try_unwrap failed for llm_domain/agent_domain` | `scepter/src/state_machine/mod.rs` |
| `RT_UNDO_NO_ACTIVE_SKILL` | `active_streaming_skill is None, defaulting to HubRis` | `scepter/src/state_machine/mod.rs` |

---

> **注意**: これはベストエフォートのカタログです。Entelecheiaは一意のコードを持つ
> 構造化エラータイプへの移行中です。このリファレンスの拡張への貢献を歓迎します。
