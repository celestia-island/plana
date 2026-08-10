+++
name = "regression_monitor"
agent = "classic-software-engineering"
execution_mode = "read"
location = "cosmos"
must_touch_next_action = false

[description]
en = "Monitor codebase for regression and degradation signals across any language: detect declining test-to-fix ratios, flaky test accumulation, test timeout escalation, broken test infrastructure, ignored test proliferation, coverage erosion, AND false positive (vacuous) tests that always pass regardless of code correctness. Auto-detects project toolchain from config files. Predict future degradation risk from trend analysis. Designed for medium-frequency (Daily tier) YOLO invocation."

[[related_tools]]
agent_name = "polemos"
tool_name = "host_command_exec"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_skills]]
agent_name = "classic_software_engineering"
tool_name = "code_health_check"

[[related_skills]]
agent_name = "classic_software_engineering"
tool_name = "testing_pipeline"
+++

# regression_monitor

Monitor the codebase for regression and degradation signals over time. This skill does NOT fix anything — it analyzes trends, detects early warning signs, and reports severity-graded findings. **Works across any language**: auto-detects the project toolchain, then uses heuristic guidance to run each check.

## IMPORTANT: Host Path Convention

Use **host paths** (take the path from the environment section's `Workspace:` line, strip `local://`, and pass it as the `cwd` parameter of `host_command_exec`), NOT container paths (`/workspace`).

## Step 0: DETECT PROJECT AND TOOLCHAIN

Before any checks, determine what the project is and what tools are available:

- Look for build system markers at the project root: `Cargo.toml` (Rust), `package.json` (JS/TS), `go.mod` (Go), `pyproject.toml` (Python), etc.
- Identify the test framework from the project's dependency/config files (don't assume — read the config).
- Determine the "compile-only check" command (e.g. `cargo check`, `tsc --noEmit`, `go build ./...`).
- Determine the "test dry-run that compiles but doesn't execute" command — if the toolchain supports it; if not, note that Step 3 will be skipped.
- Identify source code extensions and test file naming conventions from the actual project structure.
- Identify the test-skip/skip/ignore marker syntax in this language (e.g. `#[ignore]` in Rust, `it.skip` or `test.skip` in JS/TS, `@pytest.mark.skip` in Python, `t.Skip` in Go).

Use THE PROJECT'S OWN TOOLS as declared in its config files. Do not hardcode `cargo`/`npm`/`go` — discover from the project.

## SoP

### Step 1: SCAN GIT LOG FOR FIX-TO-TEST RATIO

Analyze the last 100 commits to compute the fix-to-test ratio:

**Heuristic guidance**:

- Use `git log --oneline -100`. Count commits whose conventional-commit prefix is `fix` vs `test`.
- Also cluster commits into batches of 10 to detect "fix bursts with zero tests".

**Classification** (language-agnostic):

- Fix:Test ratio in last 100 commits
  - 0:10+ → HEALTHY (more tests than fixes = test-driven mindset)
  - 1:5-10 → NORMAL (reasonable balance)
  - 5-10:1 → ELEVATED (fixes far outpace test additions)
  - 10+:1 → SEVERE (tests are not keeping up with bug fixes)
- Batch-level: detect batches of 10 commits with fixes but zero tests
  - 3+ consecutive batches with 0 tests → TREND ALERT

**Gate**: If FIX_COUNT > 40 and TEST_COUNT < 5 → severity = `severe`, tag `test_debt_accumulation`.

### Step 2: DETECT FLAKY TEST PATTERNS

Find tests marked as skipped/ignored and timeout escalation patterns.

**Heuristic guidance**:

- Search the codebase for the language-specific test-skip marker (discovered in Step 0). Count occurrences.
- Search git history (`git log --oneline -200 --grep`) for commits mentioning: "flaky", "ignore", "skip", "increase.*timeout", "timeout.*increase".
- **Timeout escalation detection**: look for commits that repeatedly bump the same test's timeout value. Scan for patterns like consecutive commits modifying the same test file+name with increasing timeout values.

**Analysis**:

- Count skipped/ignored tests total
- Count flaky-test-related commits
- Detect **timeout escalation chain**: commits that repeatedly increase the same test's timeout (e.g., 60s → 120s → 180s → 300s)
  - **Gate**: If any single test has had its timeout increased 3+ times → severity = `high`, tag `timeout_escalation`.

### Step 3: CHECK TEST INFRASTRUCTURE INTEGRITY

Verify tests can actually compile (or pass syntax/lint check, whatever the "no-execute check" is for this toolchain).

**Heuristic guidance**:

- Use the "compile-only check" command discovered in Step 0. Run it scoped to test files if the toolchain supports that. If the toolchain has a "test compile without run" mode, use that.
- If the toolchain cannot separate compilation from execution, run the full check and examine the exit code + error output. Distinguish "compilation/syntax errors" (tests are broken) from "runtime test failures" (tests exist and compile, but assertions fail). Only the former counts for this step.

**Analysis**:

- If the compile check fails with errors in test files → severity = `critical`, tag `test_infrastructure_broken` (tests cannot even compile, regression detection is BLIND)
- If it passes with errors in specific modules/packages → list broken modules, severity = `high`
- **Gate**: If more than 3 modules/packages fail test compilation → severity = `critical`.

### Step 4: DETECT MISSING TEST IMPORTS

Check for recent commits that fixed broken test infrastructure — this pattern indicates tests were broken for an extended period without detection.

**Heuristic guidance**:

- Search git log for commits that add missing imports/requires/includes to test files. Look for keywords in the commit message and diff: "missing import", "add import", "fix import", "add require", "missing dependency" — all in the context of test files.
- Search the current codebase for test files that have import/dependency errors remaining — use the language's linter or the compile check from Step 3 to find them.

**Analysis**:

- Count commits that fix missing test infrastructure
  - 1-2 commits → isolated incidents, acceptable
  - 3-5 commits → pattern emerging, tests are regularly breaking silently
  - 6+ commits → **Gate**: severity = `high`, tag `test_import_decay` (tests break and go unnoticed for extended periods)

### Step 5: MEASURE ENVIRONMENT-DEPENDENT TEST ATTRITION

Detect tests that are being skipped due to environmental dependencies (database not available, Docker not running, GPU not present, etc.).

**Heuristic guidance**:

- Search the codebase for test-skip markers (language-specific, from Step 0). For each hit, examine the skip reason/condition text nearby (the comment, the skip message string, the condition expression).
- Categorize the skip reasons:
  - Environment-gated: requires a specific service (database, cache, queue, container), hardware (GPU, PCI device), or network condition.
  - Feature-gated: behind a compile-time feature flag or build tag — these are intentional, not attrition.
  - Defect-gated: skipped because "test is broken" / "TODO fix" — these are direct regression signals.
- Only count environment-gated and defect-gated skips. Feature-gated skips are fine.

**Analysis**:

- Count environment-gated + defect-gated skipped tests
  - 1-3 → acceptable for a project with real infrastructure dependencies
  - 4-7 → **Gate**: severity = `medium`, tag `env_dependent_test_creep` (too many tests require specific environments to run)
  - 8+ → severity = `high` (most tests are environment-gated, CI coverage is unreliable)

### Step 6: ANALYZE TEST COVERAGE TREND

Assess whether the project's test surface is growing proportionally with its source surface.

**Heuristic guidance**:

- Count test files and source files using the conventions discovered in Step 0. Use the project's own directory structure — monorepos may have per-package test directories.
- Compute the ratio: test files / source files.
- Identify the largest source files (by line count) that have no corresponding test file. "Corresponding" means: same base name in a parallel test directory, or matching `_test`/`.test`/`.spec` suffix convention.
- **Do not calculate code coverage** unless the project already has a coverage tool configured. Instead, use the structural proxy above.

**Analysis**:

- Test-file to source-file ratio
  - >= 0.3 → HEALTHY
  - 0.15-0.3 → ELEVATED
  - < 0.15 → SEVERE
- Identify largest untested source files
  - If any file > 500 lines (or whatever threshold makes sense for this language) has zero corresponding test files → tag `untested_large_file`

### Step 7: DETECT FALSE POSITIVE TESTS (VACUOUS TEST AUDIT)

A **false positive test** is a test that *always passes* regardless of whether the code under test is correct. It pollutes the regression signal: the green bar lies. This step scans the codebase for the structural fingerprints of vacuous tests, computed in addition to the "test count" metrics from Steps 1–6.

**Why this matters**: a project can have 100% "passing" tests yet zero actual regression coverage. Tests that don't assert on the production code path, or that swallow the result they're supposed to verify, are worse than missing tests — they generate false confidence.

**Heuristic guidance** (run language-specific regex/heuristic scans over test files only — exclude non-test source):

1. **Assertion-free tests**. For each test function body, check whether it contains at least one real assertion construct in this language:

   - Rust: `assert!`, `assert_eq!`, `assert_ne!`, `should_panic`, `.unwrap()` / `.expect(...)` on a `Result`/`Option` returned from production code (NOT from a constant).
   - JS/TS: `expect(...)`, `assert(...)`, `chai` `expect`/`should`, `sinon.assert`, jest `expect.assertions(N)`.
   - Python: bare `assert`, `self.assertEqual`/`pytest.raises`/`pytest.fail`.
   - Go: `t.Errorf`, `t.Fatalf`, `require.*` (testify).

Flag any test function whose body has **zero** of these constructs, or whose only "assertion" is the absence of a panic.

1. **Discarded-result patterns** — tests that call the production function but immediately throw away the return value:

   - Rust: `let _ = <prod_call>(...)`, `<prod_call>(...).ok();` (without later inspection), `if let Ok(_) = ... {}` with an empty arm.
   - JS/TS: `void <prodCall>(...)`, `await <prodCall>(...)` whose result is never inspected.
   - Python: bare expression statement of a function call with no comparison.

1. **Tautological literals** — assertions that compare a literal to itself or to a constant that can only be true:

   - `assert!(true)`, `assert_eq!(1, 1)`, `expect(true).toBe(true)`, `assert "x".startswith("x")`.
   - Specifically watch for `assert_eq!(x.len(), N)` immediately after `vec![...; N]` — this asserts the standard library, not the code under test.

1. **Tests that do not invoke the function under test** — the test re-implements the production logic inline and asserts on the re-implemented copy. Detection heuristic:

   - Extract every identifier referenced in assertion arguments; if the test file's own module has a `pub fn`/`export function` whose name does NOT appear inside any assertion expression in any test, flag it.
   - For inline-logic-in-test pattern: if a test body contains a closure/block that mirrors a same-file production expression (e.g., both contain `chars().take(N).collect()`), flag for manual review.

1. **Compile-only / signature tests** — tests whose body is a `let _ = func as fn(...) -> ...;` or `_accept_x(_: &T)` shape. These verify type signatures at compile time and nothing at runtime. They are acceptable as `pub_api_*_compiles` *only if* the file has no other behavioral tests for the same type.
1. **Silent skip-on-missing-tool** — tests that `return Ok(())` / `return` / `eprintln!("skipping")` when an external binary (`git`, `cargo`, `socat`, `tmux`, `kubectl`, `tailscale`, …) is absent. These tests pass in any CI environment that lacks the tool. Distinguish from legitimate `#[ignore]`/`it.skip`/`@pytest.mark.skip` markers (which the test framework reports as ignored, not passed). The silent `return` form is the false-positive form; convert to explicit ignore markers.
1. **Permissive error-substring matching** — `assert!(err.contains("x") || err.contains("y") || ...)` with 4+ alternatives, where almost any error string matches. Flag for tightening to a specific error code or exact message.
1. **`#[ignore]` as primary coverage** — if a function/module has ONLY `#[ignore]`d tests as its coverage, the live test suite provides zero protection. Cross-reference with Step 6 coverage data.

**Analysis** (count hits per pattern):

- 0 hits across the whole codebase → HEALTHY
- 1–3 isolated hits → LOW (note in report, no gate)
- 4–10 hits OR any single pattern concentrated in one module → **Gate**: severity = `medium`, tag `false_positive_test_cluster`
- 11+ hits OR a `*_compiles`-only or `#[ignore]`-only coverage on a public API → **Gate**: severity = `high`, tag `vacuous_test_coverage` (CI is green but the code is effectively untested)

**Recommended automatic action**: when severity ≥ `medium`, include a `FORK_REQUEST` (see Step 9) targeting `testing_pipeline` with scope = the specific flagged files and the pattern detected. The remediation is to *rewrite* the test to call the production function and assert on a concrete, observable result — not to delete the test (which would lose the coverage intent).

**Do NOT auto-fix in this skill**: rewriting tests requires understanding what the production code is supposed to do. This step detects and reports; remediation forks into `testing_pipeline`.

### Step 8: RANK, PREDICT, AND REPORT

Combine all findings and compute a **Regression Risk Score** on a 0-100 scale.

**Scoring Table** (language-agnostic):

| Signal | Weight | Score range per severity band |
| --- | --- | --- |
| Fix-to-test ratio (last 100 commits) | 25% | HEALTHY 0-12 / NORMAL 12-21 / ELEVATED 21-29 / SEVERE 29-33 |
| Flaky test accumulation | 17% | 0 skipped=0 / 1-3=4 / 4-6=10 / 7+ or timeout escalation=17 |
| Test infrastructure integrity | 21% | all clean=0 / 1-2 modules fail=10 / 3-5 fail=17 / broken=21 |
| Missing infrastructure fixes (recent) | 12% | 0-1 commits=0 / 2-3=4 / 4-5=8 / 6+=12 |
| Env-dependent test attrition | 8% | 0-2=0 / 3-4=2 / 5-7=5 / 8+=8 |
| **False positive / vacuous tests (Step 7)** | **17%** | **0 hits=0 / 1-3=3 / 4-10 or cluster=10 / 11+ or vacuous-only coverage=17** |

**Total** = sum of all bands. Maximum 100 (renormalized).

**Trend Detection** (requires historical comparison — if a previous score is available in agent memory or repository data):

- Score DECREASING by >= 5 since last run → IMPROVING
- Score STABLE (change < 5) and < 30 → HEALTHY, maintain current practices
- Score STABLE (change < 5) and 30-50 → WATCH, degradation is steady but not accelerating
- Score INCREASING by >= 10 since last run → ACCELERATING DEGRADATION
- Score > 70 → CRITICAL, test infrastructure collapse imminent

**If no historical data exists**, report the current score and note that trend analysis will start on the next run.

**Report**:

```json
write_to_var({ var_name: "rep", content: "REGRESSION_MONITOR_REPORT_JSON" })
exec({ code: "import { report } from 'hubris'; report({ text: __vars['rep'] });" })
```

Report must include:

- **Project**: the detected project name and primary language(s)
- **Toolchain**: the auto-detected build system, test framework, and compile-check command
- **Regression Risk Score** (0-100) and trend arrow (↑↓→ or "first run")
- **Per-signal breakdown**: score per signal with brief justification
- **Top 3 most concerning findings** with file locations
- **Comparison to previous score** (if available)
- **Recommended actions** ranked by urgency — but do NOT execute them
- If score > 70 → additionally call `report_human` for manual review

### Step 9: FORK DECISION

After generating the report, decide whether any findings warrant forking a remediation task into a `#demiurge.xxx` session.

**Fork criteria** (fork if ALL are true):

1. At least one signal scored **HIGH or SEVERE** (not just ELEVATED)
1. The remediation work is **non-trivial**: it would require adding multiple test files, refactoring multiple modules, or running a multi-step skill chain
1. The remediation scope is **well-defined**: you can name specific files, commits, or modules that need work
1. The work **cannot fit in this tick's remaining budget**: the Daily tick has already been spent on scanning; remediation would need its own session

**Do NOT fork if**:

- All signals are HEALTHY or ELEVATED → just report, no fork needed
- The only finding is "add more tests generally" → too vague, let human prioritize
- The finding requires architecture decisions → use `report_human` instead

**Fork targets by signal type**:

| Finding | Suggested forked skill chain | Scope example |
| --- | --- | --- |
| `test_debt_accumulation` (fix:test ratio severe) | `testing_pipeline` | "Add regression tests for commits [list 5-10 specific fix commits]" |
| `timeout_escalation` (same test timeout raised 3+) | `refactoring_workflow` + `testing_pipeline` | "Refactor [test name] to eliminate timeout dependency, add proper isolation" |
| `test_infrastructure_broken` (tests don't compile) | `auto_fix` | "Fix test compilation in [list broken modules]" |
| `test_import_decay` (6+ import fix commits) | `code_health_check` | "Audit test imports across [list affected packages]" |
| `untested_large_file` (> 500 lines, 0 tests) | `testing_pipeline` | "Add test coverage for [file path]" |
| `false_positive_test_cluster` / `vacuous_test_coverage` (Step 7) | `testing_pipeline` | "Rewrite vacuous tests in [file:list] to call the production function and assert on a concrete observable result. Patterns detected: [assertion-free / discarded-result / tautological-literal / no-call-to-prod-code / compile-only / silent-skip / permissive-error-match]" |

**Fork request format**:

When forking, include a structured block in the report output:

```text
FORK_REQUEST:
  title: "[yolo:regression_monitor] <short description>"
  trigger: "<which signal and what value>"
  scope: "<specific files/commits/modules to address>"
  suggested_skills: ["<skill1>", "<skill2>"]
  severity: "<high|severe|critical>"
```

The engine will parse this block and dispatch a `#demiurge.xxx` session. If the engine does not yet support structured fork dispatch, the block serves as a recommendation in the `report_human` notification — the human can initiate the task manually.

**Commit convention for forked tasks**: commits produced by a YOLO-forked task should use the prefix pattern `test(yolo:regression_monitor):` or `fix(yolo:regression_monitor):` so they are traceable back to the monitoring cycle that triggered them.

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Trends matter more than snapshots**: A single bad metric is a finding; a worsening trend across multiple runs is a crisis
- **Test debt is invisible until it isn't**: Tests don't fail until they're needed — by then it's too late to fix them
- **Flaky tests are failing tests**: A skipped test is technical debt; track it, don't ignore it
- **A green test that asserts nothing is worse than no test**: False positive tests manufacture false confidence. Detecting them is a first-class regression signal — see Step 7
- **Timeout escalation is a leading indicator**: If you keep raising timeouts, you're treating the symptom. The cause is likely growing system complexity or weakening test isolation
- **Never fix in this skill**: This is a monitor, not a repair tool. Findings feed into `auto_fix`, `code_health_check`, and `testing_pipeline` — or into a forked `#demiurge.xxx` task when the work is too big for inline
- **Respect the project's toolchain**: Use whatever tools the project declares in its config files. Don't force `cargo` on a Node.js project or `npm` on a Rust crate. Auto-detect from the project root
- **Fork when the work outgrows the tick**: @system/yolo-fork-pattern — monitor skills detect, forked tasks remediate

> Return type: @system/return-type-convention
> IEPL-first: @system/iepl-first
