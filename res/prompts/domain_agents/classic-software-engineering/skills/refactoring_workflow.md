+++
name = "refactoring_workflow"
agent = "classic-software-engineering"
execution_mode = "write"

[description]
en = "Execute a refactoring plan: apply mechanical transforms, validate with LSP and tests, and commit in atomic steps."
zh-Hans = "重构工作流：应用机械转换 → LSP和测试验证 → 原子化提交"
zh-Hant = "重構工作流：應用機械轉換 → LSP和測試驗證 → 原子化提交"
ja = "リファクタリングワークフロー：機械的変換を適用 → LSPとテストで検証 → アトミックコミット"
ko = "리팩토링 워크플로우: 기계적 변환 적용 → LSP 및 테스트 검증 → 원자적 커밋"
fr = "Workflow de refactoring : appliquer les transformations mécaniques → valider avec LSP et tests → commits atomiques"
es = "Flujo de trabajo de refactoring : aplicar transformaciones mecánicas → validar con LSP y tests → commits atómicos"
ru = "Рабочий процесс рефакторинга : применить механические преобразования → проверить через LSP и тесты → атомарные коммиты"

[[related_tools]]
name = "refactor_suggest"
agent = "classic_software_engineering"
description = "Get the refactoring plan to execute"

[[related_tools]]
name = "lsp_refactor"
agent = "classic_software_engineering"
description = "Apply and validate refactoring operations via LSP"

[[related_tools]]
name = "lsp_diagnose"
agent = "classic_software_engineering"
description = "Verify no compilation errors after each refactoring step"

[[related_tools]]
name = "code_review"
agent = "classic_software_engineering"
description = "Review each change for correctness and completeness"

[[related_tools]]
name = "quality_check"
agent = "classic_software_engineering"
description = "Verify quality metrics improve after refactoring"

[[related_tools]]
name = "file_edit"
agent = "kalos"
description = "Apply code changes for refactoring"

[[related_tools]]
name = "exec_on_container"
agent = "neikos"
description = "Run test suites to validate refactoring"

[[related_tools]]
name = "script_exec"
agent = "skemma"
description = "Run git operations for atomic commits"

[[related_tools]]
name = "report"
agent = "hubris"
description = "Emit the refactoring result report"

[[related_skills]]
name = "refactoring_guide"
agent = "classic_software_engineering"
description = "Generate the refactoring plan before executing this workflow"
+++

# refactoring_workflow

## Description

Executes a refactoring plan by applying mechanical transforms in atomic steps, validating each step with LSP diagnostics and test execution, and committing verified changes. Ensures each refactoring step is independently reversible and verifiable.

## Preconditions

- Refactoring plan is available (from `refactoring_guide` or manually specified)
- Container is available in Write mode (code modifications required)
- Test suite exists and passes before starting (baseline)
- Git working tree is clean

## SOP

### Step 1: Baseline Verification

```bash
$ exec_on_container(command="cargo test --no-run 2>&1")
$ lsp_diagnose(scope=<target_scope>, language=<lang>)
```

- Verify all tests compile (no pre-existing failures)
- Capture baseline diagnostic count
- **Gate**: If baseline has compilation errors → error `"Fix compilation errors before refactoring"`. Abort.

### Step 2: Plan Loading

```bash
$ refactor_suggest(plan_id=<plan_id>)
```

or accept inline plan from caller.

- Load the refactoring plan with ordered steps
- Validate plan structure: each step has `files`, `transform`, `expected_result`
- **Gate**: If plan has no steps → return `"Empty plan, nothing to execute"`

### Step 3: Step-by-Step Execution

For each step in the plan:

**3a. Apply Transform**

```bash
$ file_edit(file_path=<path>, old=<before>, new=<after>)
```

or for LSP-supported refactorings:

```bash
$ lsp_refactor(file_path=<path>, refactor_type=<type>, range=<range>, params=<params>)
```

**3b. Compilation Check**

```bash
$ lsp_diagnose(file_path=<affected_file>, language=<lang>)
```

- **Gate**: If new compilation errors introduced → revert this step, log failure, continue to next step

**3c. Test Validation**

```bash
$ exec_on_container(command="cargo test <affected_modules> 2>&1")
```

- **Gate**: If tests fail → revert this step, log failure, mark step as "`failed_validation`"

**3d. Quality Check**

```bash
$ quality_check(scope=<affected_files>, metrics=["complexity", "maintainability"])
```

- **Gate**: If quality metrics degraded significantly → log warning, continue (quality regression may be temporary during multi-step refactoring)

**3e. Atomic Commit**

```bash
$ script_exec(command="git add <affected_files> && git commit -m 'refactor(<scope>): <step_description>'")
```

- Each successful step is committed atomically
- If step was reverted → no commit, log in report

### Step 4: Final Validation

After all steps:

```bash
$ exec_on_container(command="cargo test 2>&1")
$ lsp_diagnose(scope=<full_target_scope>, language=<lang>)
$ quality_check(scope=<full_target_scope>, metrics=["complexity", "maintainability", "coupling"])
```

- Full test suite must pass
- No new diagnostic errors
- Quality metrics should show improvement over baseline
- **Gate**: If full test suite fails → identify which step caused regression, revert that commit

### Step 5: Result Report

```bash
$ report(
  summary="Refactoring complete: <S>/<T> steps succeeded, <F> reverted, quality <improved|unchanged|degraded>",
  body=<execution_details_json>,
  severity="low",
  baseline=<baseline_metrics>,
  final=<final_metrics>,
  commits=<commit_hashes>,
  reverted_steps=<failed_step_details>
)
```

## Postconditions

- All successful refactoring steps committed atomically
- Failed steps reverted with no residual changes
- Full test suite passes
- Quality metrics show improvement or stability
- Each commit independently reversible

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Atomic steps**: Every change is committed individually for safe rollback
- **Validate after every step**: Never accumulate unverified changes
- **Revert on failure**: A failed step is reverted immediately, not deferred
- **Full suite validation at end**: Individual module tests during execution, full suite after completion

@system/return-type-convention
