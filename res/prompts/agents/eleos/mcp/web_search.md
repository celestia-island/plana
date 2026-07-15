+++
name = "web_search"

[description]
en = "Search the web for information"
zhs = "在网上搜索信息"
zht = "在網路上搜尋資訊"
ja = "ウェブ上で情報を検索する"
ko = "웹에서 정보 검색"
fr = "Rechercher des informations sur le web"
es = "Buscar información en la web"
ru = "Искать информацию в интернете"
+++

# web_search

Performs a web search using a configurable search engine and returns a list of results with titles and URLs. This is the primary tool for discovering information on the internet when the exact URL is unknown.

## Parameters

- **query** (required, string): The search query string. Use concise, targeted keywords for best results.
- **engine** (optional, string): Search engine to use. Default: `"duckduckgo"`.
- **limit** (optional, number): Maximum number of results to return. Default: `10`.

## Returns

### On Success

```text
Query: <query>
Engine: <engine>
Results: <count>

1. <title>
   <url>

2. <title>
   <url>

...
```

### On Failure

```text
Search failed

Query: <query>
Error: <error message>
```

## Examples

### Example 1: Basic Search

```text
web_search
  query: "Rust programming language tutorial"
```

Returns up to 10 DuckDuckGo results with titles and URLs matching the query.

### Example 2: Limit Results

```text
web_search
  query: "Tokio async runtime documentation"
  limit: 5
```

Returns at most 5 results, useful when only a few top matches are needed.

### Example 3: Custom Engine

```text
web_search
  query: "SSE server-sent events implementation"
  engine: "duckduckgo"
  limit: 3
```

Explicitly selects the search engine and restricts output to 3 results.

## Important Notes

- The `query` string should be specific enough to yield relevant results. Avoid overly broad terms.
- Results include only the title and URL, not the page content. Use `web_fetch` to retrieve the actual content of a result.
- Some search engines may rate-limit frequent requests. If results are empty, wait before retrying.
