+++
name = "performance_optimization"
agent = "classic-software-engineering"
execution_mode = "read"

[description]
en = "Identify and resolve performance bottlenecks by combining profiling data, quality metrics, and refactoring suggestions."
zhs = "性能优化：性能分析 + 质量指标 + 重构建议 → 目标优化计划"
zht = "效能最佳化：效能分析 + 品質指標 + 重構建議 → 目標最佳化計畫"
ja = "パフォーマンス最適化：プロファイリング + 品質メトリクス + リファクタリング提案 → 最適化計画"
ko = "성능 최적화: 프로파일링 + 품질 메트릭 + 리팩토링 제안 → 최적화 계획"
fr = "Optimisation des performances : profilage + métriques qualité + suggestions de refactoring → plan d'optimisation"
es = "Optimización de rendimiento : perfilado + métricas de calidad + sugerencias de refactoring → plan de optimización"
ru = "Оптимизация производительности : профилирование + метрики качества + предложения по рефакторингу → план оптимизации"

[[related_tools]]
name = "quality_check"
agent = "classic_software_engineering"
description = "Measure complexity and identify hotspots correlated with performance"

[[related_tools]]
name = "refactor_suggest"
agent = "classic_software_engineering"
description = "Generate optimization-oriented refactoring suggestions"

[[related_tools]]
name = "code_review"
agent = "classic_software_engineering"
description = "Review hot code for algorithmic inefficiency patterns"

[[related_tools]]
name = "static_analyze"
agent = "classic_software_engineering"
description = "Detect anti-patterns: unnecessary clones, O(n²) loops, blocking in async"

[[related_tools]]
name = "lsp_symbols"
agent = "classic_software_engineering"
description = "Map call graphs and identify hot paths"

[[related_tools]]
name = "exec_on_container"
agent = "neikos"
description = "Run profiling commands in the container environment"

[[related_tools]]
name = "report"
agent = "hubris"
description = "Emit the optimization plan report"

[[related_skills]]
name = "refactoring_workflow"
agent = "classic_software_engineering"
description = "Execute performance-related refactoring after optimization plan is approved"
+++

# performance_optimization

## Description

Identifies and resolves code performance bottlenecks by combining profiling data, quality metrics, and refactoring suggestions. Profiles execution hotspots, cross-references them with quality indicators, and produces a targeted optimization plan covering algorithmic improvements, memory management, and I/O efficiency.

## Preconditions

- Target scope is defined (specific functions, module, or hot path)
- Baseline metrics are available or can be collected (execution time, memory usage)
- Container with toolchain and profiling tools is available
- Performance target or acceptable threshold is defined

## SOP

### Step 1: Baseline Profiling

```bash
$ exec_on_container(command="cargo bench -- <benchmark_filter> 2>&1")
```

or for non-Rust:

```bash
$ exec_on_container(command="<language_profiler> <target> 2>&1")
```

- Collect: execution time, memory allocations, syscalls, cache misses
- Identify top-N hotspot functions (default: top 10 by exclusive time)
- **Gate**: If no profiling tools available → fall back to `quality_check` complexity metrics as proxy

### Step 2: Complexity Correlation

```bash
$ quality_check(scope=<hotspot_files>, metrics=["cyclomatic_complexity", "nesting_depth", "function_length"])
```

- Cross-reference profiling hotspots with complexity metrics
- Identify: high-complexity functions that are also performance hotspots (double penalty)
- **Gate**: If complexity and profile data disagree → trust profiling data (complex code may be fast, simple code may be slow)

### Step 3: Anti-Pattern Detection

Performance anti-patterns are detected via profiling, not `static_analyze`:

```bash
$ exec_on_container(command="<language_profiler> <target> 2>&1")
```

- Run profiler with specific flags targeting: unnecessary allocations, blocking calls in async contexts, O(n²) patterns
- Map profiler output to code locations
- **Gate**: If anti-patterns found in hot path → severity = high

### Step 4: Code Review

For each hotspot function:

```bash
$ code_review(file_path=<path>, content=<content>)
```

- Evaluate: algorithm choice, data structure appropriateness, cache locality
- Identify: unnecessary work, redundant computations, early-exit opportunities
- **Gate**: If review identifies algorithmic inefficiency (e.g., O(n²) where O(n log n) exists) → severity = critical

### Step 5: Optimization Plan Generation

```bash
$ refactor_suggest(
  findings=<all_performance_findings>,
  prioritize="impact_ease_ratio",
  constraints=["no_behavior_change", "maintain_api_surface"]
)
```

- Generate suggestions ranked by: `(expected_improvement × confidence) / implementation_effort`
- Each suggestion includes:
  - **What**: specific change (e.g., "Replace Vec::contains with HashSet for lookup-heavy path")
  - **Where**: file, function, line range
  - **Expected impact**: estimated improvement (percentage or absolute)
  - **Risk**: behavior change risk (none / low / medium / high)
  - **Verification**: how to measure the improvement

### Step 6: Report

```bash
$ report(
  summary="Performance optimization: <N> opportunities, estimated <X>% improvement",
  body=<optimization_plan_json>,
  severity="medium",
  baseline_metrics=<profiling_results>,
  opportunities=<ranked_suggestions>
)
```

## Postconditions

- Baseline profiling data captured
- Ranked optimization opportunities with expected impact
- Each suggestion includes verification criteria
- No changes applied (read-only analysis; use `refactoring_workflow` to execute)

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Profile before optimizing**: Never suggest changes without profiling evidence
- **Algorithmic wins over micro-optimization**: Prefer better algorithms over inline tricks
- **Measure, don't guess**: Every suggestion must include a verification method

@system/return-type-convention
