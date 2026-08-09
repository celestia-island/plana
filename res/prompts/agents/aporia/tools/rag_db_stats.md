+++
name = "rag_db_stats"
agent = "aporia"

[description]
en = "Get statistics about the RAG vector database"
zh-Hans = "获取 RAG 向量数据库统计信息"
zh-Hant = "獲取 RAG 向量數據庫統計信息"
ja = "RAGベクトルデータベースの統計情報を取得"
ko = "RAG 벡터 데이터베이스 통계 정보 조회"
fr = "Obtenir les statistiques de la base de données vectorielle RAG"
es = "Obtener estadísticas de la base de datos vectorial RAG"
ru = "Получить статистику векторной базы данных RAG"
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
