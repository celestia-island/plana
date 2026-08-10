+++
name = "architecture_improvement"
agent = "classic-software-engineering"
execution_mode = "read"

[description]
en = "Orchestrate architecture improvement by combining code review, static analysis, and quality metrics into a prioritized improvement plan."

[[related_tools]]
name = "code_review"
agent = "classic_software_engineering"
description = "Collect readability, complexity, and coupling observations"

[[related_tools]]
name = "static_analyze"
agent = "classic_software_engineering"
description = "Detect dead code, circular dependencies, and type-safety gaps"

[[related_tools]]
name = "quality_check"
agent = "classic_software_engineering"
description = "Measure maintainability, technical debt scores, and complexity hotspots"

[[related_tools]]
name = "lsp_symbols"
agent = "classic_software_engineering"
description = "Map module boundaries and symbol ownership"

[[related_tools]]
name = "file_list"
agent = "kalos"
description = "Enumerate source files for scope"

[[related_tools]]
name = "report"
agent = "hubris"
description = "Emit the improvement plan report"

[[related_skills]]
name = "architecture_review"
agent = "classic_software_engineering"
description = "Run architecture review first to establish baseline findings"
+++

# architecture_improvement

## Description

Orchestrates a structured architecture improvement workflow by combining code review insights, static analysis findings, and quality metrics. Identifies systemic structural weaknesses and produces a prioritized improvement plan with concrete action items, phased delivery, and verification criteria.

## Preconditions

- Target module or subsystem is identified (not the full repo — use `automated_review` for full-repo scans)
- Architecture review has been run or baseline findings are available
- Container is available in Read mode (Write mode if prototyping changes)

## SOP

### Step 1: Baseline Assessment

```bash
$ lsp_symbols(file_path=<entry_point>, depth="full")
$ quality_check(scope=<target_module>, metrics=["coupling", "complexity", "cohesion"])
```

- Map all symbols, their boundaries, and inter-module relationships
- Collect coupling scores, cyclomatic complexity, and cohesion metrics
- **Gate**: If `quality_check` returns no data → error `"Cannot assess module: no analyzable source"`

### Step 2: Code Review Scan

For each high-coupling file identified in Step 1:

```bash
$ code_review(file_path=<path>, content=<file_content>)
```

- Focus review on: responsibility violations, layer bypass, god objects, feature envy
- **Gate**: If review identifies `score < 40` → mark as critical improvement target

### Step 3: Static Analysis Pass

```bash
$ static_analyze(file_path=<path>, content=<file_content>)
```

- Detect: dead code, circular imports, unused exports, type-safety gaps
- Cross-reference with coupling data from Step 1 to identify dependency direction violations
- Accumulate into `structural_findings[]`

### Step 4: Correlation and Prioritization

- Merge findings from Steps 1-3
- Correlate: files with both high coupling AND low review scores → critical
- Rank by: `(structural_impact × remediation_ease)` — prefer high-impact, tractable fixes first
- Group into phases:
  - **Phase 1**: Quick wins (dead code removal, import cleanup) — no behavior change
  - **Phase 2**: Boundary enforcement (extract modules, introduce interfaces) — localized change
  - **Phase 3**: Structural restructuring (split services, introduce layers) — cross-cutting change

### Step 5: Plan Generation

```bash
$ report(
  summary="Architecture improvement plan: <N> phases, <M> action items, estimated risk <level>",
  body=<phased_plan_json>,
  severity="medium",
  phases=[phase1, phase2, phase3]
)
```

- Each phase includes: files affected, before/after state description, verification criteria
- **Gate**: Phase 1 items must be independently verifiable with zero risk

## Postconditions

- Phased improvement plan with clear before/after states
- Each phase independently verifiable and deployable
- Risk ranking for each action item

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Bias toward radical solutions**: Prefer fundamentally better architectures over incremental patches
- **Fork-first prototyping**: Fork container to prototype architectural changes before main workspace
- **Multi-branch exploration**: Explore different approaches in parallel container forks

@system/return-type-convention
