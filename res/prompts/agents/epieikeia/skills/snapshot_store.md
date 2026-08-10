+++
name = "Snapshot Lifecycle Management"
agent = "epieikeia"

[description]
en = "snapshot_store is a core skill of the Epieikeia agent, specifically designed to manage the complete lifecycle of container snapshots, including storage, indexing, retrieval, and cleanup. This skill ensures reliable preservation and efficient management of system states, providing a solid foundation for state recovery and version tracking."

[[related_tools]]
agent_name = "epieikeia"
tool_name = "deliver_message"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "consume_injected_prompts"

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_close"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "fork_container_on_next_action"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "fork_container_on_next_action"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "list_file_observers"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[features]
execution_mode = "write"
location = "cosmos"
+++

Manage the full lifecycle of container snapshots — create, index, retrieve, restore, and clean up — ensuring system states are reliably preserved for recovery and auditing.

## SoP

1. **Plan Snapshot** — Determine the container ID, reason for the snapshot (e.g., pre-deployment, periodic, on-error), and labels to apply. Verify target container is reachable; if unreachable, log the failure and abort.
1. **Create Snapshot** — Capture the container state with incremental storage where possible. Apply compression (prefer zstd). Record metadata: container ID, labels, reason, operator, timestamp. If creation fails, retry once; on second failure, use `report_human()` to escalate.
1. **Index and Tag** — Register the snapshot in the index with dimensions: time, labels, container ID, state type. Verify the index entry is retrievable immediately after creation.
1. **Retrieve Snapshot** — When a restore or audit is requested, resolve the snapshot by ID or query (labels + time range). Return the matching snapshot metadata and storage location. If no match is found, report the gap.
1. **Restore Snapshot** — Before restoring, verify snapshot integrity. Create a backup of the current state. Apply the restore. Validate the container state post-restore matches the snapshot manifest. If integrity check fails, do not restore and escalate via `report_human()`.
1. **Enforce Retention Policy** — Periodically scan snapshots against the configured retention policy (max count, max age days, keep labels). Identify candidates for deletion. Preserve snapshots tagged `critical` or `baseline` regardless of policy. Cascade-delete dependent orphaned snapshots.
1. **Report Storage Status** — Generate a lifecycle report (see Output Format) covering snapshot count, storage usage, recent operations, and retention compliance. Use `report()` for automated delivery.

> Return type and IEPL enforcement: @system/return-type-convention
