+++
name = "rag_db_read"
agent = "aporia"


[description]
en = "Retrieve relevant knowledge from the RAG database using semantic similarity search"
zhs = "使用语义相似性搜索从RAG数据库中检索相关知识"
zht = "使用語義相似性搜尋從RAG資料庫中檢索相關知識"
ja = "セマンティック類似性検索を使用してRAGデータベースから関連知識を取得する"
ko = "의미 유사도 검색을 사용하여 RAG 데이터베이스에서 관련 지식 검색"
fr = "Récupérer les connaissances pertinentes de la base de données RAG par recherche de similarité sémantique"
es = "Recuperar conocimientos relevantes de la base de datos RAG mediante búsqueda de similitud semántica"
ru = "Получить релевантные знания из базы данных RAG с помощью семантического поиска по сходству"
[[related_tools]]
name = "rag_db_write"
description = "Write knowledge to the RAG database"


[[related_tools]]
name = "rag_db_delete"
description = "Delete a knowledge entry by ID"


[[related_tools]]
name = "rag_db_stats"
description = "View RAG database statistics"
+++

# rag_db_read

Retrieve knowledge fragments from ApoRia's current in-memory vector-document store using cosine similarity over the provided embedding.

## Description

The `rag_db_read` tool scans ApoRia's in-memory `vector_documents` collection, computes cosine similarity against the provided query embedding, applies a fixed similarity threshold, and returns the top matches up to the specified limit. It is useful for lightweight RAG-style retrieval, but it is not a pgvector-backed database query path today.

## Parameters

- **`query_embedding`** (de facto required, array of number): The embedding vector for the search query. An empty array triggers a failure. While `query` is registered as a required parameter in the schema, it is not used in the implementation — always provide `query_embedding` instead.
- **limit** (optional, number): Maximum number of results to return. Defaults to 10.

## Returns

### On Success

```text
Retrieval successful

Query vector: [0.012, -0.034]...
Result count: <number of matches>

Result 1:
  ID: <UUID>
  Similarity: <score between 0 and 1>
  Content: <first 100 characters of matched text>...

Result 2:
  ID: <UUID>
  Similarity: <score between 0 and 1>
  Content: <first 100 characters of matched text>...

...
```

### No Results Found

```text
Retrieval successful

Query vector: [0.012, -0.034]...
Result count: 0

No matching knowledge fragments found.
```

### On Failure

```text
Operation failed

Error: query_embedding cannot be empty
```

## Use Cases

- **RAG Retrieval**: Fetch relevant context knowledge to include in an LLM prompt.
- **Context Enrichment**: Augment user queries with semantically related background information.
- **Knowledge Q&A**: Answer questions based on the stored knowledge base.
- **Similar Content Discovery**: Find document fragments that are semantically close to a reference passage.

## Examples

### Example 1: Basic semantic search

Invocation:

```text
rag_db_read
  query_embedding: [0.012, -0.034, 0.056, ..., 0.078]
  limit: 3
```

Return:

```text
Retrieval successful

Query vector: [0.012, -0.034]...
Result count: 3

Result 1:
  ID: 123e4567-e89b-12d3-a456-426614174000
  Similarity: 0.95
  Content: Project Alpha utilizes advanced semantic search capabilities to enhance information retrieval...

Result 2:
  ID: 234f5678-e89b-12d3-a456-426614174001
  Similarity: 0.89
  Content: The RAG system combines retrieval and generation to provide accurate responses...

Result 3:
  ID: 345g6789-e89b-12d3-a456-426614174002
  Similarity: 0.87
  Content: Semantic search enables finding relevant information based on meaning rather than keywords...
```

Description: Returns the top 3 most similar documents. Result 1 has the highest similarity (0.95), indicating the strongest semantic match.

### Example 2: Search with no matches

Invocation:

```text
rag_db_read
  query_embedding: [0.999, -0.001, 0.5, ..., -0.432]
  limit: 5
```

Return:

```text
Retrieval successful

Query vector: [0.999, -0.001]...
Result count: 0

No matching knowledge fragments found.
```

Description: When the query embedding does not closely match any stored vectors, zero results are returned. Consider broadening the query or verifying the embedding model.

### Example 3: Failure due to empty embedding

Invocation:

```text
rag_db_read
  query_embedding: []
  limit: 5
```

Return:

```text
Operation failed

Error: query_embedding cannot be empty
```

Description: An empty embedding array is always rejected. Ensure a valid embedding vector is computed before calling this tool.

## Important Notes

- **Storage Model**: Current retrieval scans in-memory documents rather than a PostgreSQL + pgvector backend.
- **Vector Dimensions**: The query embedding must match the stored embedding dimensions for a document to participate in retrieval.
- **Similarity Scores**: Scores range from 0 to 1. A value closer to 1 indicates higher semantic similarity.
- **Result Ordering**: Results are always sorted by similarity in descending order (highest first).
- **Performance**: Larger `limit` values increase retrieval time. Values above 20 are not recommended for latency-sensitive use cases.
- **`query` Parameter**: Although registered as required in the tool schema, the `query` string parameter is not used by the implementation. Always rely on `query_embedding` for the actual search.
