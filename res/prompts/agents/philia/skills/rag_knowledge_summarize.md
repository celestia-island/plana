+++
name = "RAG Knowledge Summarization"
agent = "philia"
[[next_action]]
agent = "skopeo"
name = "node_task_summary"

[description]
en = "This skill uses Retrieval-Augmented Generation (RAG) technology to extract key information from large documents and knowledge bases, generating structured knowledge summaries to improve information retrieval and knowledge management efficiency."
zh-Hans = "该技能使用检索增强生成（RAG）技术从大型文档和知识库中提取关键信息，生成结构化知识摘要，以提高信息检索和知识管理效率。"
zh-Hant = "該技能使用檢索增強生成（RAG）技術從大型文件和知識庫中擷取關鍵資訊，產生結構化知識摘要，以提高資訊檢索和知識管理效率。"
ja = "このスキルは検索拡張生成（RAG）技術を使用して、大規模な文書やナレッジベースから重要な情報を抽出し、構造化された知識サマリーを生成して情報検索とナレッジ管理の効率を向上させます。"
ko = "이 스킬은 검색 증강 생성(RAG) 기술을 사용하여 대규모 문서 및 지식 베이스에서 핵심 정보를 추출하고, 구조화된 지식 요약을 생성하여 정보 검색 및 지식 관리 효율을 향상시킵니다."
fr = "Cette compétence utilise la technologie de génération augmentée par récupération (RAG) pour extraire des informations clés de documents volumineux et de bases de connaissances, générant des résumés de connaissances structurés pour améliorer l'efficacité de la recherche d'informations et de la gestion des connaissances."
es = "Esta habilidad utiliza tecnología de Generación Aumentada por Recuperación (RAG) para extraer información clave de documentos grandes y bases de conocimientos, generando resúmenes de conocimiento estructurados para mejorar la eficiencia de recuperación de información y gestión del conocimiento."
ru = "Этот навык использует технологию генерации с дополненной выборкой (RAG) для извлечения ключевой информации из крупных документов и баз знаний, создавая структурированные сводки знаний для повышения эффективности поиска информации и управления знаниями."

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
