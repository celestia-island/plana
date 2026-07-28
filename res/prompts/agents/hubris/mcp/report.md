+++
name = "report"
agent = "hubris"
report_only = true

[description]
en = "Submit results to the skill chain orchestrator — the chain CONTINUES to the next skill. Use this when downstream skills need your output as input."
zhs = "向技能链编排器提交结果——链条会继续到下一个技能。当下游技能需要你的输出作为输入时使用。"
zht = "回報執行結果。路由目標由技能鏈配置自動決定。"
ja = "実行結果を報告する。ルーティング先はスキルチェーン設定により自動決定される。"
ko = "실행 결과를 보고합니다. 라우팅 대상은 스킬 체인 설정에 의해 자동으로 결정됩니다."
fr = "Signaler les résultats d'exécution. La destination est déterminée automatiquement par la configuration de la chaîne de compétences."
es = "Reportar resultados de ejecución. El destino se determina automáticamente por la configuración de la cadena de habilidades."
ru = "Сообщить результаты выполнения. Цель маршрутизации определяется автоматически конфигурацией цепочки навыков."
+++

# report

## Description

Reports the execution results of the current skill. The routing target (next skill or parent) is determined automatically by the skill chain configuration — you do NOT need to specify a target.

You may call this tool **multiple times** during a single skill execution. All calls are aggregated and sent together when the skill's thinking phase ends.

## Parameters

- **text** (string, optional): The execution output, findings, or report content.
- **summary** (string, optional): Short summary of the report.
- **body** (string, optional): Full detailed report body.
- **content** (string, optional): Report content (alternative to text/summary+body). Pass as a JSON string argument using native function calling.

## Returns

### Success

```text
Report recorded
```

### Failure

```text
No content provided for report
```

## Examples

### Example 1: Single report

```json
{ "text": "Task decomposition complete. Found 3 sub-tasks:\n1. Scan workspace structure\n2. Identify key files\n3. Generate summary report" }
```

### Example 2: Multiple reports (aggregated)

```json
{ "text": "Sub-task 1 complete: scanned workspace structure" }
{ "text": "Sub-task 2 complete: identified 12 key files" }
```

## Routing Behavior

- **This tool CONTINUES the chain** — the next skill (workplan_generate → plan_execute) will run after this.
- If you want to TERMINATE the chain and reply directly to the user, use `report_human` instead.
- **Default**: Report is routed to the next skill defined in the skill chain's `next_step` configuration.
- **Terminal skill** (no `next_step`): Report is delivered to the human user via `aporia::summarize_report`.
- You do NOT control routing — it is determined by the skill chain configuration.

## Important Notes

- Every skill must call this tool at least once before finishing
- Use native function calling with JSON arguments — NEVER output `report(...)` as plain text
- Multiple calls are aggregated; all content is combined when thinking ends
- **Language requirement**: When the report will ultimately reach the human user (terminal skill), write `summary` and all content fields in the user's preferred language as declared in the system prompt (via `user_language`, `preferred_language`, or `env.aporia.language`). Do NOT default to English unless the system prompt explicitly sets the preferred language to English.
