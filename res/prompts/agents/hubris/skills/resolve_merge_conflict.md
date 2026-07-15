+++
name = "resolve_merge_conflict"
agent = "hubris"

[description]
en = "Resolves merge conflicts between parallel container overlays by intelligently merging or selecting the correct version for each conflicting file."
zhs = "解决并行容器覆盖层之间的合并冲突，通过智能合并或为每个冲突文件选择正确版本来解决冲突。"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[features]
execution_mode = "write"
location = "cosmos"
+++

# Resolve Merge Conflict

You are resolving merge conflicts in a three-layer overlay architecture:

```text
宿主机 (Host / upstream / source of truth)
  ← merge_layer_2
# demiurge (buffer layer)
  ← merge_layer_1
写任务子容器 (fork / downstream / working copy)
```

## Input

You will receive a JSON object describing the conflicts:

```json
{
  "conflicts": [
    {
      "path": "/home/src/lib.rs",
      "child_kind": "Modified",
      "parent_kind": "Modified"
    }
  ],
  "parent_session": "demiurge",
  "workspace_uri": "local:///mnt/sdb1/project"
}
```

## Upstream/Downstream Relationship

You have two versions of each conflicting file:

- **Upstream (source of truth)**: The authoritative version. This is the layer closer to the host.
- **Downstream (to be adjusted)**: The working version derived from upstream. It may contain valid modifications, but on conflict, upstream wins.

### Which is upstream?

| Merge phase | Upstream | Downstream |
| --- | --- | --- |
| fork → #demiurge | #demiurge | fork container |
| #demiurge → host | Host workspace | #demiurge |

## Resolution Rules

1. **Read each conflicting file** from the current workspace to get the latest version.
1. **Analyze both versions** of each conflicting file with clear priority:

   - **Downstream new additions**: If downstream added content upstream does not have (new functions, new imports, new files), **preserve it**.
   - **Downstream modifications**: If downstream modified the same region upstream also modified, **use the upstream version**.
   - **Downstream deletions**: If downstream deleted code that upstream still has, **preserve the upstream code**.
   - **Non-conflicting regions**: Auto-merge — do not discard any non-conflicting changes from either side.

1. **Write the resolved content** using `file_write` for each conflict.
1. **Report** the resolution summary.

### Summary: Preserve all downstream non-conflicting changes; adopt upstream version for conflicts.

## Important

- You MUST call `file_write` for every conflict you resolve.
- You MUST call `report()` when done.
- Prefer merging over overwriting when both versions have valid changes.
- Preserve code style, comments, and structure from both versions when merging.
- When in doubt, the upstream (parent) version is always the source of truth.
