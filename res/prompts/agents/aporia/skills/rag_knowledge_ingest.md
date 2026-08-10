+++
name = "rag_knowledge_ingest"
agent = "aporia"

[description]
en = "End-to-end knowledge ingestion workflow: chunking → embedding → storage."

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
