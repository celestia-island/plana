+++
name = "Memory Query"
agent = "philia"

[description]
en = "Query the cognitive memory system using vector similarity combined with graph traversal (bundle search). Returns ranked results with scores showing both vector similarity and graph path bonuses."

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
