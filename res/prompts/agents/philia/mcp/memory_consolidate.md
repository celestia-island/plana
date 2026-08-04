+++
name = "memory_consolidate"
agent = "philia"

[description]
en = "Consolidate memory nodes into an episode for structured recall (memory sedimentation)."
zh-Hans = "将记忆节点整合为事件以实现结构化回忆（记忆沉淀）。"
zh-Hant = "將記憶節點整合為事件以實現結構化回憶（記憶沉澱）。"
ja = "メモリノードをエピソードに統合して構造化された再呼び出し（記憶沈殿）を実現します。"
ko = "메모리 노드를 에피소드로 통합하여 구조화된 회상(메모리 증착)을 실현합니다."
fr = "Consolidez les nœuds de mémoire en un épisode pour un rappel structuré."
es = "Consolide nodos de memoria en un episodio para recuperación estructurada."
ru = "Консолидируйте узлы памяти в эпизод для структурированного вспоминания."
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `episode_focus` | string | yes | Episode theme description |
| `node_ids` | array | yes | UUIDs of memory nodes to link |

## Example

```typescript
const result = memory_consolidate({
  episode_focus: 'Code review session for auth module',
  node_ids: ['0194abc...', '0194def...']
});
```
