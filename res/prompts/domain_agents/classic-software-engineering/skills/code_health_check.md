+++
name = "code_health_check"
agent = "classic-software-engineering"
execution_mode = "read"
location = "cosmos"

[description]
en = "Evaluate codebase health: oversized files, unwrap/expect usage, dead code density, and dependency freshness via host_command_exec."
zh-Hans = "代码健康检查：超大文件、unwrap/expect 使用、死代码密度、依赖新鲜度（通过 host_command_exec）。"
zh-Hant = "程式碼健康檢查：超大檔案、unwrap/expect 使用、死程式碼密度、依賴新鮮度（通過 host_command_exec）。"
ja = "コードヘルスチェック：巨大ファイル、unwrap/expect使用、デッドコード密度、依存関係の鮮度（host_command_exec経由）。"
ko = "코드 건강 검사: 과대 파일, unwrap/expect 사용, 데드 코드 밀도, 종속성 신선도 (host_command_exec 경유)."
fr = "Vérification de santé du code : fichiers surdimensionnés, usage de unwrap/expect, densité de code mort, fraîcheur des dépendances via host_command_exec."
es = "Verificación de salud del código : archivos sobredimensionados, uso de unwrap/expect, densidad de código muerto, frescura de dependencias vía host_command_exec."
ru = "Проверка здоровья кода : раздутые файлы, использование unwrap/expect, плотность мёртвого кода, свежесть зависимостей через host_command_exec."

[[related_tools]]
agent_name = "polemos"
tool_name = "host_command_exec"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_skills]]
agent_name = "skopeo"
tool_name = "smart_command_execute"
+++

# code_health_check

Assess codebase health across four dimensions: file sizes, error-handling shortcuts, dead code density, and dependency freshness. All checks run via `host_command_exec` on the host.

## IMPORTANT: Host Path Convention

Use **host paths** (take the path from the environment section's `Workspace:` line, strip `local://`, and pass it as the `cwd` parameter of `host_command_exec`), NOT container paths (`/workspace`).

## SoP

### Step 1: FILE SIZE AUDIT

Find oversized source files (>500 lines for Rust, >400 for TS/JS):

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'find packages -name \"*.rs\" -exec wc -l {} + 2>/dev/null | sort -rn | head -20', cwd: '<host-workspace>', timeout: 30 }); console.log(r.data.stdout || r.data.stderr);" })
```

**Gate**: Flag files >500 lines. If none, report "No oversized files."

### Step 2: ERROR-HANDLING AUDIT

Count `.unwrap()`, `.expect()`, `panic!()`, `todo!()` in non-test code:

```text
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'rg \"\\\\.unwrap\\\\(\\\\)|\\\\.expect\\\\(|panic!|todo!\" --type rust -c 2>/dev/null | sort -t: -k2 -rn | head -20', cwd: '<host-workspace>', timeout: 30 }); console.log(r.data.stdout || r.data.stderr);" })
```

**Classification**:

- Files in `tests/` → acceptable (test code)
- Files in `build.rs` or `main.rs` startup → acceptable (fail-fast)
- Files in business logic → report as finding
- **Gate**: If unwrap count > 20 in non-test code → severity = high

### Step 3: DEAD CODE & CLIPPY AUDIT

Run clippy to find dead code and unused imports:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo clippy -- -W clippy::all 2>&1 | grep -E \"dead_code|unused_imports|unused_variables\" | sort -u | head -30', cwd: '<host-workspace>', timeout: 180 }); console.log(r.data.stdout || r.data.stderr);" })
```

**Gate**: Count unique files with dead code. If > 10 files → severity = medium.

### Step 4: DEPENDENCY AUDIT (Rust only)

Check for outdated or unused dependencies:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo clippy -- -W clippy::all 2>&1 | grep \"patch.*was not used\" | head -10', cwd: '<host-workspace>', timeout: 180 }); console.log(r.data.stdout || r.data.stderr);" })
```

### Step 5: RANK & REPORT

Classify each finding:

- **Mechanical fix** (safe to auto-apply): unused imports, redundant clones
- **Local design** (needs review): unwrap → proper error handling
- **Architecture decision** (needs discussion): oversized files needing splits

```json
write_to_var({ var_name: "rep", content: "FINDINGS_SUMMARY" })
exec({ code: "import { report } from 'hubris'; report({ text: __vars['rep'] });" })
```

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Split by responsibility, not line count**: File splits must follow module boundaries
- **Exempt generated code, migrations, build scripts**: Do not flag auto-generated patterns
- **Keep dynamic JSON only where genuinely needed**: Extensible/plugin contexts are valid
- **Signal over noise**: Only report findings that improve code quality

> Return type: @system/return-type-convention
> IEPL-first: @system/iepl-first
