+++
name = "Federated Workspace Sync"
agent = "kalos"

[description]
en = "Federated workspace sync skill provides Kalos agent with the ability to sync workspace files across devices and platforms. This skill uses advanced sync algorithms to ensure data consistency, integrity, and efficiency, supporting multi-device collaboration and distributed development scenarios."
zhs = "联邦工作区同步技能为 Kalos 代理提供跨设备和平台同步工作区文件的能力。此技能使用高级同步算法确保数据一致性、完整性和效率，支持多设备协作和分布式开发场景。"
zht = "聯邦工作區同步技能為 Kalos 代理提供跨裝置和平台同步工作區檔案的能力。此技能使用進階同步演算法確保資料一致性、完整性和效率，支援多裝置協作和分散式開發場景。"
ja = "フェデレーテッドワークスペース同期スキルは、Kalosエージェントにデバイスやプラットフォーム間でワークスペースファイルを同期する能力を提供します。このスキルは高度な同期アルゴリズムを使用してデータの整合性、完全性、効率を確保し、マルチデバイスコラボレーションと分散開発シナリオをサポートします。"
ko = "연합 워크스페이스 동기화 스킬은 Kalos 에이전트에게 디바이스 및 플랫폼 간 워크스페이스 파일 동기화 능력을 제공합니다. 이 스킬은 고급 동기화 알고리즘을 사용하여 데이터 일관성, 무결성 및 효율성을 보장하고, 다중 디바이스 협업 및 분산 개발 시나리오를 지원합니다."
fr = "La compétence de synchronisation d'espace de travail fédéré fournit à l'agent Kalos la capacité de synchroniser les fichiers de l'espace de travail entre appareils et plateformes. Cette compétence utilise des algorithmes de synchronisation avancés pour assurer la cohérence, l'intégrité et l'efficacité des données, prenant en charge les scénarios de collaboration multi-appareils et de développement distribué."
es = "La habilidad de sincronización de espacio de trabajo federado proporciona al agente Kalos la capacidad de sincronizar archivos del espacio de trabajo entre dispositivos y plataformas. Esta habilidad utiliza algoritmos de sincronización avanzados para garantizar la consistencia, integridad y eficiencia de los datos, admitiendo escenarios de colaboración multidispositivo y desarrollo distribuido."
ru = "Навык федеративной синхронизации рабочего пространства предоставляет агенту Kalos возможность синхронизировать файлы рабочего пространства между устройствами и платформами. Этот навык использует продвинутые алгоритмы синхронизации для обеспечения согласованности, целостности и эффективности данных, поддерживая сценарии многопользовательской совместной работы и распределенной разработки."

[[related_tools]]
agent_name = "kalos"
tool_name = "file_exists"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

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

[features]
location = "cosmos"
execution_mode = "write"
+++

Sync workspace files across devices and platforms while preserving consistency, integrity, and conflict safety.

## SoP

1. **Discover scope** — Use `file_list()` and `file_exists()` to enumerate local and remote workspace paths. Collect file lists with metadata from each side.
1. **Read configuration** — Use `file_read()` to load sync config (exclude patterns, conflict strategy, priority rules). Apply defaults when config is missing.
1. **Detect changes** — Use `file_get_info()` to compare timestamps and sizes. Use `file_read()` to compute content hashes for files that differ in metadata. Classify each file as `added`, `modified`, `deleted`, or `unchanged`.
1. **Analyze conflicts** — Identify files modified on both sides since last sync. Classify conflict type: `edit-edit`, `edit-delete`, or `rename-rename`. Assess risk level.
1. **Resolve conflicts** — For low-risk auto-mergeable conflicts, apply three-way merge using content from both sides plus the base version. For high-risk conflicts, use `report_human()` to present options (keep-local, keep-remote, manual merge).
1. **Execute sync** — For each non-conflicting change, use `file_read()` on source and `file_write()` on target. Process in dependency order (directories first, then files). Handle large files with chunked reads/writes.
1. **Verify integrity** — After writing each file, use `file_read()` to re-hash and compare against the source hash. Flag any mismatch for re-transfer.
1. **Generate report** — Use `report()` to produce a structured sync summary. Use `report_human()` to surface conflicts and failures that require attention.

> Return type and IEPL enforcement: @system/return-type-convention

## Edge Cases

- **No sync config**: Apply sensible defaults, report what defaults were assumed
- **First sync**: Treat all files as new, no baseline comparison needed
- **Network/unreachable remote**: Report what's available locally, note what couldn't be checked
- **Permission errors**: Report per-file, suggest `report_human()` for resolution
