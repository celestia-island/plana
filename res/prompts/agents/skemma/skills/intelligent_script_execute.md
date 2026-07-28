+++
name = "intelligent_script_execute"
agent = "skemma"

[description]
en = "Intelligent Script Execution and Result Analysis"
zh-Hans = "智能脚本执行与结果分析"
zh-Hant = "智慧腳本執行與結果分析"
ja = "インテリジェントなスクリプト実行と結果分析"
ko = "지능형 스크립트 실행 및 결과 분석"
fr = "Exécution intelligente de scripts et analyse des résultats"
es = "Ejecución inteligente de scripts y análisis de resultados"
ru = "Интеллектуальное выполнение скриптов и анализ результатов"

[[related_tools]]
agent_name = "skemma"
tool_name = "script_exec"

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
execution_mode = "write"
+++

Execute scripts safely with pre-flight analysis, sandboxed runtime, output capture, and structured result validation.

## SoP

1. **Receive and classify** the script. Identify language (Python, Shell, JavaScript), dependencies, and required environment. If the language or environment is unavailable, stop and report the gap.

1. **Pre-flight analysis**. Read the script source and assess risk — look for dangerous operations (file deletion, network calls, privilege escalation), infinite-loop patterns, and sensitive-data leakage. If a critical security issue is found, block execution immediately and report.

1. **Prepare execution context**. Determine appropriate resource limits (timeout, memory, CPU). Configure sandbox isolation. If the user has not specified limits, apply safe defaults: timeout 300s, memory 2048MB, sandboxed directory.

1. **Execute the script** via `script_exec()`. Pass resource limits and sandbox configuration. Monitor stdout, stderr, and resource consumption in real time.

1. **Handle runtime issues**. If timeout is exceeded, terminate the process and report partial output. If memory limit is breached, kill immediately and report peak usage. On non-zero exit code, capture stderr and classify the error (syntax, runtime, logic).

1. **Validate results**. Check exit code, verify expected output files exist, and confirm output matches the expected schema if one was provided. Flag any unexpected side effects.

1. **Report outcomes**. Produce a structured execution report summarizing status, duration, resource usage, captured output, and any warnings or errors. If the result requires human judgment, escalate via `report_human()`.

> Return type and IEPL enforcement: @system/return-type-convention
