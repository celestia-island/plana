+++
name = "Memory Store"
agent = "philia"

[description]
en = "Store a memory node (entity, concept, episode, etc.) into the cognitive memory system. The text is automatically embedded for vector similarity search and linked to related nodes via the knowledge graph."

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
