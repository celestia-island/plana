+++
name = "Memory Query"
agent = "philia"

[description]
en = "Query the cognitive memory system using vector similarity combined with graph traversal (bundle search). Returns ranked results with scores showing both vector similarity and graph path bonuses."
zhs = "使用向量相似度结合图遍历（捆绑搜索）查询认知记忆系统。返回带有评分的排序结果，评分包含向量相似度和图路径加成。"
zht = "使用向量相似度結合圖遍歷（捆綁搜索）查詢認知記憶系統。返回帶有評分的排序結果，評分包含向量相似度和圖路徑加成。"
ja = "ベクトル類似度とグラフトラバーサル（バンドル検索）を組み合わせて認知メモリシステムを照会します。ベクトル類似度スコアとグラフパスボーナスを含む順位付けされた結果を返します。"
ko = "벡터 유사도와 그래프 순회(번들 검색)를 결합하여 인지 메모리 시스템을 쿼리합니다. 벡터 유사도 점수와 그래프 경로 보너스가 포함된 순위별 결과를 반환합니다."
fr = "Interrogez le système de mémoire cognitive en combinant la similarité vectorielle et le parcours de graphe (recherche en bundle). Retourne des résultats classés avec des scores de similarité vectorielle et des bonus de chemin de graphe."
es = "Consulta el sistema de memoria cognitiva utilizando similitud vectorial combinada con recorrido de grafos (búsqueda en paquete). Devuelve resultados clasificados con puntuaciones de similitud vectorial y bonificaciones de ruta de grafos."
ru = "Запросите когнитивную систему памяти, используя векторное сходство в сочетании с обходом графа (пакетный поиск). Возвращает ранжированные результаты с оценками векторного сходства и бонусами за пути графа."

[[related_tools]]
agent_name = "philia"
tool_name = "memory_query"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_store"

[features]
location = "cosmos"
execution_mode = "read"
+++

## Memory Query

Query memories using the M-Flow inspired bundle search algorithm: vector anchors identify seed nodes, then graph propagation discovers related context with path-cost scoring.

### Usage via exec

```json
// Query for Rust-related memories
const result = memory_query({
  query: 'What programming languages does the user prefer?',
  limit: 5,
  graph_depth: 2,
  node_type_filter: 'entity'
});
```

### Parameters

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `query` | string | yes | Natural language query |
| `limit` | number | no | Max results (default: 10) |
| `graph_depth` | number | no | Graph traversal depth (default: 2) |
| `node_type_filter` | string | no | Filter by node type |
| `for_context_injection` | boolean | no | Optimize results for LLM context injection (shorthand for typical context-prepare defaults: limit=10, graph_depth=1). Default: false |
