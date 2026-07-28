+++
name = "web_search_summarize"
agent = "eleos"
[[next_action]]
agent = "skopeo"
name = "node_task_summary"

[description]
en = "Web Search and Summarization"
zh-Hans = "网页搜索与摘要"
zh-Hant = "網頁搜尋與摘要"
ja = "Web検索と要約"
ko = "웹 검색 및 요약"
fr = "Recherche web et résumé"
es = "Búsqueda web y resumen"
ru = "Веб-поиск и краткое изложение"

[[related_tools]]
agent_name = "eleos"
tool_name = "web_search"

[[related_tools]]
agent_name = "eleos"
tool_name = "web_fetch"

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
execution_mode = "read"
location = "cosmos"
+++

Search the web for a given query, fetch and extract content from top results, deduplicate, and produce a concise structured summary with source citations.

## SoP

1. Receive the search query and identify its intent (informational, navigational, or transactional).
1. Extract key concepts and named entities from the query to form effective search terms.
1. Determine search scope including time range, language, and approximate result count.
1. Validate the query for safety and reject any input containing malicious patterns.
1. Execute the web search using `web_search()` with the configured query and filters.
1. Select the top-ranking results and fetch each page using `web_fetch()`.
1. Extract the body text from fetched pages, stripping boilerplate, ads, and navigation.
1. Remove duplicate or near-duplicate content across all fetched sources.
1. Score each unique source for relevance, authority, and freshness against the original query.
1. Use `llm_chat()` to synthesize a structured summary from the scored, deduplicated content.
1. Validate that the summary is non-empty, covers multiple domains, and cites its sources.
1. If coverage or quality is insufficient, broaden the query and repeat from step 5.
1. Deliver the final report using `report()` or `report_human()` in the requested format.

> Return type and IEPL enforcement: @system/return-type-convention
