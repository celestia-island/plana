+++
name = "i18n_document_sync"
agent = "classic-software-engineering"
execution_mode = "read"
status = "archived"

[description]
en = "Discover internationalized content, choose a base language, and orchestrate translation sync."

[[related_tools]]
name = "script_exec"
agent = "skemma"
description = "Scan documentation trees and i18n resource files"

[[related_tools]]
name = "quality_check"
agent = "classic_software_engineering"
description = "Validate translated structure and completeness"

[[related_tools]]
name = "file_list"
agent = "kalos"
description = "Enumerate i18n files across directory trees"

[[related_tools]]
name = "file_read"
agent = "kalos"
description = "Read base language and target language content for diff analysis"

[[related_tools]]
name = "report_human"
agent = "hubris"
description = "Escalate unresolved terminology or policy conflicts"
+++

# i18n_document_sync

> **ARCHIVED — NOT IN ACTIVE DEVELOPMENT**
> This skill references the `literary-creation` Layer2 agent which has been archived. The `translation_workflow` delegation requires that agent to be restored first. Do not implement or schedule unless explicitly requested.

## Description

Scans a repository for internationalized assets, determines the base language, and orchestrates translation synchronization across all supported locales. Discovers Markdown docs, prompts, JSON/TOML/YAML resources, and language-scoped guides, then validates structural consistency across languages.

## Preconditions

- Repository has internationalized content (multiple language files or directories)
- Base language is determined or can be inferred
- Container with file access is available

## SOP

### Step 1: Asset Discovery

```bash
$ file_list(path=<scope>, recursive=true, patterns=["**/*.md", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml"])
```

- Scan docs, packages, config trees for internationalized content
- Detect layout pattern:
  - **Directory-per-language**: `docs/en/`, `docs/ja/`, `docs/ko/`
  - **Suffix-per-language**: `soul/{agent}.md (flat, one per agent)`, `soul/{agent}.md`, `res/prompts/soul/{agent}.md`
  - **Key-value resource**: `locales/en.json`, `locales/ja.json`
- Catalog all discovered assets with language tag and path
- **Gate**: If no multilingual content found → return `"No internationalized content detected"`

### Step 2: Base Language Confirmation

- Check explicit `base_language` parameter from caller
- If not provided, check agent preference or repository convention
- Fall back to: language with most files, or `en` by convention
- **Gate**: If base language cannot be determined → error `"Cannot determine base language. Specify explicitly."`

### Step 3: Diff Analysis

For each base-language file:

```bash
$ file_read(path=<base_file>)
```

- Match to target-language counterparts using detected naming pattern
- For each target language:
  - **Missing**: target file does not exist → flag as "missing translation"
  - **Stale**: target file exists but older than base → flag as "needs update"
  - **Structural drift**: headings, frontmatter keys, links differ → flag as "structural mismatch"
- **Gate**: If > 50% of files are missing for a language → flag as "language incomplete"

### Step 4: Structural Validation

```bash
$ quality_check(scope=<all_language_files>, metrics=["structural_consistency"])
```

- Verify frontmatter keys match across languages
- Check heading structure (same hierarchy, same count)
- Validate links, code fences, table shapes remain aligned
- Protect non-translatable tokens: code blocks, commands, paths, config keys, placeholders
- **Gate**: If structural drift detected → flag specific files for manual review

### Step 5: Coverage Report

```bash
$ report(
  summary="i18n sync: <L> languages, <T> total items, <M> missing, <S> stale, <D> drifted",
  body=<coverage_matrix_json>,
  severity=<highest_severity>,
  base_language=<lang>,
  coverage_matrix={<lang>: {total, translated, stale, drifted, pct}>,
  items_requiring_human=<terminology_conflicts>
)
```

## Postconditions

- Coverage matrix with per-language completeness
- Missing and stale translations identified
- Structural mismatches flagged for human review
- No translations applied (read-only analysis; translation execution requires literary-creation agent)

@system/return-type-convention
