+++
name = "auto_fix"
agent = "classic-software-engineering"
execution_mode = "write"
location = "cosmos"
must_touch_next_action = false

[description]
en = "Self-iteration loop: scan for warnings/errors, auto-fix what is safe (unused imports, clippy suggestions, i18n gaps), verify with cargo check, and commit. Runs a bounded number of fix cycles."
zh-Hans = "自迭代循环：扫描警告/错误，自动修复安全项（未使用的导入、clippy 建议、i18n 缺失），用 cargo check 验证并提交。运行有限次修复循环。"
zh-Hant = "自迭代循環：掃描警告/錯誤，自動修復安全項（未使用的導入、clippy 建議、i18n 缺失），用 cargo check 驗證並提交。運行有限次修復循環。"
ja = "自己反復ループ：警告/エラーをスキャンし、安全な項目（未使用インポート、clippy提案、i18n欠落）を自動修正し、cargo checkで検証してコミット。制限付き修正サイクルを実行。"
ko = "자체 반복 루프: 경고/오류를 스캔하고, 안전한 항목(미사용 임포트, clippy 제안, i18n 누락)을 자동 수정하고, cargo check로 검증 후 커밋. 제한된 수정 사이클 실행."
fr = "Boucle d'auto-itération : scanner les avertissements/erreurs, corriger automatiquement ce qui est sûr (imports inutilisés, suggestions clippy, lacunes i18n), vérifier avec cargo check et committer. Exécute un nombre limité de cycles de correction."
es = "Bucle de auto-iteración: escanear advertencias/errores, corregir automáticamente lo seguro (imports no usados, sugerencias clippy, brechas i18n), verificar con cargo check y hacer commit. Ejecuta un número limitado de ciclos de corrección."
ru = "Цикл самоитерации: сканировать предупреждения/ошибки, автоматически исправлять безопасное (неиспользуемые импорты, предложения clippy, пробелы i18n), проверить через cargo check и закоммитить. Выполняет ограниченное число циклов исправлений."

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

# auto_fix

Self-iteration loop: scan → filter → fix → verify → commit. Runs up to 3 fix cycles per invocation.

## IMPORTANT: Workspace Path

This skill runs in a cosmos container with the workspace mounted at `/workspace`.
When calling `host_command_exec`, use `cd /workspace` as the working directory.
Do NOT use `/workspace` — that variable does not exist. Always use `/workspace`.

## CRITICAL: Safety Boundaries

You MUST follow these rules:

1. **NEVER add new features, new files, or new modules.** Only remove or modify existing code.
1. **NEVER change function signatures, public APIs, or trait implementations.** Only fix internal code quality issues.
1. **Maximum 3 fix cycles.** After 3 cycles, report what remains and stop.
1. **Always verify with `cargo check` after each fix.** If `cargo check` fails, revert the change.
1. **Commit after each successful fix cycle.** Use descriptive commit messages.

## CRITICAL: Fix Priority

Fix in this order (highest impact first):

1. **Compilation errors** — code that doesn't compile
1. **`cargo fmt --all`** — formatting issues (run first, always safe)
1. **`cargo clippy --fix`** — auto-fixable clippy warnings (redundant clones, unnecessary `to_string`, etc.)
1. **Unused imports** — `use` statements for symbols not referenced
1. **Unused variables** — prefix with `_` or remove
1. **Dead code** — `#[allow(dead_code)]` or remove if clearly unused

**Do NOT fix:**

- `missing documentation` warnings (too noisy, not a code quality issue)
- `generated N warnings` summary lines
- Warnings from third-party dependency code

## SoP

### Step 1: SCAN (mandatory — first cycle only)

Run `cargo clippy` to identify all actionable warnings:

```text
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /workspace && cargo clippy --message-format=json -- -W clippy::all 2>&1 | head -500', timeout: 300 }); const out = r.data.stdout || r.data.stderr || ''; const findings = []; for (const line of out.split('\\n')) { try { const m = JSON.parse(line); if (m.reason === 'compiler-message' && m.message && m.message.level !== 'note') { const rendered = m.message.rendered || ''; if (rendered.includes('missing documentation')) continue; const code = m.message.code?.code || ''; findings.push({ level: m.message.level, code, msg: rendered.split('\\n')[0], file: m.message.spans?.[0]?.file_name, line: m.message.spans?.[0]?.line_start }); } } catch {} } write_to_var({ var_name: 'scan', content: JSON.stringify(findings) }); console.log('Scan found', findings.length, 'actionable findings:', JSON.stringify(findings.slice(0, 10), null, 1));" })
```

**Gate**: If 0 actionable findings → skip to Step 5, report "No issues found."

### Step 2: FORMAT (mandatory, always safe)

Run `cargo fmt --all` to fix all formatting issues:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /workspace && cargo fmt --all 2>&1', timeout: 60 }); console.log(r.data.stdout || r.data.stderr || 'Formatted');" })
```

### Step 3: AUTO-FIX (mandatory)

Try `cargo clippy --fix --allow-dirty` first. This handles most mechanical fixes:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /workspace && cargo clippy --fix --allow-dirty --allow-staged -- -W clippy::all 2>&1 | tail -20', timeout: 300 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

### Step 4: VERIFY (mandatory after each fix)

Run `cargo check` to ensure nothing broke:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /workspace && cargo check 2>&1 | tail -20', timeout: 300 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

**Gate**: If `cargo check` fails with errors → the auto-fix broke something. Run:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /workspace && git checkout -- . 2>&1', timeout: 30 }); console.log(r.data.stdout || r.data.stderr || 'Reverted');" })
```

Then stop and report "Auto-fix caused compilation errors. Reverted."

### Step 5: COMMIT (mandatory after successful verify)

If `cargo check` passes, commit the changes:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /workspace && git add -A && git diff --cached --stat', timeout: 30 }); console.log(r.data.stdout || r.data.stderr || 'No changes');" })
```

If there are staged changes:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: \"cd /workspace && git commit -m '🐛 Fix clippy warnings from the auto-fix cycle.'\", timeout: 30 }); console.log(r.data.stdout || r.data.stderr || 'Committed');" })
```

### Step 6: RE-SCAN (optional — for remaining issues)

Re-run Step 1 to check if all issues are fixed. If new issues remain AND cycle count < 3, go back to Step 2.

### Step 7: REPORT

```json
write_to_var({ var_name: "rep", content: "EXECUTION_SUMMARY" })
exec({ code: "import { report } from 'hubris'; report({ text: __vars['rep'] });" })
```

Report must include:

- Number of fix cycles executed
- Files modified per cycle
- Warnings/errors before and after
- Whether `cargo check` passes

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Mechanical fixes only**: Never make design decisions. If a fix requires judgment, skip it and report it.
- **Verify or revert**: Every fix must pass `cargo check`. If it doesn't, revert immediately.
- **Bounded iteration**: Maximum 3 cycles. Don't get stuck in infinite loops.
- **Commit early and often**: Each successful fix cycle gets its own commit. Easy to revert if needed.

> Return type: @system/return-type-convention
> IEPL-first: @system/iepl-first
