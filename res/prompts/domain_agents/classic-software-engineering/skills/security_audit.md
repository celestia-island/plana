+++
name = "security_audit"
agent = "classic-software-engineering"
execution_mode = "read"

[description]
en = "Orchestrate security auditing: delegate to OreXis for vulnerability scanning, then enrich with code-level context."
zhs = "安全审计编排：委托OreXis扫描漏洞，然后用代码级上下文丰富结果"
zht = "安全審計編排：委託OreXis掃描漏洞，然後用程式碼級上下文豐富結果"
ja = "セキュリティ監査オーケストレーション：OreXisに脆弱性スキャンを委任し、コードレベルのコンテキストで結果を強化"
ko = "보안 감사 오케스트레이션: OreXis에 취약점 스캔을 위임하고 코드 수준 컨텍스트로 결과 보강"
fr = "Orchestration d'audit de sécurité : déléguer à OreXis pour l'analyse des vulnérabilités, puis enrichir avec le contexte au niveau du code"
es = "Orquestación de auditoría de seguridad : delegar a OreXis para el análisis de vulnerabilidades, luego enriquecer con contexto a nivel de código"
ru = "Оркестрация аудита безопасности : делегировать OreXis сканирование уязвимостей, затем обогатить контекстом уровня кода"

[[related_tools]]
name = "security_audit"
agent = "orexis"
description = "Core vulnerability scanning, secret detection, and dependency risk analysis"

[[related_tools]]
name = "static_analyze"
agent = "classic_software_engineering"
description = "Detect code-level security anti-patterns (SQL injection, XSS vectors, unsafe deserialization)"

[[related_tools]]
name = "code_review"
agent = "classic_software_engineering"
description = "Review security-sensitive code paths for authentication, authorization, and data handling issues"

[[related_tools]]
name = "lsp_diagnose"
agent = "classic_software_engineering"
description = "Verify that security fixes do not introduce compilation errors"

[[related_tools]]
name = "report_human"
agent = "hubris"
description = "Escalate critical security findings to human reviewers"

[[related_tools]]
name = "report"
agent = "hubris"
description = "Emit the security audit report"

[[related_skills]]
name = "automated_review"
agent = "classic_software_engineering"
description = "Include security audit as part of full automated review pipeline"
+++

# security_audit

## Description

Orchestrates security auditing by delegating core vulnerability scanning to OreXis, then enriching the results with code-level context from static analysis and code review. Produces a consolidated security report with severity-ranked findings, attack vectors, and remediation guidance.

## Preconditions

- Target scope is defined (file, module, dependency list, or full repo)
- OreXis agent is available for core security scanning
- Container with toolchain is available

## SOP

### Step 1: Core Security Scan (OreXis)

```bash
$ security_audit(scope=<scope>, check_dependencies=true, check_secrets=true, check_configs=true)
```

- Delegate to OreXis for:
  - **Dependency scanning**: CVE database lookup, known vulnerability patterns
  - **Secret detection**: API keys, tokens, passwords, private keys in source
  - **Configuration risks**: insecure defaults, exposed debug endpoints, missing HTTPS
- **Gate**: If OreXis returns critical findings (exposed secrets, critical CVEs) → immediately `report_human` and set `severity = critical`

### Step 2: Code-Level Context Analysis

For security-critical paths flagged by OreXis (auth, crypto, data handling):

```bash
$ code_review(file_path=<path>, content=<content>)
$ static_analyze(file_path=<path>, checks=["error_handling", "dead_code"])
```

- Review code alongside OreXis findings for: input validation gaps, insecure error handling, auth bypass paths
- Use `static_analyze` only for structural issues that compound security risk (dead auth code, swallowed errors)
- **Gate**: If auth bypass or data exposure detected → severity = critical

### Step 3: Finding Correlation

- Merge findings from all three analysis dimensions
- Correlate: same vulnerability reported by multiple tools → increase confidence
- Deduplicate: same (file, line, category) → keep finding with most specific remediation
- Classify by severity:
  - **Critical**: exploitable vulnerabilities, exposed secrets, auth bypass
  - **High**: injection risks, insecure defaults, missing validation
  - **Medium**: deprecated APIs, weak crypto, info leakage
  - **Low**: best practice recommendations, defense-in-depth suggestions

### Step 4: Report

```bash
$ report(
  summary="Security audit: <C> critical, <H> high, <M> medium, <L> low findings",
  body=<security_findings_json>,
  severity=<highest_severity>,
  categories=["vulnerabilities", "secrets", "dependencies", "code_patterns", "configuration"],
  critical_findings=<list_of_items_requiring_immediate_action>
)
```

- Critical findings include: exploit scenario, affected endpoints, remediation steps
- If critical findings exist → also call `report_human` with immediate notification

## Postconditions

- Consolidated security report with severity-ranked findings
- Critical findings escalated to human reviewers
- Each finding includes file location, category, severity, and remediation guidance

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Critical findings are non-negotiable**: Exposed secrets and critical CVEs must be escalated immediately
- **Defense in depth**: Report even low-severity hardening suggestions
- **Code context enriches scan results**: Raw CVE data without code context is less actionable

@system/return-type-convention
