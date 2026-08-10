+++
name = "code_standards"
agent = "classic-software-engineering"
execution_mode = "read"
location = "cosmos"

[description]
en = "Unified code quality enforcement: scan for naming violations, import ordering, dependency conventions, and structural consistency via host_command_exec."
zh-Hans = "统一代码质量管控：通过 host_command_exec 扫描命名违规、导入排序、依赖规范和结构一致性。"
zh-Hant = "統一程式碼品質管控：通過 host_command_exec 掃描命名違規、導入排序、依賴規範和結構一致性。"
ja = "統合コード品質管理：host_command_exec経由で命名違反、インポート順序、依存関係規約、構造的一貫性をスキャン。"
ko = "통합 코드 품질 관리: host_command_exec를 통해 명명 위반, 임포트 순서, 종속성 규칙, 구조적 일관성을 스캔."
fr = "Application unifiée de la qualité du code : scanner les violations de nommage, l'ordre des imports, les conventions de dépendances et la cohérence structurelle via host_command_exec."
es = "Aplicación unificada de calidad de código : escanear violaciones de nombres, orden de imports, convenciones de dependencias y consistencia estructural vía host_command_exec."
ru = "Единый контроль качества кода : сканировать нарушения именования, порядок импортов, соглашения о зависимостях и структурную согласованность через host_command_exec."

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

# code_standards

Scan the codebase for standards compliance: naming conventions, import ordering, dependency conventions, and structural consistency.

## IMPORTANT: Host Path Convention

Use **host paths** (take the path from the environment section's `Workspace:` line, strip `local://`, and pass it as the `cwd` parameter of `host_command_exec`), NOT container paths (`/workspace`).

## SoP

### Step 1: CONFIGURATION DISCOVERY

Read project configuration files:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cat .clippy.toml 2>/dev/null; echo \"---\"; cat rustfmt.toml 2>/dev/null; echo \"---\"; head -50 Cargo.toml 2>/dev/null', cwd: '<host-workspace>', timeout: 15 }); console.log(r.data.stdout || r.data.stderr);" })
```

**Gate**: If no config files found → use sensible defaults, note "Using default standards."

### Step 2: CLIPPY STANDARDS CHECK

Run clippy with strict standards:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo clippy -- -W clippy::all -D clippy::enum_glob_use -D clippy::unwrap_used 2>&1 | grep -v \"missing documentation\" | grep \"warning\\|error\" | sort -u | head -40', cwd: '<host-workspace>', timeout: 180 }); console.log(r.data.stdout || r.data.stderr);" })
```

### Step 3: IMPORT ORDERING CHECK (Rust)

Verify import grouping follows std → external → crate → super → self:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo clippy -- -W clippy::wildcard_imports -W clippy::useless_import 2>&1 | grep \"wildcard_import\\|useless_import\" | head -20', cwd: '<host-workspace>', timeout: 180 }); console.log(r.data.stdout || r.data.stderr);" })
```

### Step 4: DEPENDENCY CONVENTION CHECK

For Rust projects, check Cargo.toml for:

- Workspace inheritance usage
- Version format consistency
- Unused patches

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo clippy 2>&1 | grep \"patch.*was not used\" | head -10', cwd: '<host-workspace>', timeout: 180 }); console.log(r.data.stdout || r.data.stderr);" })
```

### Step 5: REPORT

```json
write_to_var({ var_name: "rep", content: "FINDINGS_SUMMARY" })
exec({ code: "import { report } from 'hubris'; report({ text: __vars['rep'] });" })
```

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Standards are a team contract**: Respect project-level config; never impose contradictory standards
- **Auto-fixable vs human judgment**: Never silently apply fixes that change semantics
- **Incremental enforcement**: Focus on new/changed code in CI, full scans for scheduled audits

> Return type: @system/return-type-convention
> IEPL-first: @system/iepl-first
