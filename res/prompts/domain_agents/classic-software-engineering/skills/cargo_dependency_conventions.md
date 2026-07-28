+++
name = "cargo_dependency_conventions"
agent = "classic-software-engineering"
status = "ruleset"

[description]
en = "Configurable ruleset for Cargo.toml dependency grouping, ordering, and versioning. Loaded by code_standards during Standards Discovery."
zh-Hans = "Cargo.toml 依赖分组、排序与版本规范的可配置规则集。由 code_standards 在标准发现阶段加载。"
zh-Hant = "Cargo.toml 依賴分組、排序與版本規範的可配置規則集。由 code_standards 在標準發現階段載入。"
ja = "Cargo.tomlの依存関係グループ化、順序付け、バージョン管理のための設定可能なルールセット。code_standardsが標準検出フェーズで読み込む。"
ko = "Cargo.toml 종속성 그룹화, 순서 지정 및 버전 관리를 위한 구성 가능한 규칙 세트. code_standards가 표준 검색 단계에서 로드합니다."
fr = "Ensemble de règles configurables pour le regroupement, l'ordonnancement et le versionnement des dépendances Cargo.toml. Chargé par code_standards lors de la découverte des normes."
es = "Conjunto de reglas configurables para agrupación, ordenamiento y versionado de dependencias Cargo.toml. Cargado por code_standards durante el Descubrimiento de Estándares."
ru = "Настраиваемый набор правил для группировки, упорядочивания и версионирования зависимостей Cargo.toml. Загружается code_standards на этапе обнаружения стандартов."

[[loaded_by]]
skill = "code_standards"
phase = "Standards Discovery"
description = "Validates Cargo.toml dependency declarations against grouping, ordering, and versioning conventions"
+++

# cargo_dependency_conventions (RULESET)

> **Status**: This file is a **configuration ruleset**, not an executable skill. It is loaded by [`code_standards`](./code_standards.md) during the **Standards Discovery** phase (SOP Step 1) and validated during the **Dependency Convention Verification** phase (Step 4). Do not invoke this file directly.

## Description

Enforces unified Cargo.toml dependency writing, grouping, ordering, and versioning conventions across the Entelecheia workspace. When adding, upgrading, or reordering any dependency, this ruleset defines the expected structure.

## Dependency Grouping and Ordering

Within `[dependencies]`, group dependencies by functional/semantic category, separated by blank lines. Within each group, sort alphabetically by crate name:

1. **Workspace dependencies** — Internal workspace packages with `_` prefix aliases, e.g. `_shared`, `_res`, `_tui`.
1. **Workspace agent dependencies** — Other agent crates within the workspace, e.g. `haplotes`, `skopeo`, `hubris`.
1. **Language fundamentals & tooling** — Error handling, traits, CLI, e.g. `anyhow`, `thiserror`, `async-trait`, `clap`.
1. **Data & serialization** — Data structures, serialization, IDs, time, e.g. `serde`, `serde_json`, `uuid`, `chrono`, `regex`, `toml`, `bincode`.
1. **Logging & tracing** — e.g. `tracing`, `tracing-subscriber`, `tracing-appender`.
1. **Async / concurrency runtime** — e.g. `tokio` and related crates (`tokio-stream`, `tokio-tungstenite`, `tokio-util`), `futures`, `async-stream`.
1. **Filesystem & paths** — e.g. `dirs`, `notify`, `walkdir`, `tempfile`.
1. **Network / protocol** — HTTP, WebSocket, MCP protocol stack, e.g. `reqwest`, `axum`, `tower`, `tower-http`, `bollard`, `interprocess`, `urlencoding`.

## Version Number Rules

All dependency versions use caret (`^`) semantic versioning:

1. **Major version >= 1**: Keep only major version, write as `^<major>`.

   - `4.1.3` → `^4`
   - `2.0.0` → `^2`

1. **Major version 0, minor >= 1**: Keep `0.<minor>`, write as `^0.<minor>`.

   - `0.12.4` → `^0.12`
   - `0.3.0` → `^0.3`

1. **Version at 0.0.x**: Pin exact version.

   - `0.0.7` → `0.0.7`

1. **Dependencies with features**: Follow the above rules and add `features` field.

   ```toml
   anyhow = { version = "^1", features = ["backtrace"] }
   serde = { version = "^1", features = ["derive"] }
   uuid = { version = "^1", features = [
     "v4",
     "serde",
   ] }
   ```

## Workspace `[workspace.dependencies]` Grouping

The workspace root `Cargo.toml` uses `[workspace.dependencies]` with these semantic groups (ordered):

1. **Workspace internal aliases** — `_shared`, `_res`, `_tui`
1. **Language fundamentals** — `anyhow`, `async-trait`, `thiserror`, `clap`
1. **Data & serialization** — `serde`, `serde_json`, `toml`, `bincode`, `uuid`, `chrono`, `regex`
1. **Logging & tracing** — `tracing`, `tracing-subscriber`, `tracing-appender`
1. **Async / concurrency** — `tokio`, `tokio-util`, `tokio-tungstenite`, `tokio-stream`, `futures`, `async-stream`
1. **Database** — `sea-orm`, `sea-orm-migration`, `deadpool-postgres`, `r2d2`, `pgvector`
1. **Network / protocol** — `reqwest`, `axum`, `tower`, `tower-http`, `bollard`, `interprocess`
1. **Filesystem & paths** — `dirs`, `notify`, `tempfile`
1. **Crypto & encoding** — `aes-gcm`, `base64`, `rand`, `sha2`, `hkdf`, `secrecy`
1. **TUI** — `ratatui`, `crossterm`, `unicode-width`
1. **Caching & concurrency** — `lru`, `dashmap`, `parking_lot`, `once_cell`, `lazy_static`, `flume`
1. **System & tools** — `sysinfo`, `pci-ids`, `sys-info`, `atty`, `arboard`, `yuuka`, `include_dir`, `libc`, `git2`, `strum`, `jsonwebtoken`, `dotenv`, `dotenvy`, `metrics`, `bytes`, `pin-project`, `url`, `urlencoding`, `boa_engine`, `boa_runtime`, `boa_gc`, `half`
1. **Test dependencies** — `tokio-test`, `tempfile`, `mockall`, `wiremock`

## Sub-crate `[dependencies]` Conventions

- Use `crate-name.workspace = true` for dependencies already in `[workspace.dependencies]`.
- For non-workspace dependencies, follow the same version and feature conventions.
- Remove stale comments like `# 核心库包（带下划线别名）` and `# 核心包`.

## Validation Rules (for code_standards)

When `code_standards` loads this ruleset, it validates:

1. **Group ordering**: Each group must appear in the order listed above
1. **Alphabetical within group**: Dependencies within each group sorted alphabetically
1. **Version format**: All versions follow the caret convention rules
1. **Workspace references**: Sub-crates use `workspace = true` for available dependencies
1. **No stale comments**: Dependency blocks should not contain descriptive comments

## Important Notes

- When adding a new dependency, place it in the semantically closest group.
- Always use caret (`^`) versions except for `0.0.x` crates.
- Dev-dependencies follow the same grouping philosophy.
- Full reference: `docs/design/en/18-cargo-dependency-conventions.md`
