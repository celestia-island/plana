+++
name = "smart_text_chunker"
agent = "aporia"

[description]
en = "Smart text chunking preserving semantic boundaries"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "aporia"
tool_name = "rag_db_write"

[features]
execution_mode = "read"
location = "cosmos"
+++

Split raw text into semantically coherent chunks that preserve boundary integrity, adapting strategy by text type (code, prose, logs) for downstream retrieval workflows.

> Current-state note: this skill describes a target ingestion helper pattern. Downstream embedding and persistence behavior in the current codebase is simpler than the full design narrative sometimes implies.

## SoP

1. Receive the input text and detect its type (code, document, markdown, log) and language/encoding.
1. Validate text quality: check for empty input, encoding anomalies, or excessive size requiring segmentation.
1. Select a chunking strategy based on text type: code-aware (function/class boundaries, ~512 chars), semantic (paragraph/heading boundaries, ~1024 chars), or fixed-size fallback for unstructured text.
1. Configure chunk parameters: target size, overlap ratio (10–20%), and minimum chunk size (100 chars).
1. Detect semantic boundaries (paragraphs, headings, code blocks, function signatures) using structural analysis via `llm_chat()` when heuristic rules are ambiguous.
1. Split text along detected boundaries, applying overlap between adjacent chunks to preserve context continuity.
1. Validate each chunk: verify no mid-sentence or mid-code truncation, check size is within configured bounds, and re-split outliers with adjusted boundaries.
1. Extract metadata for each chunk (position range, length, boundary type, language, source section).
1. Return the chunk list with metadata and a quality summary.

> Return type and IEPL enforcement: @system/return-type-convention
