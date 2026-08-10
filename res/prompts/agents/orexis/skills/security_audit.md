+++
name = "Security Audit"
agent = "orexis"

[description]
en = "Vulnerability Security Audit Workflow is a comprehensive security audit skill for systematically checking security vulnerabilities and compliance issues in codebases, dependencies, and infrastructure configurations. This skill integrates multiple security inspection tools and methodologies, providing end-to-end security assessment capabilities."

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "orexis"
tool_name = "audit_alignment"

[[related_skills]]
agent_name = "orexis"
tool_name = "layer3_preflight_guard"


[features]
execution_mode = "read"
location = "cosmos"
+++

Systematically audit codebases, dependencies, and infrastructure configurations for security vulnerabilities and compliance issues (OWASP Top 10, CWE/SANS Top 25, PCI-DSS, HIPAA, GDPR).

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `deep` | boolean | no | If true, run a deep/full audit (slower but more thorough) |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields. `data` carries the audit report: severity-ranked findings, affected paths, remediation guidance, and the governance checklist verdict.

## Decision Philosophy

When performing security audits:

- **Bias toward defense-in-depth recommendations**: Do not recommend the minimal fix that addresses only the specific vulnerability found. Recommend layered defenses — the immediate code fix, the architectural improvement that prevents the vulnerability class, the monitoring that detects exploitation attempts, and the process change that prevents recurrence. A security audit that fixes one bug is maintenance; one that prevents an entire class of bugs is security engineering.

- **Fearless experimentation**: If the audit reveals architectural security flaws that cannot be adequately addressed by patching individual vulnerabilities, flag them as architectural risks even when only incremental fixes are feasible in the short term. A security audit that silently accepts a fundamentally insecure architecture has failed its purpose — name the architectural risk explicitly so stakeholders can prioritize structural remediation.

- **Sandbox-first validation**: When the remediation plan recommends significant security changes, test them in an isolated environment. Validate that the remediation actually closes the vulnerability and does not introduce new attack surfaces before presenting the plan.

## SoP

1. **Confirm authorization and scope** — Verify audit request authorization. Extract audit scope: code paths, repository URLs, branches, technology stack, dependency manifests, and configuration files. If authorization is unclear, pause and request confirmation via `report_human()`.

1. **Analyze threats** — Scan authentication modules, payment logic, file upload functions, and public API endpoints. Check external dependencies for CVE records. Identify default configurations and weak credentials. Review sensitive information exposure in logs. If zero-day or active attack signs are detected, immediately escalate via `report_human()`.

1. **Select tools and configure** — Determine scan priorities (high-risk areas first). Choose SAST/DAST engines and dependency scanners. Set risk thresholds (critical/high/medium/low) and resource limits (timeout, concurrency). If resources are constrained, focus on highest-risk areas.

1. **Execute scans** — Run SAST scan, dependency vulnerability scan, configuration audit, sensitive information scan, permission and access control check, and compliance verification. Deduplicate results across all scanners. Verify scan completeness. If scan times out, save checkpoint and resume.

1. **Verify results** — Verify vulnerability reproducibility with reproduction steps. Confirm risk rating accuracy. Check dependency fix version availability and compatibility. Cross-verify results from multiple analysis angles. Mark unverified findings as `pending_confirmation`.

1. **Generate report** — Organize vulnerability list sorted by severity. Generate dependency security, configuration audit, and compliance check sections. Provide remediation recommendations with code examples and effort estimates. Output in requested format.

1. **Capture knowledge** — Extract vulnerability patterns and effective remediation strategies. Update vulnerability knowledge base and custom scanning rules. Record false positive patterns. Update secure coding guidelines.

## Governance Checklist

> Mandatory for every audit of a Celestia-island repository. Work through the
> items and report each as `[x]` (pass), `[ ]` (fail), or `[-]` (not applicable).
> A single unchecked item must be listed in the report as a finding.

- [ ] **Credential scan** — `grep -rniE "password|secret|token|api_key|passwd"` over the diff/scope finds no real secrets. Only placeholders (`<your-password>`, `CHANGE_ME`, `sk-xxx`) and RFC 5737 documentation addresses (192.0.2.x / 198.51.100.x / 203.0.113.x) are allowed in tracked files, including comments, examples, defaults, and test data.
- [ ] **Internal IP hardcoding** — No `192.168.x.x` / `10.x.x.x` / `172.16-31.x.x` private addresses in tracked files (docs, configs, install scripts, tests). Where an address is needed, use RFC 5737 documentation addresses.
- [ ] **Workspace-local credentials stay local** — Secrets that live only in local workspace files (e.g. `AGENTS.md`, `PLAN.md`) are never copied into any repository file, including repo-level `AGENTS.md` and README.
- [ ] **Commit message compliance** — All commits follow AGENTS.md §3: `<gitmoji> <Capitalized English summary ending with period.>`; no `fix:`/`feat:` colon prefixes, no CJK, no "Merge branch" subjects.
- [ ] **Dead code / unwired module check** — Modules, hooks, or packages with zero production references are either wired (with the data they need actually reaching them) or removed. A hook that can never receive its input is a finding (E4 lesson: dead hooks must not silently no-op).
- [ ] **Regression tests for security fixes** — Every security remediation ships with a test that reproduces the vulnerability and fails without the fix; the test suite passes with the fix.
- [ ] **Secret rotation on exposure** — If a real credential was ever committed (even after history rewrite), it is reported and rotated; history rewriting without rotation does not count as remediation.
- [ ] **Dependency advisories** — `cargo audit` / equivalent shows no unaddressed known-vulnerability advisories in the locked dependency graph; exceptions are recorded with upstream blockers.
- [ ] **Principle of least privilege** — New code, tools, or hooks request the minimum permission scope; write/exec tools are not granted to read-only or coordinator roles.
- [ ] **Self-modification safety** — Changes that regenerate or self-modify code (surgery chains, codegen, installers) pass a pre-change checkpoint and a post-change build/test validation gate before commit.

> Return type and IEPL enforcement: @system/return-type-convention

```text
```
