+++
name = "Knowledge Base Health Check"
agent = "philia"

[description]
en = "Validate RAG index quality by checking document counts, embedding dimensions, workspace index status, and detecting stale or corrupted entries. Produces a health report for knowledge base maintenance."
zh-Hans = "通过检查文档数量、嵌入维度、工作空间索引状态以及检测过期或损坏的条目来验证 RAG 索引质量。生成知识库维护的健康报告。"
zh-Hant = "通過檢查文檔數量、嵌入維度、工作空間索引狀態以及檢測過期或損壞的條目來驗證 RAG 索引質量。生成知識庫維護的健康報告。"
ja = "RAG インデックスの品質を検証し、ドキュメント数、埋め込み次元、ワークスペースインデックスの状態、古い/破損したエントリをチェックします。"
ko = "RAG 인덱스 품질을 검증하여 문서 수, 임베딩 차원, 워크스페이스 인덱스 상태, 오래되거나 손상된 항목을 감지합니다."
fr = "Valider la qualité de l'index RAG en vérifiant le nombre de documents, les dimensions d'embedding et l'état de l'index."
es = "Validar la calidad del índice RAG verificando recuentos de documentos, dimensiones de embedding y estado del índice."
ru = "Проверить качество индекса RAG, проверив количество документов, размерности эмбеддингов и статус индексации."

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
