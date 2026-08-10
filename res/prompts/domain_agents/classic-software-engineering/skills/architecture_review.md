+++
name = "architecture_review"
agent = "classic-software-engineering"
execution_mode = "read"

[description]
en = "Comprehensive architecture review combining dependency analysis, quality metrics, and symbol mapping to evaluate system structure."

[[related_tools]]
name = "lsp_symbols"
agent = "classic_software_engineering"
description = "Map module structure, symbol ownership, and boundaries"

[[related_tools]]
name = "static_analyze"
agent = "classic_software_engineering"
description = "Detect circular dependencies, unused exports, and import direction"

[[related_tools]]
name = "quality_check"
agent = "classic_software_engineering"
description = "Measure maintainability, technical debt, and complexity metrics"

[[related_tools]]
name = "file_list"
agent = "kalos"
description = "Enumerate source files for the target scope"

[[related_tools]]
name = "report"
agent = "hubris"
description = "Emit the architecture review report"

[[related_skills]]
name = "architecture_improvement"
agent = "classic_software_engineering"
description = "Follow up with improvement plan when review finds architectural drift"
+++

# architecture_review

## Description

Performs a comprehensive architecture review by combining dependency inspection, quality metrics, and symbol mapping. Evaluates system structure for scalability, maintainability, and resilience, producing a structured report with risk-ranked findings and improvement recommendations.

## Preconditions

- Target scope is defined: `module_path` or `full_repo`
- Container with toolchain is available
- Source code is parseable (no syntax errors that block LSP)

## SOP

### Step 1: Module Inventory

```bash
$ file_list(path=<scope>, recursive=true, extensions=[".rs", ".ts", ".py", ".go"])
```

- Enumerate all source files in scope
- Filter to supported languages
- Build a file tree representing the module structure
- **Gate**: If no source files found → error `"No analyzable source in scope"`

### Step 2: Symbol and Boundary Mapping

For each entry-point file and module root:

```bash
$ lsp_symbols(file_path=<path>, depth="full")
```

- Catalog all public symbols, their types (function, struct, trait, interface, class)
- Map symbol ownership: which module owns which symbols
- Identify module boundaries and public API surface
- **Gate**: If LSP fails to resolve symbols → log warning, fall back to static text parsing

### Step 3: Dependency Analysis

For each file in scope:

```bash
$ static_analyze(file_path=<path>, content=<file_content>)
```

- Extract import/dependency graph from all modules
- Detect: circular dependencies, transitive dependency chains, unused exports
- Flag cross-layer imports (e.g., UI importing data layer directly)
- **Gate**: If circular dependency detected → severity = critical

### Step 4: Quality Metrics Assessment

```bash
$ quality_check(scope=<scope>, metrics=["maintainability", "complexity", "coupling", "cohesion"])
```

- Collect maintainability index per module
- Identify complexity hotspots (functions/files with high cyclomatic complexity)
- Compute coupling metrics: afferent (incoming) and efferent (outgoing) coupling
- **Gate**: If maintainability index < 20 for any module → flag as "critical debt"

### Step 5: Finding Correlation and Report

- Merge findings from Steps 2-4
- Correlate structural weaknesses with quality regressions:
  - High coupling + low maintainability → structural issue
  - Circular deps + high complexity → architectural anti-pattern
- Rank findings by: `irreversibility × blast_radius × current_impact`
- Eliminate redundant reports (same root cause → single finding)

```bash
$ report(
  summary="Architecture review: <N> findings, risk level <level>",
  body=<structured_findings_json>,
  severity=<highest_severity>,
  categories=["coupling", "layering", "complexity", "dead_code"]
)
```

## Postconditions

- Architecture review report with risk-ranked findings
- Dependency graph visualization data
- Recommended remediation paths with effort estimates

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Focus on expensive-to-reverse decisions**: Defer style preferences, prioritize structural issues
- **Validate against runtime behavior**: Dependency findings should be cross-checked, not just import graphs
- **Delegate file-level health to `code_health_check`**: When architecture is sound but files are unhealthy

@system/return-type-convention
