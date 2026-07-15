+++
name = "static_analyze"
agent = "classic-software-engineering"
execution_mode = "read"
location = "cosmos"

[description]
en = "Run language-specific static analysis (cargo clippy, eslint, pylint, go vet) via host_command_exec. Parses structured output and returns categorized findings with file/line/severity."
zhs = "通过 host_command_exec 运行语言特定的静态分析（cargo clippy、eslint、pylint、go vet）。解析结构化输出并返回按文件/行号/严重性分类的问题。"
zht = "通過 host_command_exec 運行語言特定的靜態分析（cargo clippy、eslint、pylint、go vet）。解析結構化輸出並返回按檔案/行號/嚴重性分類的問題。"
ja = "host_command_exec 経由で言語固有の静的解析（cargo clippy、eslint、pylint、go vet）を実行。構造化出力を解析し、ファイル/行/重大度で分類した結果を返します。"
ko = "host_command_exec를 통해 언어별 정적 분석(cargo clippy, eslint, pylint, go vet) 실행. 구조화된 출력을 파싱하고 파일/줄/심각도별로 분류된 결과를 반환합니다."
fr = "Exécuter l'analyse statique spécifique au langage (cargo clippy, eslint, pylint, go vet) via host_command_exec. Analyse la sortie structurée et retourne les résultats catégorisés par fichier/ligne/sévérité."
es = "Ejecutar análisis estático específico del lenguaje (cargo clippy, eslint, pylint, go vet) a través de host_command_exec. Analiza la salida estructurada y devuelve hallazgos categorizados por archivo/línea/severidad."
ru = "Запустить статический анализ для конкретного языка (cargo clippy, eslint, pylint, go vet) через host_command_exec. Разбирает структурированный вывод и возвращает классифицированные результаты по файлу/строке/серьёзности."

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

# static_analyze

Run language-specific static analysis tools on the workspace and return categorized, actionable findings.

## IMPORTANT: Host Path Convention

This skill runs on the **host** via `host_command_exec`. Use **host paths** (e.g. `/mnt/sdb1/entelecheia`), NOT container paths (`/workspace`).

## Analysis Commands by Language

| Language | Detection File | Scan Command | Output Format |
| --- | --- | --- | --- |
| Rust | `Cargo.toml` | `cargo clippy --message-format=json -- -W clippy::all 2>&1` | JSON lines |
| TypeScript/JS | `package.json` | `npx eslint --format json --no-color . 2>&1` | JSON array |
| Python | `pyproject.toml` / `setup.py` | `python -m pylint --output-format=json . 2>&1` | JSON array |
| Go | `go.mod` | `go vet ./... 2>&1` | text |
| Java | `pom.xml` | `mvn compile 2>&1` | text |

## CRITICAL: Single Exec Rule

You MUST run the scan command in a **single** `host_command_exec` call. Do NOT split detection and scanning into separate calls.

## SoP

### Step 1: DETECT & SCAN (mandatory)

Run the appropriate scan command. Use the workspace path from the environment:

```typescript
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /mnt/sdb1/entelecheia && cargo clippy --message-format=json -- -W clippy::all 2>&1 | head -200', timeout: 180 }); const out = r.data.stdout || r.data.stderr || JSON.stringify(r.data); const lines = out.split('\\n'); const findings = []; for (const line of lines) { try { const msg = JSON.parse(line); if (msg.reason === 'compiler-message' && msg.message) { findings.push({ level: msg.message.level, code: msg.message.code?.code || 'unknown', message: msg.message.rendered?.trim(), spans: (msg.message.spans || []).map(s => ({ file: s.file_name, line_start: s.line_start, line_end: s.line_end })) }); } } catch {} } write_to_var({ var_name: 'dag', content: JSON.stringify(findings) }); console.log('Findings:', findings.length, 'Errors:', findings.filter(f => f.level === 'error').length, 'Warnings:', findings.filter(f => f.level === 'warning').length);" })
```

**Language-specific templates (copy-paste the one you need):**

#### Rust — cargo clippy (structured JSON)

```typescript
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /mnt/sdb1/entelecheia && cargo clippy --message-format=json -- -W clippy::all 2>&1 | head -500', timeout: 180 }); const out = r.data.stdout || r.data.stderr || ''; const findings = []; for (const line of out.split('\\n')) { try { const m = JSON.parse(line); if (m.reason === 'compiler-message' && m.message && m.message.level !== 'note') { findings.push({ level: m.message.level, code: m.message.code?.code || '', msg: m.message.rendered?.split('\\n')[0], file: m.message.spans?.[0]?.file_name, line: m.message.spans?.[0]?.line_start }); } } catch {} } write_to_var({ var_name: 'dag', content: JSON.stringify(findings) }); console.log(JSON.stringify({ total: findings.length, errors: findings.filter(f=>f.level==='error').length, warnings: findings.filter(f=>f.level==='warning').length }, null, 1));" })
```

#### Rust — cargo check (compilation errors only)

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /mnt/sdb1/entelecheia && cargo check 2>&1 | tail -50', timeout: 180 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

#### TypeScript/JavaScript — eslint

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /mnt/sdb1/entelecheia && npx eslint --format json --no-color . 2>&1 | head -200', timeout: 120 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

#### Python — pylint

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /mnt/sdb1/entelecheia && python -m pylint --output-format=json . 2>&1 | head -200', timeout: 120 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

#### Go — go vet

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /mnt/sdb1/entelecheia && go vet ./... 2>&1', timeout: 120 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

### Step 2: FILTER (mandatory)

Filter out noise. Exclude:

- `missing documentation` warnings (not actionable)
- `generated N warnings` summary lines
- Warnings from dependency crates (paths containing `.cargo/`, `node_modules/`)
- Patch/version warnings

Only keep actionable findings: unused imports, unused variables, dead code, type errors, clippy style issues, real compilation errors.

### Step 3: REPORT

```json
write_to_var({ var_name: "rep", content: "FINDINGS_SUMMARY" })
exec({ code: "import { report } from 'hubris'; report({ text: __vars['rep'] });" })
```

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Signal over noise**: Filter aggressively. Only report findings the developer can act on.
- **One scan per call**: Never run `cargo clippy` then `cargo check` separately — they overlap.
- **JSON parsing first**: Always try structured parsing. Fall back to text parsing only if JSON fails.
- **Respect timeout**: Set `timeout: 180` for large workspaces. If it times out, narrow scope to specific packages.

> Return type: @system/return-type-convention
> IEPL-first: @system/iepl-first
