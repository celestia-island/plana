+++
name = "testing_pipeline"
agent = "classic-software-engineering"
execution_mode = "read"

[description]
en = "Orchestrate testing pipeline: analyze test coverage, verify test quality, and generate test improvement recommendations."
zhs = "测试流水线：分析测试覆盖率 + 验证测试质量 + 生成改进建议"
zht = "測試流水線：分析測試覆蓋率 + 驗證測試品質 + 生成改進建議"
ja = "テストパイプライン：テストカバレッジ分析 + テスト品質検証 + 改善提案の生成"
ko = "테스트 파이프라인: 테스트 커버리지 분석 + 테스트 품질 검증 + 개선 권장 사항 생성"
fr = "Pipeline de test : analyser la couverture de tests + vérifier la qualité des tests + générer des recommandations d'amélioration"
es = "Pipeline de pruebas : analizar cobertura de pruebas + verificar calidad de pruebas + generar recomendaciones de mejora"
ru = "Конвейер тестирования : анализ покрытия тестами + проверка качества тестов + генерация рекомендаций по улучшению"

[[related_tools]]
name = "quality_check"
agent = "classic_software_engineering"
description = "Measure test coverage metrics and identify untested code paths"

[[related_tools]]
name = "static_analyze"
agent = "classic_software_engineering"
description = "Detect test anti-patterns: flaky tests, missing assertions, test-only code in production"

[[related_tools]]
name = "code_review"
agent = "classic_software_engineering"
description = "Review test code for meaningful assertions, edge case coverage, and isolation"

[[related_tools]]
name = "lsp_diagnose"
agent = "classic_software_engineering"
description = "Verify test compilation and detect test-specific diagnostics"

[[related_tools]]
name = "exec_on_container"
agent = "neikos"
description = "Run test suites and collect results"

[[related_tools]]
name = "report"
agent = "hubris"
description = "Emit the test quality report"

[[related_skills]]
name = "code_health_check"
agent = "classic_software_engineering"
description = "Assess test file health (size, complexity, duplication) as part of pipeline"
+++

# testing_pipeline

## Description

Orchestrates a comprehensive testing pipeline: analyzes test coverage, verifies test quality, runs test execution, and generates improvement recommendations. Ensures tests are meaningful, not just present — checking for assertions, edge cases, isolation, and maintainability.

## Preconditions

- Target scope is defined (module, crate, or full repo)
- Test suite exists (if not, report "No tests found" and recommend test creation)
- Container with toolchain is available for test execution

## SOP

### Step 1: Test Discovery

```bash
$ file_list(path=<scope>, recursive=true, pattern="**/*test*")
```

- Enumerate all test files and test functions
- Classify: unit tests, integration tests, e2e tests, benchmark tests
- Map test-to-source coverage: which source files have corresponding tests
- **Gate**: If no test files found → return `"No tests found for scope. Recommend creating test suite."`

### Step 2: Test Compilation Check

```bash
$ lsp_diagnose(scope=<test_files>, language=<lang>)
```

- Verify all test files compile without errors
- **Gate**: If test compilation errors found → list errors, set severity = high ("Tests cannot run")

### Step 3: Test Execution

```bash
$ exec_on_container(command="cargo test --workspace 2>&1")
```

- Run full test suite
- Capture: pass/fail counts, execution time per test, failed test details
- **Gate**: If test failures → list failing tests with error messages, set severity based on failure count

### Step 4: Coverage Analysis

```bash
$ quality_check(scope=<scope>, metrics=["line_coverage", "branch_coverage", "function_coverage"])
```

- Measure coverage per module and per file
- Identify: untested public functions, uncovered error paths, missing edge case tests
- **Gate**: If line coverage < 50% → severity = high; if < 80% → severity = medium

### Step 5: Test Quality Review

For each test file:

```bash
$ code_review(file_path=<test_file>, content=<content>)
$ lsp_diagnose(file_path=<test_file>, language=<lang>)
```

- Check for:
  - **Missing assertions**: tests that run code but never verify results (detected via `code_review`)
  - **Flaky patterns**: time-dependent, random, order-dependent tests (detected via `code_review` + `lsp_diagnose` warnings)
  - **Test leakage**: test-only utilities imported in production code (detected via `code_review`)
  - **Hardcoded data**: credentials, paths, environment-specific values in tests (detected via OreXis `security_audit` if scope includes test files)
- **Gate**: If missing assertions found → severity = medium ("Tests provide false confidence")

### Step 6: Improvement Recommendations

- Generate recommendations ranked by impact:
  - **Critical**: Failing tests → fix immediately
  - **High**: Missing test coverage for critical paths → add tests
  - **Medium**: Test quality issues (flaky, missing assertions) → improve existing tests
  - **Low**: Test organization, naming, documentation improvements
- For each recommendation: file, specific action, expected benefit

### Step 7: Report

```bash
$ report(
  summary="Testing pipeline: <P> pass, <F> fail, <S> skip. Coverage: <C>% line, <B>% branch. <N> quality issues.",
  body=<test_report_json>,
  severity=<highest_severity>,
  coverage=<coverage_data>,
  failing_tests=<list>,
  quality_issues=<list>,
  recommendations=<ranked_list>
)
```

## Postconditions

- Complete test quality report with execution results
- Coverage data with gap analysis
- Ranked improvement recommendations
- No test changes applied (read-only analysis)

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Tests must earn confidence**: A test without assertions is worse than no test (false confidence)
- **Coverage is necessary but not sufficient**: 100% coverage with poor assertions is still low quality
- **Flaky tests are failing tests**: Treat flakiness as a bug, not an inconvenience

@system/return-type-convention
