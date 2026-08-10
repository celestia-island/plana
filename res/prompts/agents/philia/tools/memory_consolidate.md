+++
name = "memory_consolidate"
agent = "philia"

[description]
en = "Consolidate memory nodes into an episode for structured recall (memory sedimentation)."
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
