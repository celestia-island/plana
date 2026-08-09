+++
name = "memory_store"
agent = "philia"

[description]
en = "Store a memory node into the cognitive memory system with automatic embedding for semantic search."
zh-Hans = "将记忆节点存入认知记忆系统，自动计算嵌入向量用于语义搜索。"
zh-Hant = "將記憶節點存入認知記憶系統，自動計算嵌入向量用於語義搜索。"
ja = "意味検索のための自動埋め込み付きでメモリノードを認知メモリシステムに保存します。"
ko = "의미 검색을 위한 자동 임베딩과 함께 메모리 노드를 인지 메모리 시스템에 저장합니다."
fr = "Stockez un nœud de mémoire dans le système de mémoire cognitive avec vectorisation automatique."
es = "Almacene un nodo de memoria en el sistema de memoria cognitiva con incrustación automática."
ru = "Сохраните узел памяти в когнитивную систему с автоматической векторизацией."
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `text` | string | yes | Memory text content |
| `node_type` | string | yes | Node type: entity, concept, episode, facet |
| `entity_type` | string | no | Entity subtype: person, technology, concept, etc. |
| `source_episode_id` | string | no | Source episode UUID |
| `related_node_ids` | array | no | UUIDs of related memory nodes |
| `properties` | object | no | Additional metadata key-value pairs |

## Example

```typescript
const result = memory_store({
  text: 'User prefers Rust for systems programming',
  node_type: 'entity',
  entity_type: 'technology',
  related_node_ids: [],
  properties: { confidence: 'high' }
});
```
