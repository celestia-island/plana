+++
name = "Audit Alignment Skill"
agent = "orexis"

[description]
en = "Audit Alignment Skill is a specialized skill for checking whether code complies with security standards and coding standards. This skill ensures code follows organizational security policies, industry standards, and best practices through automated scanning and rule matching, playing a key role in code review, compliance verification, and quality gates."

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
execution_mode = "read"
location = "cosmos"
+++

Verify that code complies with security standards (OWASP, CERT, CWE) and organizational policies through automated scanning and rule matching.

## SoP

1. **Gather scope** — Identify target code paths, technology stack, dependency manifests, configuration files, and historical audit baselines. If custom rules are missing, fall back to built-in defaults.

1. **Analyze threats** — Identify high-risk code areas (auth, payment, file handling, APIs). Trace data flows for sensitive information paths. Detect insecure patterns: hardcoded secrets, SQL concatenation, eval usage, and configuration misconfigurations. Evaluate code complexity in security-critical modules.

1. **Configure analysis** — Select applicable rule sets based on identified standards and stack. Set scan scope (`full`, `diff-only`, `staged-only`), severity threshold, and quality gate criteria. Allocate scan resources and define false-positive handling strategy.

1. **Execute scans** — Run static code analysis, dataflow analysis, secrets detection, complexity analysis, configuration checks, and dependency compliance audit. Aggregate and deduplicate findings. Classify by severity and standard category.

1. **Verify results** — Cross-verify findings from multiple analysis angles. Confirm severity classifications. Validate remediation suggestions are actionable. Check for false positives and mark accordingly. Ensure file coverage is >= 95%.

1. **Generate report** — Organize findings by severity and standard category. Calculate compliance scores per standard. Provide prioritized remediation recommendations. Determine quality gate pass/fail. Output in requested format.

1. **Capture knowledge** — Extract common violation patterns. Record effective remediation strategies as reusable templates. Update custom rules based on false positive/negative feedback. Store compliance baseline for future trend analysis.

> Return type and IEPL enforcement: @system/return-type-convention
