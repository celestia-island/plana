+++
name = "Memory Store"
agent = "philia"

[description]
en = "Store a memory node (entity, concept, episode, etc.) into the cognitive memory system. The text is automatically embedded for vector similarity search and linked to related nodes via the knowledge graph."
zh-Hans = "将记忆节点（实体、概念、事件等）存入认知记忆系统。文本会自动计算嵌入向量用于相似性搜索，并通过知识图谱关联到相关节点。"
zh-Hant = "將記憶節點（實體、概念、事件等）存入認知記憶系統。文本會自動計算嵌入向量用於相似性搜索，並透過知識圖譜關聯到相關節點。"
ja = "記憶ノード（エンティティ、概念、エピソードなど）を認知メモリシステムに保存します。テキストは自動的に埋め込みベクトル化され、類似性検索とナレッジグラフによる関連ノードへのリンクに使用されます。"
ko = "메모리 노드(엔티티, 개념, 에피소드 등)를 인지 메모리 시스템에 저장합니다. 텍스트는 자동으로 임베딩되어 벡터 유사도 검색 및 지식 그래프를 통한 관련 노드 연결에 사용됩니다."
fr = "Stockez un nœud de mémoire (entité, concept, épisode, etc.) dans le système de mémoire cognitive. Le texte est automatiquement vectorisé pour la recherche de similarité et lié aux nœuds associés via le graphe de connaissances."
es = "Almacena un nodo de memoria (entidad, concepto, episodio, etc.) en el sistema de memoria cognitiva. El texto se incrusta automáticamente para búsqueda de similitud y se vincula a nodos relacionados mediante el grafo de conocimiento."
ru = "Сохраните узел памяти (сущность, концепцию, эпизод и т.д.) в когнитивную систему памяти. Текст автоматически векторизуется для поиска по сходству и связывается с родственными узлами через граф знаний."

[[related_tools]]
agent_name = "philia"
tool_name = "memory_store"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_query"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_consolidate"

[features]
location = "cosmos"
execution_mode = "write"
+++

## Memory Store

Store a new memory node into the cognitive memory engine. Each node is embedded for semantic search and optionally linked to existing nodes.

### Usage via exec

```json
// Store an entity memory
const result = memory_store({
  text: 'User prefers Rust for systems programming',
  node_type: 'entity',
  entity_type: 'technology',
  related_node_ids: [],
  properties: { confidence: 'high' }
});
```

### Parameters

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `text` | string | yes | The memory text content |
| `node_type` | string | yes | Type: entity, concept, episode, facet |
| `entity_type` | string | no | Subtype: person, technology, concept, etc. |
| `source_episode_id` | string | no | Link to source episode |
| `related_node_ids` | string[] | no | IDs of related memory nodes |
| `properties` | object | no | Additional metadata |
