+++
name = "memory_query"
agent = "philia"

[description]
en = "Query the cognitive memory system using vector similarity + graph traversal (bundle search)."
zh-Hans = "使用向量相似度 + 图遍历（捆绑搜索）查询认知记忆系统。"
zh-Hant = "使用向量相似度 + 圖遍歷（捆綁搜索）查詢認知記憶系統。"
ja = "ベクトル類似度とグラフトラバーサル（バンドル検索）で認知メモリシステムを照会します。"
ko = "벡터 유사도와 그래프 순회(번들 검색)로 인지 메모리 시스템을 쿼리합니다."
fr = "Interrogez le système de mémoire cognitive par similarité vectorielle + parcours de graphe."
es = "Consulte el sistema de memoria cognitiva mediante similitud vectorial + recorrido de grafos."
ru = "Запросите когнитивную память через векторное сходство + обход графа."
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `query` | string | yes | Natural language query |
| `limit` | number | no | Max results (default: 10) |
| `graph_depth` | number | no | Graph traversal depth (default: 2) |
| `node_type_filter` | string | no | Filter by node type |

## Example

```typescript
const result = memory_query({
  query: 'What does the user know about Rust?',
  limit: 5,
  graph_depth: 2
});
```
