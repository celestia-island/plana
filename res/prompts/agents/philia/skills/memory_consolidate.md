+++
name = "Memory Consolidate"
agent = "philia"

[description]
en = "Consolidate scattered memory nodes into a cohesive episode. Links multiple memory nodes under a single episode node, enabling structured recall of related memories."

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
