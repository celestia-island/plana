+++
name = "context_prepare"
agent = "philia"

[description]
en = "Prepare context for LLM by retrieving relevant memories from the cognitive memory system."
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `query` | string | yes | Context query |
| `max_nodes` | number | no | Maximum nodes to return (default: 10) |

## Example

```typescript
const result = context_prepare({
  query: 'User preferences and project context',
  max_nodes: 10
});
```
