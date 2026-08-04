+++
name = "Memory Garbage Collection"
agent = "philia"

[description]
en = "Prune stale, orphaned, and low-value memory nodes from the knowledge graph. Identifies unreachable nodes, expired temporal data, and redundant duplicates, then removes them to maintain graph health."
zh-Hans = "从知识图谱中清除过时、孤立和低价值的记忆节点。识别不可达节点、过期时序数据和冗余副本，然后删除以维护图谱健康。"
zh-Hant = "從知識圖譜中清除過時、孤立和低價值的記憶節點。識別不可達節點、過期時序數據和冗餘副本，然後刪除以維護圖譜健康。"
ja = "ナレッジグラフから古い、孤立した、低価値のメモリノードを整理します。"
ko = "지식 그래프에서 오래되고 고아되며 가치가 낮은 메모리 노드를 정리합니다."
fr = "Élaguer les nœuds de mémoire obsolètes, orphelins et de faible valeur du graphe de connaissances."
es = "Podar nodos de memoria obsoletos, huérfanos y de bajo valor del grafo de conocimiento."
ru = "Обрезать устаревшие, потерянные и низкоценные узлы памяти из графа знаний."

[[related_tools]]
agent_name = "philia"
tool_name = "memory_query"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_store"

[features]
execution_mode = "write"
location = "cosmos"
+++

## Memory Garbage Collection

Automated memory graph maintenance skill. Runs periodically to keep the knowledge graph healthy.

## SoP

1. **Scan for orphan nodes**: Use `memory_query` with `subgraph: true` to retrieve the full node+edge structure. Identify nodes that have zero incoming and zero outgoing edges — these are orphans with no relational context.

1. **Identify stale temporal data**: Check memory nodes with timestamps older than the retention window (default: 30 days for raw observations, 90 days for derived insights). Nodes that have been superseded by newer consolidated episodes are candidates.

1. **Detect near-duplicate nodes**: Query for nodes with embedding similarity above 0.95 that share the same entity type. Flag duplicate entries that were stored by different ingestion paths.

1. **Classify candidates**: For each flagged node, classify as:

   - **ORPHAN**: No edges, no references from any episode
   - **STALE**: Superseded by a consolidation, older than retention window
   - **DUPLICATE**: Near-identical to another node with higher connectivity

1. **Prune with safety margin**: Never remove nodes that:

   - Are referenced by any active episode (last 7 days)
   - Have edge degree > 3 (well-connected nodes)
   - Were created within the last 24 hours (too new to evaluate)

1. **Execute removal**: For each pruned node, log the node ID, type, reason, and timestamp. Remove edges first, then the node itself.

1. **Report**: Generate a summary via `report()`:

   - Total nodes before/after
   - Pruned count by category (orphan/stale/duplicate)
   - Graph health metrics (average degree, connectivity ratio)

## Decision Philosophy

- **Conservative by default**: When uncertain, keep the node. Memory is cheap; lost context is expensive.
- **Never prune during active sessions**: If any agent session is in progress, defer GC to next cycle.
- **Idempotent**: Running GC twice produces the same result — no double-deletion risk.
