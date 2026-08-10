+++
name = "remote_deploy"
agent = "haplotes"

[[next_action]]
agent = "skopeo"
name = "node_task_summary"

[description]
en = "Remote deployment gateway with host preparation, service lifecycle, and health verification"

[[related_tools]]
agent_name = "haplotes"
tool_name = "llm_provider_call"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
location = "cosmos"
execution_mode = "edge"
+++

# remote_deploy

Deploy services to remote hosts based on upstream context. This skill is the **gateway** for remote deployment — the upstream caller does NOT have direct access to deployment operations.

## SoP

1. **Parse request** — The upstream context describes what to deploy, where, and how. Extract target hosts, service configuration, version, and deployment strategy.
1. **Pre-flight assessment** — Use `llm_provider_call()` to analyze:

   - Target host specifications vs service requirements
   - Network connectivity assumptions
   - Dependency requirements
   - Risk level of the deployment

1. **Confirm with human** — For production deployments, always use `report_human()` to confirm the deployment plan before execution. Include rollback strategy.
1. **Prepare** — Generate deployment configuration using `llm_provider_call()`:

   - Adapt configuration to target host specs
   - Generate environment variables, ports, volumes
   - Create health check endpoints

1. **Deploy** — Execute the deployment plan step by step:

   - Upload artifacts
   - Start/upgrade services
   - Run smoke tests

1. **Verify** — Check health endpoints, run integration tests, confirm service is responsive.
1. **Report** — Call `report()` with outcome, version, host, and health status.

> Return type and IEPL enforcement: @system/return-type-convention

## Safety Rules

- Always create a rollback point before deployment
- Never deploy to all hosts simultaneously (use canary/rolling strategy)
- Set explicit health check timeouts
- Require `report_human()` confirmation for production deployments

## Edge Cases

- **Host unreachable**: Report connectivity error, suggest alternative hosts
- **Deployment fails**: Execute rollback automatically, report what failed
- **Health check fails**: Report partial deployment, suggest manual intervention
- **Version conflict**: Report the conflict, suggest upgrade path or `report_human()`
- **Insufficient resources**: Report requirements vs available, suggest scaling or cleanup
