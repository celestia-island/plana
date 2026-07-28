+++
name = "Degradation Detection"
agent = "epieikeia"

[description]
en = "`degradation_check` is a critical skill of the Epieikeia agent, used to detect system degradation status in real-time, automatically triggering degradation strategies to ensure system maintains core service availability when some functions fail. This skill improves system fault tolerance and business continuity through intelligent monitoring and automated response."
zhs = "`degradation_check`是Epieikeia智能体的关键技能，用于实时检测系统降级状态，自动触发降级策略以确保在部分功能失效时系统仍能维持核心服务可用性。该技能通过智能监控和自动响应提升系统的容错能力和业务连续性。"
zht = "`degradation_check`是Epieikeia智能體的關鍵技能，用於即時偵測系統降級狀態，自動觸發降級策略以確保在部分功能失效時系統仍能維持核心服務可用性。該技能透過智慧監控和自動回應提升系統的容錯能力和業務連續性。"
ja = "`degradation_check`はEpieikeiaエージェントの重要なスキルであり、システムの劣化状態をリアルタイムで検出し、一部の機能が失敗した場合でもコアサービスの可用性を維持するために自動的に劣化戦略をトリガーします。このスキルはインテリジェントな監視と自動応答により、システムのフォールトトレランスとビジネス継続性を向上させます。"
ko = "`degradation_check`는 Epieikeia 에이전트의 핵심 스킬로, 시스템 저하 상태를 실시간으로 감지하고 일부 기능이 실패해도 핵심 서비스 가용성을 유지하도록 자동으로 저하 전략을 트리거합니다. 이 스킬은 지능형 모니터링과 자동 응답을 통해 시스템의 내결함성과 비즈니스 연속성을 향상시킵니다."
fr = "`degradation_check` est une compétence critique de l'agent Epieikeia, utilisée pour détecter en temps réel l'état de dégradation du système, en déclenchant automatiquement des stratégies de dégradation pour s'assurer que le système maintient la disponibilité des services de base lorsque certaines fonctions échouent. Cette compétence améliore la tolérance aux pannes et la continuité des activités grâce à une surveillance intelligente et une réponse automatisée."
es = "`degradation_check` es una habilidad crítica del agente Epieikeia, utilizada para detectar el estado de degradación del sistema en tiempo real, activando automáticamente estrategias de degradación para asegurar que el sistema mantenga la disponibilidad de servicios principales cuando algunas funciones fallan. Esta habilidad mejora la tolerancia a fallos y la continuidad del negocio mediante monitoreo inteligente y respuesta automatizada."
ru = "`degradation_check` — это критически важный навык агента Epieikeia, предназначенный для обнаружения состояния деградации системы в реальном времени с автоматическим запуском стратегий деградации для обеспечения доступности основных служб при сбое некоторых функций. Этот навык повышает отказоустойчивость системы и непрерывность бизнес-процессов с помощью интеллектуального мониторинга и автоматизированного реагирования."

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
tool_name = "fork_container_on_next_action"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "list_file_observers"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_skills]]
agent_name = "epieikeia"
tool_name = "instruction_failure_backtrace"

[features]
execution_mode = "read"
location = "cosmos"
+++

Detect system degradation in real-time by collecting metrics, evaluating thresholds, applying mitigation strategies, and producing a structured report.

## SoP

1. **Collect Metrics** — Gather current system performance data: CPU usage, memory usage, disk usage, error rate, response time, and external dependency health. Use cached last-known values if collection fails and flag staleness.
1. **Evaluate Thresholds** — Compare collected metrics against configured thresholds (CPU 80%, memory 85%, disk 90%, error rate 5%, response time 5000ms). If multiple metrics breach, classify at the highest applicable degradation level.
1. **Classify Degradation Level** — Assign one of: `minimal`, `partial`, `severe`, `critical`. When trend data is ambiguous, default to the more conservative (higher) level. Treat timed-out dependency checks as unavailable.
1. **Capture Pre-degradation Snapshot** — Save the current system state before applying any changes. If snapshot capture fails, proceed but log the gap.
1. **Select and Execute Strategy** — Choose the matching degradation strategy for the classified level. Identify non-core services to shut down, apply rate limiting and circuit breaking, and extend cache TTLs. If no matching strategy exists, fall back to the next higher level's strategy. Retry failed commands once; escalate to manual intervention on second failure.
1. **Verify Results** — Confirm non-core services are stopped, rate limits are active, core service response times have improved, and error rates are trending downward. If core services do not improve, escalate to `critical`.
1. **Report and Capture Knowledge** — Generate a degradation report (see Output Format). Record the event, root cause, strategy effectiveness, and update threshold tuning recommendations. Use `report()` for automated delivery and `report_human()` for severity >= `severe`.

> Return type and IEPL enforcement: @system/return-type-convention
