+++
name = "workspace_search"
agent = "aporia"

[description]
en = "Semantic search over indexed workspace files using natural language queries"
zhs = "使用自然语言查询对已索引的工作区文件进行语义搜索"
zht = "使用自然語言查詢對已索引的工作區檔案進行語義搜尋"
ja = "自然言語クエリを使用してインデックス済みワークスペースファイルの意味検索を行う"
ko = "자연어 쿼리를 사용하여 인덱싱된 워크스페이스 파일의 의미 검색"
fr = "Recherche sémantique dans les fichiers indexés de l'espace de travail par requêtes en langage naturel"
es = "Búsqueda semántica en archivos indexados del espacio de trabajo mediante consultas en lenguaje natural"
ru = "Семантический поиск по проиндексированным файлам рабочего пространства с использованием запросов на естественном языке"
+++

# workspace_search

## Description

Performs semantic search over the workspace files previously indexed by `workspace_index`. Converts the natural language query into an embedding, retrieves the most relevant document chunks by cosine similarity, and returns them with file path, line range, and relevance score.

## Parameters

- **query** (string, required): Natural language search query describing what you are looking for.
- **limit** (number, optional): Maximum number of results to return. Defaults to `10`.

## Returns

### On Success

```text
Search complete

Query: "<query>"
Results: <number>

Result 1:
  File: <file_path>
  Lines: <start>–<end>
  Score: <relevance_score>
  Content: <matched text excerpt>

Result 2:
  File: <file_path>
  Lines: <start>–<end>
  Score: <relevance_score>
  Content: <matched text excerpt>

...
```

### No Results

```text
Search complete

Query: "<query>"
Results: 0

No matching documents found. Ensure the workspace has been indexed with workspace_index.
```

### On Failure

```text
Search failed

Error: <error message>
```

## Examples

### Example 1: Search for authentication logic

Invocation:

```text
workspace_search
  query: "How does the authentication middleware validate JWT tokens?"
  limit: 5
```

Return:

```text
Search complete

Query: "How does the authentication middleware validate JWT tokens?"
Results: 3

Result 1:
  File: src/middleware/auth.rs
  Lines: 45–72
  Score: 0.94
  Content: fn validate_jwt(token: &str, secret: &str) -> Result<Claims, AuthError> { ... }

Result 2:
  File: docs/api/authentication.md
  Lines: 12–28
  Score: 0.88
  Content: All API endpoints require a valid JWT token in the Authorization header...

Result 3:
  File: src/config/security.rs
  Lines: 8–15
  Score: 0.81
  Content: pub const JWT_EXPIRY_SECONDS: u64 = 3600;
```

### Example 2: No results

Invocation:

```text
workspace_search
  query: "quantum computing simulation module"
  limit: 5
```

Return:

```text
Search complete

Query: "quantum computing simulation module"
Results: 0

No matching documents found. Ensure the workspace has been indexed with workspace_index.
```

## Important Notes

- **Prerequisite**: The workspace must be indexed first using `workspace_index`. Searching without indexing returns zero results.
- **Score interpretation**: Relevance scores range from 0 to 1. Scores above 0.7 generally indicate strong matches.
- **Query quality**: Specific, descriptive queries yield better results than vague or overly broad ones.
- **Result ordering**: Results are sorted by relevance score in descending order.
- **Staleness**: The index reflects the state at the time of the last `workspace_index` call. File changes after indexing are not reflected until re-indexed.
