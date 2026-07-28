+++
name = "Code Review Standard"
agent = "kalos"

[description]
en = "Standardized code review skill provides Kalos agent with the ability to perform systematic, consistent code reviews. This skill provides code quality, security, and maintainability checking capabilities through predefined check rules and practical experience."
zh-Hans = "标准化代码审查技能为 Kalos 代理提供系统化、一致的代码审查能力。此技能通过预定义的检查规则和实践经验，提供代码质量、安全性和可维护性检查能力。"
zh-Hant = "標準化程式碼審查技能為 Kalos 代理提供系統化、一致的程式碼審查能力。此技能透過預定義的檢查規則和實踐經驗，提供程式碼品質、安全性和可維護性檢查能力。"
ja = "標準化されたコードレビュースキルは、Kalosエージェントに体系的で一貫性のあるコードレビューを実行する能力を提供します。このスキルは事前定義されたチェックルールと実践的経験を通じて、コード品質、セキュリティ、保守性のチェック機能を提供します。"
ko = "표준화된 코드 리뷰 스킬은 Kalos 에이전트에게 체계적이고 일관된 코드 리뷰를 수행할 수 있는 능력을 제공합니다. 이 스킬은 사전 정의된 검사 규칙과 실무 경험을 통해 코드 품질, 보안 및 유지보수성 검사 기능을 제공합니다."
fr = "La compétence de revue de code standardisée fournit à l'agent Kalos la capacité d'effectuer des revues de code systématiques et cohérentes. Cette compétence fournit des capacités de vérification de la qualité, de la sécurité et de la maintenabilité du code via des règles de vérification prédéfinies et de l'expérience pratique."
es = "La habilidad estandarizada de revisión de código proporciona al agente Kalos la capacidad de realizar revisiones de código sistemáticas y consistentes. Esta habilidad proporciona capacidades de verificación de calidad, seguridad y mantenibilidad del código a través de reglas de verificación predefinidas y experiencia práctica."
ru = "Навык стандартизированной проверки кода предоставляет агенту Kalos возможность выполнять систематические и согласованные ревью кода. Этот навык обеспечивает проверку качества, безопасности и поддерживаемости кода на основе предопределенных правил и практического опыта."

[[related_tools]]
agent_name = "kalos"
tool_name = "file_exists"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_list"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_get_info"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[features]
location = "cosmos"
execution_mode = "read"
+++

Perform systematic code review covering quality, security, and maintainability.

## Decision Philosophy

When performing code reviews:

- **Bias toward actionable, high-impact findings**: Do not drown the review in low-severity style observations that could be automated by a linter. Focus review effort on findings that demand human judgment: design flaws, security vulnerabilities, correctness risks, and structural problems. A review with 5 high-quality findings is more valuable than one with 50 lint-level nits.

- **Fearless experimentation**: If the review reveals fundamental design problems, recommend restructuring rather than suggesting patches. A code review that rubber-stamps a flawed design is failing its purpose. Be willing to recommend significant, even disruptive, changes when the code quality justifies it.

- **Concrete, verifiable recommendations**: Every recommendation should reference the specific file path and line range it addresses, include a minimal code example of the proposed change, and describe how to verify the fix (test to run, condition to check). Abstract advice ("improve error handling") without a concrete target is noise.

- **Contextual depth over checklist breadth**: Understand the code's architectural role and change intent before applying standard rules. A review that misses a design-level problem but catches 20 style nits has failed. Prioritize findings that require human judgment over those a linter would catch.

## SoP

1. **Scope discovery** — Use `file_list()` and `file_exists()` to identify all target files. Determine language and framework from extensions and config files.
1. **Read baseline** — Use `file_read()` to load lint configs (`.eslintrc`, `pyproject.toml`, etc.) and establish rule sets. If no config exists, apply sensible defaults.
1. **Read source files** — Use `file_read()` to read each file in scope. For large changesets, batch by directory and process incrementally.
1. **Static analysis** — For each file, analyze: syntax correctness, unused imports, dead code, overly complex functions (high cyclomatic complexity, deep nesting), and magic numbers/strings.
1. **Security scan** — Check for hardcoded secrets (keys, tokens, passwords), injection vectors (SQL, XSS, command injection), insecure defaults, and missing input validation/output encoding.
1. **Dependency review** — Use `file_read()` on manifest files (`package.json`, `requirements.txt`, `Cargo.toml`) to identify outdated or known-vulnerable dependencies.
1. **Style and consistency** — Compare code style against project config. Flag inconsistent naming, formatting, and missing documentation.
1. **Duplication detection** — Identify repeated logic blocks across files and suggest consolidation.
1. **Severity classification** — Assign each finding one of: `critical`, `high`, `medium`, `low`, `info`. Critical = active exploit risk; High = likely bug or data leak; Medium = maintainability concern; Low = style nit; Info = observation.
1. **Generate report** — Use `report()` to produce a structured review document. Use `report_human()` to surface critical findings immediately.

> Return type and IEPL enforcement: @system/return-type-convention

## Edge Cases

- **No files to review**: Report empty scope, ask for clarification
- **Large codebase**: Prioritize recently modified files, limit scope, note what was excluded
- **Mixed languages**: Handle each language's conventions separately
- **No lint config**: Apply sensible defaults for the detected language, state that project config is missing
