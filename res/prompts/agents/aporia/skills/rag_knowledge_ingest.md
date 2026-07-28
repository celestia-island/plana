+++
name = "rag_knowledge_ingest"
agent = "aporia"

[description]
en = "End-to-end knowledge ingestion workflow: chunking → embedding → storage."
zh-Hans = "端到端知识摄取工作流：分块 → 嵌入 → 存储。"
zh-Hant = "端到端知識攝取工作流程：分塊 → 嵌入 → 儲存。"
ja = "エンドツーエンドのナレッジ取り込みワークフロー：チャンキング → 埋め込み → 保存。"
ko = "엔드투엔드 지식 수집 워크플로우: 청킹 → 임베딩 → 저장."
fr = "Flux de travail d'ingestion de connaissances de bout en bout : découpage → plongement → stockage."
es = "Flujo de trabajo de ingesta de conocimientos de extremo a extremo: segmentación → incrustación → almacenamiento."
ru = "Сквозной рабочий процесс загрузки знаний: разбиение → вложение → хранение."

[[related_tools]]
agent_name = "aporia"
tool_name = "rag_db_write"

[[related_tools]]
agent_name = "aporia"
tool_name = "rag_db_read"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_index"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_status"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_skills]]
agent_name = "aporia"
tool_name = "smart_text_chunker"

[features]
execution_mode = "write"
location = "cosmos"
+++

Ingest raw documents into the ApoRia RAG database. For initial workspace indexing (scanning all files, chunking, embedding, and storing), use `workspace_index`. For targeted document ingestion, use `rag_db_write`.

## SoP

1. **Check current index status**: Call `workspace_status({})` to see if the workspace has been indexed before. If `total_files` is 0 or `last_indexed` is null, proceed to step 2. If the index is current, report health and exit.

1. **Index the workspace**: Call `workspace_index({ workspace_root: '/workspace', full_rebuild: true })`. This scans all workspace files, chunks them, computes embeddings, and stores them in the RAG vector database. Monitor the result for file/chunk counts.

1. **Verify ingestion**: Call `workspace_status({})` again to confirm `total_files > 0` and `total_chunks > 0`.

1. **Probe search quality**: Call `workspace_search({ query: 'main entry point', limit: 3 })` to verify semantic search returns results.

1. **Report**: Report total files indexed, total chunks, embedding dimensions, and any errors encountered.
