+++
name = "rag_db_stats"
agent = "aporia"

[description]
en = "Get statistics about the RAG vector database"
+++

# rag_db_stats

返回 RAG 向量数据库的统计信息。

## Parameters

无参数。

## Returns

```json
{
  "total_documents": 42,
  "total_media_assets": 7,
  "embedding_dimensions": 1536,
  "storage_backend": "postgresql+pgvector"
}
```

## Examples

```json
{}
```

## Notes

- `storage_backend` 反映当前使用的存储后端（`postgresql+pgvector` 或 `in-memory`）
- `embedding_dimensions` 为 `null` 表示数据库为空
