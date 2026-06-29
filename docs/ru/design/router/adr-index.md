+++
title = "Architecture Decision Records (ADR)"
description = """Указатель записей архитектурных решений для evernight."""
lang = "ru"
category = "design"
subcategory = "router"
+++

# Architecture Decision Records (ADR)

This directory records the key architectural decisions made during the development of Evernight. Each ADR explains **what** was decided, **why** it was decided, and what **trade-offs** were considered.

ADRs follow the [Michael Nygard ADR template](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions). They are immutable once published — superseded decisions are marked accordingly.

## Index

| # | Title | Status |
|---|-------|--------|
| screen-capture | [Screen Capture Architecture](../en/adr/screen-capture.md) | Accepted |
| feature-flags | [Feature Flag Architecture](../en/adr/feature-flags.md) | Accepted |
| ssh-backend | [SSH Backend — russh with Shared Connection](../en/adr/ssh-backend.md) | Accepted |
| async-runtime | [Async Runtime — tokio](../en/adr/async-runtime.md) | Accepted |
| error-handling | [Error Handling — thiserror with crate Result](../en/adr/error-handling.md) | Accepted |
| module-decoupling | [Module Decoupling and Type Ownership](../en/adr/module-decoupling.md) | Accepted |
| signaling-transport | [Signaling Transport — Dual Unix Socket / TCP](../en/adr/signaling-transport.md) | Accepted |
| backend-traits | [TerminalBackend / ViewportBackend / FileBackend Trait Abstractions](../en/adr/backend-traits.md) | Accepted |
| ssh-connection-pool | [SSH Connection Pool](../en/adr/ssh-connection-pool.md) | Accepted |
| vnc-rfb-client | [VNC (RFB) Protocol Client](../en/adr/vnc-rfb-client.md) | Accepted |
| ssh-config-parser | [SSH Config Parser](../en/adr/ssh-config-parser.md) | Accepted |
| connection-entry-uri | [Connection Entry URI Scheme](../en/adr/connection-entry-uri.md) | Accepted |
| serial-port-aoba | [Serial Port Communication via aoba](../en/adr/serial-port-aoba.md) | Accepted |
| container-runtime | [Container Runtime Client (Docker/Podman)](../en/adr/container-runtime.md) | Accepted |

## Language Directories

| Code | Language |
|------|----------|
| `en/` | English (authoritative) |
| `zhs/` | Simplified Chinese |
| `zht/` | Traditional Chinese |
| `ja/` | Japanese |
| `ko/` | Korean |
| `fr/` | French |
| `es/` | Spanish |
| `ru/` | Russian |
