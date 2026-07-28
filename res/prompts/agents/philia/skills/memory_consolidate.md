+++
name = "Memory Consolidate"
agent = "philia"

[description]
en = "Consolidate scattered memory nodes into a cohesive episode. Links multiple memory nodes under a single episode node, enabling structured recall of related memories."
zhs = "将分散的记忆节点整合为一个连贯的事件。将多个记忆节点关联到一个事件节点下，实现相关记忆的结构化回忆。"
zht = "將分散的記憶節點整合為一個連貫的事件。將多個記憶節點關聯到一個事件節點下，實現相關記憶的結構化回憶。"
ja = "散在するメモリノードを一つのコヒーレントなエピソードに統合します。複数のメモリノードを単一のエピソードノードにリンクし、関連メモリの構造化された再呼び出しを可能にします。"
ko = "분산된 메모리 노드를 하나의 일관된 에피소드로 통합합니다. 여러 메모리 노드를 단일 에피소드 노드에 연결하여 관련 메모리의 구조화된 회상을 가능하게 합니다."
fr = "Consolidez les nœuds de mémoire dispersés en un épisode cohérent. Lie plusieurs nœuds de mémoire sous un seul nœud d'épisode, permettant un rappel structuré des souvenirs associés."
es = "Consolida nodos de memoria dispersos en un episodio cohesivo. Vincula múltiples nodos de memoria bajo un único nodo de episodio, permitiendo la recuperación estructurada de recuerdos relacionados."
ru = "Консолидируйте разбросанные узлы памяти в связный эпизод. Связывает несколько узлов памяти под одним узлом эпизода, обеспечивая структурированное вспоминание связанных воспоминаний."

[[related_tools]]
agent_name = "philia"
tool_name = "memory_consolidate"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_store"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_query"

[features]
location = "cosmos"
execution_mode = "write"
+++

## Memory Consolidate

Group related memory nodes into an episode for structured recall. This implements the memory sedimentation mechanism from the PhiLia cognitive architecture.

### Usage via exec

```typescript
const result = memory_consolidate({
  episode_focus: 'User onboarding session',
  node_ids: ['0194abc...', '0194def...', '0194ghi...']
});
```

### Parameters

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `episode_focus` | string | yes | Description of the episode theme |
| `node_ids` | string[] | yes | IDs of memory nodes to consolidate |
