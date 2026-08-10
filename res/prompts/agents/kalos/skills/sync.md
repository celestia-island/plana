+++
name = "Federated Workspace Sync"
agent = "kalos"

[description]
en = "Federated workspace sync skill provides Kalos agent with the ability to sync workspace files across devices and platforms. This skill uses advanced sync algorithms to ensure data consistency, integrity, and efficiency, supporting multi-device collaboration and distributed development scenarios."

[[related_tools]]
agent_name = "kalos"
tool_name = "file_exists"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_list"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_get_info"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
location = "cosmos"
execution_mode = "write"
+++

Sync workspace files across devices and platforms while preserving consistency, integrity, and conflict safety.

## SoP

1. **Discover scope** — Use `file_list()` and `file_exists()` to enumerate local and remote workspace paths. Collect file lists with metadata from each side.
1. **Read configuration** — Use `file_read()` to load sync config (exclude patterns, conflict strategy, priority rules). Apply defaults when config is missing.
1. **Detect changes** — Use `file_get_info()` to compare timestamps and sizes. Use `file_read()` to compute content hashes for files that differ in metadata. Classify each file as `added`, `modified`, `deleted`, or `unchanged`.
1. **Analyze conflicts** — Identify files modified on both sides since last sync. Classify conflict type: `edit-edit`, `edit-delete`, or `rename-rename`. Assess risk level.
1. **Resolve conflicts** — For low-risk auto-mergeable conflicts, apply three-way merge using content from both sides plus the base version. For high-risk conflicts, use `report_human()` to present options (keep-local, keep-remote, manual merge).
1. **Execute sync** — For each non-conflicting change, use `file_read()` on source and `file_write()` on target. Process in dependency order (directories first, then files). Handle large files with chunked reads/writes.
1. **Verify integrity** — After writing each file, use `file_read()` to re-hash and compare against the source hash. Flag any mismatch for re-transfer.
1. **Generate report** — Use `report()` to produce a structured sync summary. Use `report_human()` to surface conflicts and failures that require attention.

> Return type and IEPL enforcement: @system/return-type-convention

## Edge Cases

- **No sync config**: Apply sensible defaults, report what defaults were assumed
- **First sync**: Treat all files as new, no baseline comparison needed
- **Network/unreachable remote**: Report what's available locally, note what couldn't be checked
- **Permission errors**: Report per-file, suggest `report_human()` for resolution
