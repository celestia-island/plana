+++
name = "rag_knowledge_retrieve"
agent = "aporia"

[description]
en = "Semantic retrieval and intelligent summarization"

[[related_tools]]
agent_name = "aporia"
tool_name = "rag_db_read"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_skills]]
agent_name = "aporia"
tool_name = "rag_knowledge_ingest"

[[related_skills]]
agent_name = "philia"
tool_name = "rag_knowledge_summarize"

[features]
execution_mode = "read"
location = "cosmos"
+++

Retrieve semantically relevant passages from the current ApoRia knowledge path and synthesize a concise, source-backed summary.

> Current-state note: this describes the intended retrieval workflow. The underlying `rag_db_read` implementation currently scans in-memory vector documents using cosine similarity over the provided embedding.

## SoP

1. Receive the user query and classify its type (keyword, natural language, or code description).
1. Extract key entities and concepts from the query to refine the search intent.
1. Query the target collection using `rag_db_read()` with the query text, specifying top-k (default 10) and similarity threshold (default 0.7).
1. Filter out results below the similarity threshold and re-rank the remaining hits by relevance, freshness, and source diversity.
1. If no results pass the threshold, lower it by 0.1 and retry once; if still empty, report no results found.
1. Synthesize a structured summary from the top-ranked passages using `llm_chat()`, preserving factual accuracy and citing sources.
1. Attach source citations (document ID, file path, section) to each claim in the summary.
1. Evaluate result quality from similarity scores and source diversity.
1. Return the summary with citations and metadata.

> Return type and IEPL enforcement: @system/return-type-convention
