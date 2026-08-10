+++
name = "discover_hooks"
agent = "epieikeia"

[description]
en = "Discover registered hooks by namespace. Returns list of hooks matching a namespace prefix, enabling epieikeia to diagnose available safety-net capabilities."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[features]
execution_mode = "read"
location = "local"
must_touch_next_action = false
+++

# discover_hooks

## Parameters

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| namespace_prefix | string | yes | Dot-separated namespace prefix to query (e.g. "pipeline.surgery") |
| include_disabled | boolean | no | Include disabled hooks (default: false) |

## Response

```json
{
  "namespace_prefix": "pipeline.surgery",
  "hooks": [
    {
      "name": "pre_surgery_checkpoint",
      "namespace": "pipeline.surgery.pre.checkpoint",
      "description": "Record git HEAD before self-modification chain",
      "hook_type": "validation",
      "phase": "pre",
      "priority": 100,
      "enabled": true
    },
    {
      "name": "post_surgery_rollback",
      "namespace": "pipeline.surgery.post.rollback",
      "description": "Validate after chain + rollback on failure",
      "hook_type": "validation",
      "phase": "post",
      "priority": 80,
      "enabled": true
    },
    {
      "name": "noa_merge_commit",
      "namespace": "pipeline.surgery.post.commit",
      "description": "Merge noa workspaces + git commit after successful chain",
      "hook_type": "custom",
      "phase": "post",
      "priority": 50,
      "enabled": true
    }
  ],
  "total": 3
}
```

## Usage

This tool enables epieikeia to:

1. **Diagnose available safety nets** — before a chain starts, discover what pre/post hooks exist
1. **Plan mitigation strategies** — if a hook can validate, epieikeia knows it doesn't need to do manual validation
1. **Coordinate with other agents** — share hook information to avoid duplicate safety checks

## Namespace Convention

```text
pipeline.surgery.*    — Self-modification safety hooks
tool.file_write.*     — File write validation hooks
agent.*               — Agent lifecycle hooks
```
