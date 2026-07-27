+++
name = "Instruction Failure Root Cause Backtrace"
agent = "epieikeia"

[description]
en = "`instruction_failure_backtrace` is the core diagnostic skill of the Epieikeia agent, specifically designed to analyze the root causes of instruction execution failures, providing detailed error diagnosis and remediation recommendations. This skill helps quickly locate and resolve faults in complex systems through intelligent backtrace analysis."
zhs = "`instruction_failure_backtrace`是Epieikeia智能体的核心诊断技能，专为分析指令执行失败的根因而设计，提供详细的错误诊断和修复建议。该技能通过智能回溯分析帮助快速定位和解决复杂系统中的故障。"
zht = "`instruction_failure_backtrace`是Epieikeia智能體的核心診斷技能，專為分析指令執行失敗的根因而設計，提供詳細的錯誤診斷和修復建議。該技能透過智慧回溯分析幫助快速定位和解決複雜系統中的故障。"
ja = "`instruction_failure_backtrace`はEpieikeiaエージェントのコア診断スキルであり、命令実行失敗の根本原因を分析し、詳細なエラー診断と修復提案を提供することに特化しています。このスキルはインテリジェントなバックトレース分析により、複雑なシステム内の障害を迅速に特定・解決します。"
ko = "`instruction_failure_backtrace`는 Epieikeia 에이전트의 핵심 진단 스킬로, 명령어 실행 실패의 근본 원인을 분석하고 상세한 오류 진단 및 수정 권고사항을 제공합니다. 이 스킬은 지능형 백트레이스 분석을 통해 복잡한 시스템의 장애를 신속하게 찾아 해결합니다."
fr = "`instruction_failure_backtrace` est la compétence de diagnostic principale de l'agent Epieikeia, conçue spécifiquement pour analyser les causes profondes des échecs d'exécution d'instructions, en fournissant un diagnostic d'erreur détaillé et des recommandations de remédiation. Cette compétence aide à localiser et résoudre rapidement les défauts dans les systèmes complexes grâce à une analyse intelligente de la trace arrière."
es = "`instruction_failure_backtrace` es la habilidad de diagnóstico central del agente Epieikeia, diseñada específicamente para analizar las causas raíz de los fallos en la ejecución de instrucciones, proporcionando diagnóstico de errores detallado y recomendaciones de corrección. Esta habilidad ayuda a localizar y resolver rápidamente fallas en sistemas complejos mediante análisis inteligente de retrotrazado."
ru = "`instruction_failure_backtrace` — это основной навык диагностики агента Epieikeia, предназначенный для анализа корневых причин сбоев выполнения инструкций с предоставлением подробной диагностики ошибок и рекомендаций по исправлению. Этот навык помогает быстро выявлять и устранять неисправности в сложных системах с помощью интеллектуального анализа обратной трассировки."

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
agent_name = "epieikeia"
tool_name = "deliver_message"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "consume_injected_prompts"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[features]
execution_mode = "read"
location = "cosmos"
+++

Diagnose instruction execution failures by capturing error context, tracing the causal chain back to the root cause, and producing prioritized remediation recommendations.

## SoP

1. **Capture Failure Context** — Record the failing instruction ID, error type, error message, timestamp, and full execution context. If context collection partially fails, preserve whatever is available and flag gaps.
1. **Classify Error** — Categorize the error as: `syntax`, `runtime`, `logic`, `timeout`, `resource`, or `dependency`.
1. **Build Causal Chain** — Trace backwards from the surface error through intermediate failures to the root cause. For each hop, record the failing component, the error propagated, and the link to the next hop. If the chain exceeds depth 50, truncate and mark as `max_depth_reached`.
1. **Correlate with History** — Match the current failure pattern against known historical patterns. Report the closest matches with similarity scores. If no match is found, flag for manual review.
1. **Determine Root Cause** — Synthesize the causal chain and historical matches into a single root cause statement. If uncertain, use `report_human()` to request guidance before proceeding.
1. **Generate Remediation Recommendations** — For each identified root cause, produce up to 5 prioritized remediation actions sorted by impact and difficulty. Include a brief rationale for each. If `auto_apply` is appropriate and low-risk, propose a dry-run first.
1. **Report and Archive** — Emit a structured report (see Output Format) via `report()`. For severity >= `critical` or unresolved failures, additionally use `report_human()`. Archive the analysis for future pattern matching.

> Return type and IEPL enforcement: @system/return-type-convention
