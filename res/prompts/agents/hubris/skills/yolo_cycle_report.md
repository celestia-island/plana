+++
name = "YOLO Cycle Meta-Report"
agent = "hubris"

[description]
en = "Analyze YOLO auto-cruise cycle results and produce a meta-report summarizing system health, skill execution outcomes, token usage trends, and anomalies across all YOLO tiers."
zh-Hans = "分析 YOLO 自动巡航周期结果，生成元报告，汇总系统健康状态、技能执行结果、Token 使用趋势和跨层异常。"
zh-Hant = "分析 YOLO 自動巡航週期結果，生成元報告，匯總系統健康狀態、技能執行結果、Token 使用趨勢和跨層異常。"
ja = "YOLO 自動クルーズサイクルの結果を分析し、システムヘルス、スキル実行結果、トークン使用傾向、異常のメタレポートを生成します。"
ko = "YOLO 자동 순환 주기 결과를 분석하여 시스템 상태, 스킬 실행 결과, 토큰 사용 추세 및 이상에 대한 메타 보고서를 생성합니다."
fr = "Analyser les résultats du cycle YOLO et produire un méta-rapport sur la santé du système."
es = "Analizar los resultados del ciclo YOLO y producir un meta-informe sobre la salud del sistema."
ru = "Анализировать результаты цикла YOLO и создать мета-отчёт о состоянии системы."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "list_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "create_todo"

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_close"

[features]
execution_mode = "read"
location = "cosmos"
+++

## YOLO Cycle Meta-Report

Produces a summary of YOLO auto-cruise activity across all tiers (Periodic + Daily). This is a meta-analysis skill — it examines what YOLO has been doing and reports trends, not individual results.

## SoP

1. **Gather cycle history**: Review the YOLO daemon's recent execution log. For each tier:

   - Periodic (1h): How many cycles completed in the last 24 hours?
   - Daily (6h): How many cycles completed in the last 24 hours?
   - For each completed cycle: which skills ran, did they succeed or fail?

1. **Token usage analysis**: Summarize cumulative token consumption by YOLO:

   - Total tokens used by Periodic tier skills
   - Total tokens used by Daily tier skills
   - Compare to previous cycle (is usage trending up/down/stable?)
   - Flag any single skill that consumed >50% of the cycle's total tokens

1. **Skill success rate**: For each YOLO skill, compute:

   - Success count / total invocations
   - Average execution time
   - Any skills that failed consecutively (2+ failures = needs attention)

1. **Anomaly detection**: Flag:

   - Skills that consistently time out (may need SoP simplification)
   - Skills that produce empty reports (may need tool access fixes)
   - Any tier that was disabled or skipped during the period
   - Unusual token spikes (>2x average for a single invocation)

1. **System health indicators**: Derive from skill results:

   - Memory graph: growing/stable/shrinking (from `memory_consolidate` output)
   - RAG index: healthy/degraded (from `knowledge_base_health` output)
   - Security: issues found/issues resolved (from `security_audit` output)
   - Code quality: violations found/fixed (from `code_review_standard` output)

1. **Tech debt scan**: Query TODO items with `tech_debt` tag:

   ```typescript
   exec({ code: "import { list_todo } from 'hubris'; const todos = await list_todo({ view: 'tree' }); const debts = todos.data?.items?.filter(t => t.metadata?.tags?.includes('tech_debt')) || []; console.log(JSON.stringify(debts, null, 2));" })
   ```

   - If `tech_debt` items exist and no critical YOLO failures are blocking: **dispatch the highest-priority `tech_debt` item for repayment** by reporting it as a recommendation.
   - If a `tech_debt` item is stale (>7 days old) or blocked: flag it for human review.

1. **Generate report**: Use `report()` with type `system` containing:

   - Executive summary (1-2 sentences)
   - Per-tier execution table (skill, status, tokens, duration)
   - **Tech debt inventory** (count, highest priority item, recommended action)
   - Anomaly list (if any)
   - Trend indicators (token usage, success rate)
   - Recommendations (if any skills need adjustment)

## Decision Philosophy

- **Observation only**: This skill never modifies state. It reads and reports.
- **Actionable over verbose**: If everything is healthy, the report should be short. Only elaborate on anomalies.
- **Trend-aware**: A single failure is noise. Consecutive failures are signal.
