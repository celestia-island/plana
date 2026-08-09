+++
id = "secret-placeholder"
title = "秘密占位符约定"
kind = "plan"
+++

# Secret Placeholder Convention

> **DRAFT / PLANNED** — This system is NOT yet implemented. The design below describes planned functionality. No secret vault, placeholder resolution, RBAC enforcement, or audit trail exists in the current runtime. All references to `secret_create`, `$secret.*`, `can_handle_secrets`, Orexis auto-review, etc. describe planned features.

This document defines how sensitive information (API keys, tokens, credentials, private keys) is protected within skill prompts and MCP tool calls.

> Full design document: [secret-placeholder-PLAN.md](./secret-placeholder-PLAN.md)

## Placeholder Format

```text
$secret.XXXXXX
```

Where `XXXXXX` is a 6-character random alphanumeric identifier (`[a-zA-Z0-9]{6}`). The format uses `$` prefix + `.` separator, consistent with the existing `vars.var_name` variable access syntax. The identifier reveals nothing about the secret's content, type, or purpose.

## Creation

When a skill or agent needs to reference sensitive information:

```typescript
const placeholder = await secret_create({
  summary: "GitHub deploy key for entelecheia/entelecheia — push access",
  scope: { agents: ["hubris", "kalos"], skills: ["plan_execute"] },
  ttl_seconds: 3600,  // optional, defaults to container lifetime
  metadata: { provider: "github", key_type: "deploy_key" }
});
// Returns: { id: "a3Fk9m", placeholder: "$secret.a3Fk9m" }
```

**Constraints**:

- `summary` is mandatory, 20–200 characters. It MUST describe what the secret IS and what it's FOR — sufficient for Orexis to judge access reasonability.
- `scope` is mandatory — minimum one agent, one skill.
- `ttl_seconds` defaults to container session lifetime (max 24h).
- The real value is NEVER returned to the caller — only the placeholder string.
- The real value is stored encrypted in the container's in-memory secret vault (AES-256-GCM, key derived from session token).

## Resolution

Placeholders are resolved to their real values ONLY at MCP tool call time, inside the McpRouter, and ONLY when:

1. The MCP tool has `can_handle_secrets: true` in its configuration
1. The calling agent is within the secret's declared `scope`
1. The RBAC level permits access

If `can_handle_secrets` is `false` (default), the tool receives the literal placeholder string `$secret.a3Fk9m`.

## MCP Tool Configuration

Each MCP tool definition in TOML:

```toml
[tool.github_push]
can_handle_secrets = true
rbac_secret_level = "agent_review"
allowed_secret_metadata = { provider = ["github"] }
```

## RBAC Access Levels

| Level | Real value? | Orexis invoked? | Human asked? |
| --- | --- | --- | --- |
| `full_access` | Yes, unconditionally | No | No |
| `agent_review` | Only if Orexis `audit_alignment` approves | Yes, synchronous | No |
| `human_review` | Only if Orexis + human approve | Yes, synchronous | Yes, `ask` |
| `full_deny` | Never | No | No |

## Orexis Agent Auto-Review

When `rbac_secret_level = "agent_review"`, Orexis evaluates:

1. **Purpose match**: Does the tool's purpose align with the secret's summary?
1. **Context justification**: Does the current execution phase justify accessing this secret?
1. **Least privilege**: Is there a less-sensitive alternative?
1. **Scope compliance**: Is the caller within the secret's declared scope?
1. **Pattern anomalies**: Has this agent/skill requested this secret before? In similar contexts?

Orexis returns `{ decision: "allow" | "deny", reasoning, risk_level }` within 5 seconds. On timeout: treat as `deny`.

## Storage and Lifetime

- **In-memory**: Per-container encrypted vault (never on disk, never in snapshots)
- **Container fork**: Placeholders are NOT inherited. Parent must explicitly delegate.
- **TTL**: Expired secrets are removed from vault and become unresolvable.
- **Snapshot**: Secrets are redacted before snapshot (`[SECRET_REDACTED]`).

## Audit Trail

Every lifecycle event is logged: `secret.create`, `secret.resolve`, `secret.deny`, `secret.revoke`, `secret.expire`. The real value is NEVER logged. Placeholders in log entries are masked as `[SECRET:a3Fk9m]`.

## Error Handling

| Scenario | Behavior |
| --- | --- |
| Unknown ID | Error: `Unknown secret: a3Fk9m`. Skill must recreate. |
| Orexis timeout | Treat as deny. Log warning. Skill may retry. |
| TTL expired | Resolution fails. Skill must recreate. |
| Scope mismatch | Error: `Agent kalos not in scope for secret a3Fk9m`. |
| No `can_handle_secrets` | Placeholder passed as literal string. |
| Human denies | Error: `Human review denied for secret a3Fk9m`. |
