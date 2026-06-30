+++
title = "Platform Design Documents"
description = """Cross-project (platform-level) design documents. Unlike the per-project core/webui/router subcategories, documents here cover concerns that span all three projects (entelecheia, shittim-chest, evernight) — for example the unified supervision, rolling-update and replication architecture they share."""
lang = "en"
category = "design"
subcategory = "platform"
+++

# Platform Design Documents

> **Scope.** These documents are *platform-level*: they cut across
> `core` (entelecheia), `webui` (shittim-chest) and `router`
> (evernight). Per-project designs live under their own subcategories.

## Index

| Document | Summary |
| --- | --- |
| [Supervision, Rolling Update & Replication](supervision-and-rolling-update.md) | A single supervision-tree backbone shared by all three projects: uniform signal/drain semantics, systemd socket activation for zero-downtime handoff, a pluggable coordination-lock trait, and two fault-tolerance strategies (Replica = load-balancing ⊃ rolling update; Leader/Follower = edge HA) built on the same Worker + Supervisor primitives. |

## Language Directories

| Code | Language |
| --- | --- |
| `en/` | English (canonical) |
| `zhs/` | 简体中文 (Simplified Chinese) |
| `zht/` | 繁體中文 (Traditional Chinese) |
| `ja/` | 日本語 (Japanese) |
| `ko/` | 한국어 (Korean) |
| `fr/` | Français (French) |
| `es/` | Español (Spanish) |
| `ru/` | Русский (Russian) |
