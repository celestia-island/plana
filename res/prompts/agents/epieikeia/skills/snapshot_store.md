+++
name = "Snapshot Lifecycle Management"
agent = "epieikeia"

[description]
en = "snapshot_store is a core skill of the Epieikeia agent, specifically designed to manage the complete lifecycle of container snapshots, including storage, indexing, retrieval, and cleanup. This skill ensures reliable preservation and efficient management of system states, providing a solid foundation for state recovery and version tracking."
zhs = "snapshot_store是Epieikeia智能体的核心技能，专为管理容器快照的完整生命周期而设计，包括存储、索引、检索和清理。该技能确保系统状态的可靠保存和高效管理，为状态恢复和版本追踪提供坚实基础。"
zht = "snapshot_store是Epieikeia智能體的核心技能，專為管理容器快照的完整生命週期而設計，包括儲存、索引、檢索和清理。該技能確保系統狀態的可靠保存和高效管理，為狀態恢復和版本追蹤提供堅實基礎。"
ja = "snapshot_storeはEpieikeiaエージェントのコアスキルであり、コンテナスナップショットの完全なライフサイフサイクル管理（保存、インデックス化、取得、クリーンアップ）に特化しています。このスキルはシステム状態の信頼性の高い保存と効率的な管理を確保し、状態復旧とバージョン追跡の堅固な基盤を提供します。"
ko = "snapshot_store는 Epieikeia 에이전트의 핵심 스킬로, 컨테이너 스냅샷의 전체 수명 주기(저장, 인덱싱, 검색, 정리) 관리에 특화되어 있습니다. 이 스킬은 시스템 상태의 신뢰할 수 있는 보존과 효율적인 관리를 보장하며, 상태 복구 및 버전 추적을 위한 견고한 기반을 제공합니다."
fr = "snapshot_store est une compétence principale de l'agent Epieikeia, conçue spécifiquement pour gérer le cycle de vie complet des instantanés de conteneur, y compris le stockage, l'indexation, la récupération et le nettoyage. Cette compétence assure la préservation fiable et la gestion efficace des états du système, fournissant une base solide pour la récupération d'état et le suivi des versions."
es = "snapshot_store es una habilidad central del agente Epieikeia, diseñada específicamente para gestionar el ciclo de vida completo de las instantáneas de contenedor, incluyendo almacenamiento, indexación, recuperación y limpieza. Esta habilidad asegura la preservación confiable y la gestión eficiente de los estados del sistema, proporcionando una base sólida para la recuperación de estado y el seguimiento de versiones."
ru = "snapshot_store — это основной навык агента Epieikeia, предназначенный для управления полным жизненным циклом снимков контейнера, включая хранение, индексацию, получение и очистку. Этот навык обеспечивает надёжное сохранение и эффективное управление состояниями системы, создавая надёжную основу для восстановления состояния и отслеживания версий."

[[related_tools]]
agent_name = "epieikeia"
tool_name = "deliver_message"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "consume_injected_prompts"

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_close"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "fork_container_on_next_action"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "fork_container_on_next_action"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "list_file_observers"

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
execution_mode = "write"
location = "cosmos"
+++

Manage the full lifecycle of container snapshots — create, index, retrieve, restore, and clean up — ensuring system states are reliably preserved for recovery and auditing.

## SoP

1. **Plan Snapshot** — Determine the container ID, reason for the snapshot (e.g., pre-deployment, periodic, on-error), and labels to apply. Verify target container is reachable; if unreachable, log the failure and abort.
1. **Create Snapshot** — Capture the container state with incremental storage where possible. Apply compression (prefer zstd). Record metadata: container ID, labels, reason, operator, timestamp. If creation fails, retry once; on second failure, use `report_human()` to escalate.
1. **Index and Tag** — Register the snapshot in the index with dimensions: time, labels, container ID, state type. Verify the index entry is retrievable immediately after creation.
1. **Retrieve Snapshot** — When a restore or audit is requested, resolve the snapshot by ID or query (labels + time range). Return the matching snapshot metadata and storage location. If no match is found, report the gap.
1. **Restore Snapshot** — Before restoring, verify snapshot integrity. Create a backup of the current state. Apply the restore. Validate the container state post-restore matches the snapshot manifest. If integrity check fails, do not restore and escalate via `report_human()`.
1. **Enforce Retention Policy** — Periodically scan snapshots against the configured retention policy (max count, max age days, keep labels). Identify candidates for deletion. Preserve snapshots tagged `critical` or `baseline` regardless of policy. Cascade-delete dependent orphaned snapshots.
1. **Report Storage Status** — Generate a lifecycle report (see Output Format) covering snapshot count, storage usage, recent operations, and retention compliance. Use `report()` for automated delivery.

> Return type and IEPL enforcement: @system/return-type-convention
