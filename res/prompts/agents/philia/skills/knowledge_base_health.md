+++
name = "Knowledge Base Health Check"
agent = "philia"

[description]
en = "Validate RAG index quality by checking document counts, embedding dimensions, workspace index status, and detecting stale or corrupted entries. Produces a health report for knowledge base maintenance."

[[related_tools]]
agent_name = "aporia"
tool_name = "rag_db_stats"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_status"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_search"

[features]
execution_mode = "read"
location = "cosmos"
+++

## Knowledge Base Health Check

Automated RAG index quality validation. Runs periodically to ensure the knowledge base is healthy and search results are meaningful.

## SoP

1. **Gather baseline stats**: Call `rag_db_stats` to get:

   - Total documents in the vector store
   - Total media assets
   - Embedding dimensions (should be consistent)
   - Storage backend type

1. **Check workspace index**: Call `workspace_status` to verify:

   - Is the workspace currently being indexed? (should not be stuck)
   - Total files, chunks, bytes indexed
   - Last indexed timestamp (should be recent)
   - Whether indexing is in progress (stale = problem)

1. **Probe search quality**: Execute `workspace_search` with 2-3 canonical queries against known code patterns. Verify:

   - Results are returned (no empty results for broad queries)
   - Top results have relevance scores above 0.5
   - No obvious missing files that should be indexed

1. **Detect anomalies**: Check for:

   - Embedding dimension mismatch (if dims changed between documents)
   - Zero-document state (index was wiped)
   - Stale index (`last_indexed` > 24h ago with active workspace changes)
   - Excessive chunk count (potential over-chunking)

1. **Classify health status**:

   - **HEALTHY**: All checks pass, index is current, search works
   - **DEGRADED**: Some checks fail but core search works (stale index, minor anomalies)
   - **UNHEALTHY**: Core search broken or index empty (needs re-index)

1. **Report**: Generate a structured health report via `report()`:

   - Overall status (HEALTHY/DEGRADED/UNHEALTHY)
   - Document count and growth trend
   - Any anomalies detected with severity
   - Recommended actions (re-index, prune, resize chunks)
