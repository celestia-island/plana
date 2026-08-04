+++
name = "Audit Alignment Skill"
agent = "orexis"

[description]
en = "Audit Alignment Skill is a specialized skill for checking whether code complies with security standards and coding standards. This skill ensures code follows organizational security policies, industry standards, and best practices through automated scanning and rule matching, playing a key role in code review, compliance verification, and quality gates."
zh-Hans = "审计对齐技能是一项专门技能，用于检查代码是否符合安全标准和编码规范。该技能通过自动化扫描和规则匹配确保代码遵循组织安全策略、行业标准和最佳实践，在代码审查、合规验证和质量门禁中发挥关键作用。"
zh-Hant = "審計對齊技能是一項專門技能，用於檢查程式碼是否符合安全標準和編碼規範。該技能透過自動化掃描和規則匹配確保程式碼遵循組織安全策略、產業標準和最佳實踐，在程式碼審查、合規驗證和品質門禁中發揮關鍵作用。"
ja = "監査アライメントスキルは、コードがセキュリティ標準とコーディング標準に準拠しているかを確認する専門スキルです。このスキルは自動スキャンとルールマッチングにより、コードが組織のセキュリティポリシー、業界標準、ベストプラクティスに従うことを確保し、コードレビュー、コンプライアンス検証、品質ゲートで重要な役割を果たします。"
ko = "감사 정렬 스킬은 코드가 보안 표준 및 코딩 표준을 준수하는지 확인하는 전문 스킬입니다. 이 스킬은 자동 스캔 및 규칙 매칭을 통해 코드가 조직 보안 정책, 업계 표준 및 모범 사례를 따르도록 보장하며, 코드 검토, 규정 준수 검증 및 품질 게이트에서 핵심적인 역할을 합니다."
fr = "La compétence d'alignement d'audit est une compétence spécialisée pour vérifier si le code respecte les normes de sécurité et les normes de codage. Cette compétence assure que le code suit les politiques de sécurité organisationnelles, les normes de l'industrie et les meilleures pratiques grâce à une analyse automatisée et au matching de règles, jouant un rôle clé dans la revue de code, la vérification de conformité et les portes de qualité."
es = "La habilidad de Alineación de Auditoría es una habilidad especializada para verificar si el código cumple con los estándares de seguridad y los estándares de codificación. Esta habilidad asegura que el código siga las políticas de seguridad organizacionales, estándares de la industria y mejores prácticas mediante escaneo automatizado y coincidencia de reglas, jugando un papel clave en la revisión de código, verificación de cumplimiento y puertas de calidad."
ru = "Навык аудита соответствия — это специализированный навык для проверки соответствия кода стандартам безопасности и стандартам кодирования. Этот навык обеспечивает соблюдение кодом политик безопасности организации, отраслевых стандартов и лучших практик посредством автоматизированного сканирования и сопоставления правил, играя ключевую роль в код-ревью, проверке соответствия и контроле качества."

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
