# i18n & Format Alignment Plan

Target: restructure `docs/` → `docs/{lang}/...`, add TOML frontmatter to every document, batch-translate to 8 languages.

Target languages: `en`, `zh`, `zht`, `ja`, `ko`, `fr`, `es`, `ru`

---

## Phase 1 — Restructure into docs/en/

| Step | Status |
|------|--------|
| Move all current md files → `docs/en/` (except README.md, logo.webp, licenses/) | [ ] |
| Create empty dirs for other languages | [ ] |
| Update docs/README.md index | [ ] |
| Commit | [ ] |

---

## Phase 2 — TOML Frontmatter

Add `+++ ... +++` header to every doc:

```toml
+++
title = "Document Title"
description = "Brief description"
lang = "en"
category = "meta|architecture|design|guides"
subcategory = "core|webui"
+++
```

| Batch | Files | Status |
|-------|-------|--------|
| meta/ (4 files) | CLA, CODE_OF_CONDUCT, CONTRIBUTING, SECURITY | [ ] |
| architecture/core/ (2 files) | architecture, plan | [ ] |
| architecture/webui/ (2 files) | about, architecture | [ ] |
| design/core/ (29 files) | agent-config-system ... wasi-plugin-system | [ ] |
| design/webui/ (13 files) | 01-..10, plant-project-format, rbac-design, README | [ ] |
| guides/core/ (11 files) | agent-development ... webhook-setup | [ ] |
| guides/webui/ (6 files) | architecture ... webhook-setup | [ ] |

---

## Phase 3a — Translate to zh (Simplified Chinese)

| Batch | Files | Status |
|-------|-------|--------|
| meta (4) | CLA, CODE_OF_CONDUCT, CONTRIBUTING, SECURITY | [ ] |
| architecture/core (2) | architecture, plan | [ ] |
| architecture/webui (2) | about, architecture | [ ] |
| design/core (29) | all 29 design docs | [ ] |
| design/webui (13) | all 13 design docs | [ ] |
| guides/core (11) | all 11 guides | [ ] |
| guides/webui (6) | all 6 guides | [ ] |

---

## Phase 3b — Translate to zht (Traditional Chinese)

Same 7 batches as Phase 3a. Status: [ ] all

---

## Phase 3c — Translate to ja (Japanese)

Same 7 batches. Status: [ ] all

---

## Phase 3d — Translate to ko (Korean)

Same 7 batches. Status: [ ] all

---

## Phase 3e — Translate to fr (French)

Same 7 batches. Status: [ ] all

---

## Phase 3f — Translate to es (Spanish)

Same 7 batches. Status: [ ] all

---

## Phase 3g — Translate to ru (Russian)

Same 7 batches. Status: [ ] all

---

## Phase 4 — Format Alignment

| Step | Status |
|------|--------|
| Verify all files have TOML frontmatter | [ ] |
| Consistent heading hierarchy (H2 after frontmatter) | [ ] |
| License block at end of each doc | [ ] |
| Commit | [ ] |
