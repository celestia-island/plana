+++
name = "Parallel Container Conflict Detection and Resolution"
agent = "philia"
[[next_action]]
agent = "skopeo"
name = "node_task_summary"

[description]
en = "This skill specializes in detecting and resolving conflict issues in parallel container operations, ensuring operation consistency and data integrity in multi-container environments."
zh-Hans = "该技能专注于检测和解决并行容器操作中的冲突问题，确保多容器环境下的操作一致性和数据完整性。"
zh-Hant = "該技能專注於偵測和解決平行容器操作中的衝突問題，確保多容器環境下的操作一致性和資料完整性。"
ja = "このスキルは並行コンテナ操作における競合問題の検出と解決に特化し、マルチコンテナ環境での操作の一貫性とデータの完全性を確保します。"
ko = "이 스킬은 병렬 컨테이너 작업의 충돌 문제 감지 및 해결에 특화되어 있으며, 다중 컨테이너 환경에서 작업 일관성과 데이터 무결성을 보장합니다."
fr = "Cette compétence se spécialise dans la détection et la résolution des problèmes de conflit dans les opérations parallèles de conteneurs, assurant la cohérence des opérations et l'intégrité des données dans les environnements multi-conteneurs."
es = "Esta habilidad se especializa en detectar y resolver problemas de conflicto en operaciones paralelas de contenedores, asegurando la consistencia de operaciones y la integridad de datos en entornos de múltiples contenedores."
ru = "Этот навык специализируется на обнаружении и разрешении конфликтных ситуаций при параллельных операциях с контейнерами, обеспечивая согласованность операций и целостность данных в мультиконтейнерных средах."

[[related_tools]]
agent_name = "philia"
tool_name = "memory_query"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_store"

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

Detect and resolve conflicts in multi-container parallel operations to ensure data integrity and operational consistency.

## SoP

1. **Gather context** — Load current container states, shared resource manifests, and lock ownership data via `memory_query()`. Identify all active parallel operations and their access patterns (read/write).
1. **Analyze threats** — Classify detected conflicts by type (write-write, read-write, resource contention). Assess deadlock probability via dependency-cycle detection. Evaluate data corruption risk and cascading failure potential for each conflict.
1. **Decide strategy** — Select a resolution approach: lock-based coordination, optimistic concurrency with retry, last-writer-wins merge, or full serialization. Set lock granularity, timeout thresholds, and retry/backoff parameters.
1. **Execute resolution** — Acquire locks on contested resources, apply the chosen strategy, execute operations within transaction boundaries, synchronize state across containers, then release locks. Rebalance resource allocation post-resolution.
1. **Verify results** — Confirm all conflicts were resolved, validate data integrity across affected containers, ensure all locks were released, and check that no new conflicts were introduced.
1. **Report** — Compile resolution details via `report()`: conflict types, strategies applied, resolution timelines, and resource allocation changes. Escalate unresolved conflicts to `report_human()`.
1. **Capture knowledge** — Persist resolution patterns and effectiveness metrics to `memory_store()` for future conflict prediction and prevention.

> Return type and IEPL enforcement: @system/return-type-convention
