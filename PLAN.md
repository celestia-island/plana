# i18n & Format Alignment Plan

Status: **COMPLETED** — all 7 target languages + retructured docs with TOML frontmatter.

Target languages: `en`, `zhs`, `zht`, `ja`, `ko`, `fr`, `es`, `ru`

---

## Phase 1 — Restructure into docs/{lang}/ prefix

| Step | Status |
|------|--------|
| Move all current md files → `docs/en/` (except README.md, logo.webp, licenses/) | [x] |
| Create empty dirs for other languages | [x] |
| Update docs/README.md index | [x] |
| Commit | [x] |

---

## Phase 2 — TOML Frontmatter

| Batch | Status |
|-------|--------|
| meta/ (4 files) | [x] |
| architecture/core/ (2 files) | [x] |
| architecture/webui/ (2 files) | [x] |
| design/core/ (29 files) | [x] |
| design/webui/ (13 files) | [x] |
| guides/core/ (zhs source, 11 files) | [x] |
| guides/webui/ (6 files) | [x] |

---

## Phase 3a — Translate to zhs (Simplified Chinese)

| Batch | Files | Status |
|-------|-------|--------|
| meta (4) | CLA, CODE_OF_CONDUCT, CONTRIBUTING, SECURITY | [x] |
| architecture (4) | core/architecture, core/plan, webui/about, webui/architecture | [x] |
| guides/webui (6) | README, architecture, building, CONTRIBUTING, fundamentals, webhook-setup | [x] |
| design/core (29) | all 29 design docs | [x] |
| design/webui (13) | all 13 design docs | [x] |
| guides/core (11) | originally zhs, moved from en/ to zhs/ in Phase 2 | [x] |

---

## Phase 3b — zht (Traditional Chinese)

Same batches. Status: [x] complete (67 docs)

---

## Phase 3c — ja (Japanese)

Same batches. Status: [x] complete (67 docs)

---

## Phase 3d — ko (Korean)

Same batches. Status: [x] complete (67 docs)

---

## Phase 3e — fr (French)

Same batches. Status: [x] complete (67 docs)

---

## Phase 3f — es (Spanish)

Same batches. Status: [x] complete (67 docs)

---

## Phase 3g — ru (Russian)

Same batches. Status: [x] complete (67 docs)

---

## Final Counts

| Language | Files |
|----------|-------|
| en (source) | 56 |
| zhs | 67 |
| zht | 67 |
| ja | 67 |
| ko | 67 |
| fr | 67 |
| es | 67 |
| ru | 67 |
| **Total** | **525** |
