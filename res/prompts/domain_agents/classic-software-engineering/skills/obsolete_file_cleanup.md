+++
name = "obsolete_file_cleanup"
agent = "classic-software-engineering"
execution_mode = "write"

[description]
en = "Clean obsolete files safely in versioned repositories and escalate sensitive file exposure."
zhs = "安全清理版本库中的过时文件，升级处理敏感文件暴露"
zht = "安全清理版本庫中的過時檔案，升級處理敏感檔案暴露"
ja = "バージョン管理リポジトリ内の古いファイルを安全にクリーンアップし、機密ファイルの露出をエスカレーション"
ko = "버전 관리 저장소에서 오래된 파일을 안전하게 정리하고 민감 파일 노출을 에스컬레이션"
fr = "Nettoyer en toute sécurité les fichiers obsolètes dans les dépôts versionnés et signaler l'exposition de fichiers sensibles"
es = "Limpiar archivos obsoletos de forma segura en repositorios versionados y escalar la exposición de archivos sensibles"
ru = "Безопасная очистка устаревших файлов в версионируемых репозиториях и эскалация случаев утечки конфиденциальных файлов"

[[related_tools]]
name = "lsp_symbols"
agent = "classic_software_engineering"
description = "Check whether source files are referenced from active modules"

[[related_tools]]
name = "static_analyze"
agent = "classic_software_engineering"
description = "Identify dead code, unreachable modules, and unused artifacts"

[[related_tools]]
name = "security_audit"
agent = "orexis"
description = "Detect sensitive files and leakage risk"

[[related_tools]]
name = "script_exec"
agent = "skemma"
description = "Run git-aware scans and cleanup verification scripts"

[[related_tools]]
name = "file_list"
agent = "kalos"
description = "Enumerate candidate files for cleanup"

[[related_tools]]
name = "file_delete"
agent = "kalos"
description = "Remove confirmed obsolete files"

[[related_tools]]
name = "report_human"
agent = "hubris"
description = "Escalate secret exposure or high-risk cleanup decisions to humans"

[[related_tools]]
name = "report"
agent = "hubris"
description = "Emit the cleanup report"
+++

# obsolete_file_cleanup

## Description

Removes obsolete source files, low-value report artifacts, personal configuration residue, and sensitive files from versioned repositories with precise traceability and minimal risk. Confirms deletion safety through reference analysis and always escalates sensitive file exposure to human reviewers before taking destructive action.

## Preconditions

- Repository is under version control (Git)
- Current branch allows destructive cleanup (not a protected branch)
- NeiKos container is available in Write mode (for file deletion)
- Git working tree is clean or changes are committed

## SOP

### Step 1: Repository and Risk Check

```bash
$ script_exec(command="git status --porcelain && git branch --show-current")
```

- Verify Git state is healthy (no merge conflicts, detached HEAD, etc.)
- Confirm current branch is not `main`/`master` (unless explicitly requested)
- Define cleanup scope: `scope` parameter or full working tree
- **Gate**: If on protected branch and scope includes deletions → error `"Cannot perform destructive cleanup on <branch>"`

### Step 2: Candidate Discovery

```bash
$ file_list(path=<scope>, recursive=true, exclude=[".git", "target", "node_modules"])
$ static_analyze(scope=<scope>, checks=["dead_code"])
```

- Classify files into categories:
  - **Orphan source**: not imported/referenced by any active module
  - **Disposable reports**: generated summaries, audit artifacts, temporary logs
  - **Personal config**: editor settings, agent residue (`.DS_Store`, `*.swp`, `.env.local`)
  - **Sensitive**: `.env`, credentials, key material, tokens
- **Gate**: If no candidates found → return `"No obsolete files detected"`

### Step 3: Orphan Confirmation

For each orphan source file candidate:

```bash
$ lsp_symbols(file_path=<candidate>, depth="exports")
```

- Cross-reference exports with imports across the entire codebase
- Check build scripts, registries, entrypoints, and test files for references
- Mark as:
  - **High-confidence deletion**: zero references found across all sources
  - **Ambiguous**: dynamic imports, reflection, or config-driven loading possible
  - **In-use**: references found → remove from deletion candidates
- **Gate**: Ambiguous files → send to manual review, do not auto-delete

### Step 4: Report and Config Cleanup

For disposable reports and personal config:

```bash
$ file_delete(path=<disposable_file>)
```

- Remove: generated summaries, temporary logs, editor residue
- Preserve: shared team configuration that has become part of workflow
- **Gate**: If file is referenced in CI/CD config or Makefile → preserve, do not delete

### Step 5: Sensitive File Handling

For each sensitive file detected:

```bash
$ security_audit(scope=<sensitive_file>)
$ report_human(message="SENSITIVE FILE DETECTED: <file_path>. Tracked in version control. Recommend: 1) Add to .gitignore 2) Remove from tracking 3) Rotate exposed credentials")
```

- Add to `.gitignore` immediately
- Remove from Git tracking: `script_exec(command="git rm --cached <file>")`
- **NEVER auto-delete** sensitive files without human escalation
- **Gate**: If credentials/keys detected → CRITICAL severity, block further cleanup until human confirms

### Step 6: Deletion and Verification

```bash
$ file_delete(path=<confirmed_obsolete_file>)
```

- Execute grouped deletions in batches (max 10 files per batch)
- After each batch:

  ```bash
  $ script_exec(command="cargo check 2>&1 | head -5")
  ```

or equivalent build verification for the project language

- **Gate**: If build breaks → `git checkout -- <deleted_files>`, log failure, abort batch

### Step 7: Cleanup Report

```bash
$ report(
  summary="Cleanup: <D> deleted, <A> ambiguous (manual review), <S> sensitive (escalated)",
  body=<cleanup_details_json>,
  severity=<highest_severity>,
  deleted_files=<list>,
  ambiguous_files=<list>,
  sensitive_files=<list>
)
```

## Postconditions

- Traceable cleanup report with all actions taken
- Sensitive files removed from tracking but NOT deleted (human must confirm rotation)
- Build verified after all deletions
- No orphan sidecar processes

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Only delete with version control recovery**: Never delete files without Git or equivalent
- **Never silently delete sensitive files**: Always escalate to human first
- **Conservative with dynamic loading**: If evidence is incomplete, send to manual review

@system/return-type-convention
