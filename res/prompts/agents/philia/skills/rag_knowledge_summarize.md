+++
name = "RAG Knowledge Summarization"
agent = "philia"
[[next_action]]
agent = "skopeo"
name = "node_task_summary"

[description]
en = "This skill uses Retrieval-Augmented Generation (RAG) technology to extract key information from large documents and knowledge bases, generating structured knowledge summaries to improve information retrieval and knowledge management efficiency."

[[related_tools]]
agent_name = "philia"
tool_name = "memory_query"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_store"

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
location = "cosmos"
execution_mode = "read"
+++

Extract, condense, and integrate key information from large documents and knowledge bases into structured summaries using retrieval-augmented generation.

## SoP

1. **Gather context** — Load existing knowledge base schema, indexing status, and related prior summaries via `memory_query()`. Identify source documents, their formats, and user-specified focus areas. Use `report_human()` to confirm summary requirements and detail level.
1. **Analyze quality risks** — Assess source reliability, detect potential contradictions across sources, evaluate information-loss risk during summarization, and identify hallucination risk. Flag low-credibility sources and coverage gaps.
1. **Decide strategy** — Select retrieval approach (vector-first, keyword-first, hybrid), summarization mode (extractive, abstractive, hybrid), summary parameters (length, style, focus areas), and deduplication/conflict-resolution rules. Set quality gates: minimum source count and coverage thresholds.
1. **Execute summarization** — Retrieve relevant content chunks, merge and deduplicate results, generate the knowledge summary via `llm_chat()`, and integrate into the knowledge base. Validate summary coverage against original source scope.
1. **Verify results** — Cross-reference summary against source documents for factual accuracy. Confirm all focus areas are addressed, length constraints are met, and no hallucinated content is present. If coverage is below threshold, re-retrieve missed content and regenerate.
1. **Report** — Output the final summary with source attributions via `report()`. Include retrieval statistics, quality metrics (coverage, accuracy, compression ratio), and recommendations for supplementary sources. Use `report_human()` for summaries requiring manual review.
1. **Capture knowledge** — Store the summary, retrieval patterns, and quality benchmarks to `memory_store()`. Update the knowledge index for improved future retrieval.

> Return type and IEPL enforcement: @system/return-type-convention
