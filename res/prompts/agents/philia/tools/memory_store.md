+++
name = "memory_store"
agent = "philia"

[description]
en = "Store a memory node into the cognitive memory system with automatic embedding for semantic search."
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
