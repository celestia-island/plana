+++
name = "memory_query"
agent = "philia"

[description]
en = "Query the cognitive memory system using vector similarity + graph traversal (bundle search)."
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `query` | string | yes | Natural language query |
| `limit` | number | no | Max results (default: 10) |
| `graph_depth` | number | no | Graph traversal depth (default: 2) |
| `node_type_filter` | string | no | Filter by node type |

## Example

```typescript
const result = memory_query({
  query: 'What does the user know about Rust?',
  limit: 5,
  graph_depth: 2
});
```
