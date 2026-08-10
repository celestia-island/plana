+++
name = "Memory Garbage Collection"
agent = "philia"

[description]
en = "Prune stale, orphaned, and low-value memory nodes from the knowledge graph. Identifies unreachable nodes, expired temporal data, and redundant duplicates, then removes them to maintain graph health."

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
