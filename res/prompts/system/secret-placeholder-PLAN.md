+++
id = "secret-placeholder-PLAN"
title = "秘密占位符计划"
kind = "plan"
+++

# PLAN: Secret Placeholder System

## 1. Syntax Review

### 1.1 Proposed Format

```text
$secret_var_XXXXXX$
```

Where `XXXXXX` is a 6-character random alphanumeric ([a-zA-Z0-9]{6}).

### 1.2 Conflict Analysis

Existing `$` usage in the IEPL runtime:

| Pattern | Example | Context |
| --- | --- | --- |
| ES module import (current) | `file_read({...})` | `import { file_read } from 'kalos'` — resolved to `__native_dispatch` internally |
| `vars['var_name']` | `vars['report_body']` | Bracket access — legacy vars prefix |
| `vars.var_name` | `vars.report_body` | Dot access — legacy vars prefix |

**Problem**: The proposed `$secret_var_XXXXXX$` is the only pattern with a **closing `$`**. This creates parsing ambiguity:

- `$secret_var_abc123$` could be parsed as `$secret_var_abc123` + `$` (empty expression)
- The IEPL TS parser may reject this as a syntax error

### 1.3 Recommended Fix

Change delimiter to avoid conflict with existing `$` prefix syntax:

```text
Option A: §secret_var_XXXXXX§      (uses § U+00A7 — unambiguous, never appears in code)
Option B: %secret_var_XXXXXX%      (uses % — could conflict with printf-style formatting)
Option C: $secret.XXXXXX          (consistent with vars.var_name pattern, no closing delimiter)
```

**Recommendation: Option C** — `$secret.XXXXXX`. This is consistent with the existing `$.` variable access pattern, requires no new parser rules, and the dot notation naturally reads as "the secret named XXXXXX."

---

## 2. Complete Lifecycle

### 2.1 Creation

**Who**: Any agent skill that handles credentials, tokens, or keys.

**When**: At the point where a secret would otherwise enter plain-text context.

**How**:

```typescript
const placeholder = await secret_create({
  summary: "GitHub deploy key for entelecheia/entelecheia — push access",
  scope: { agents: ["hubris", "kalos"], skills: ["plan_execute", "smart_write_file"] },
  ttl_seconds: 3600,  // optional, defaults to container lifetime
  metadata: { provider: "github", key_type: "deploy_key" }
});
// Returns: { id: "a3Fk9m", placeholder: "$secret.a3Fk9m" }
```

**Constraints**:

- `summary` is mandatory, 20–200 characters, must describe what the secret IS and what it's FOR
- `scope` is mandatory — minimum one agent, one skill
- `ttl_seconds` defaults to container session lifetime (max 24h)
- The real value is NEVER returned to the caller — only the placeholder string
- The real value is stored encrypted in the container's in-memory secret vault

### 2.2 Storage

| Layer | Storage | Encryption |
| --- | --- | --- |
| **In-memory** | Per-container secret vault (HashMap<SecretId, SecretValue>) | AES-256-GCM, key derived from container session token |
| **At rest** | NEVER written to disk, never in snapshots, never in logs | N/A |
| **In transit** | MCP tool call payload | TLS (existing MCP transport encryption) |

**Container fork**: Placeholders are NOT inherited by child containers. Each fork gets an empty vault. If a child needs a secret, the parent must explicitly delegate via `secret_delegate()`.

### 2.3 Resolution

**Where**: In the McpRouter, between skill execution and MCP tool dispatch.

**When**: An MCP tool call payload contains a `$secret.XXXXXX` string.

**Process**:

```text
1. McpRouter intercepts MCP call payload
2. Scans for $secret.XXXXXX patterns
3. For each match:
   a. Look up secret in container vault
   b. Check MCP tool's can_handle_secrets config
   c. If false → leave placeholder as-is (tool receives literal string)
   d. If true → evaluate RBAC level
4. Inject resolved values into payload (replacing placeholders)
5. Dispatch MCP call
```

### 2.4 Invalidation

| Trigger | Action |
| --- | --- |
| TTL expired | Secret removed from vault, placeholder becomes unresolvable |
| Container terminates | Entire vault destroyed |
| Explicit revoke | `secret_revoke({ id: "a3Fk9m" })` |
| Scope violation detected | Orexis auto-revokes and logs |

---

## 3. RBAC Model

### 3.1 Configuration

Per MCP tool, in the tool's TOML definition:

```toml
[mcp.github_push]
can_handle_secrets = true
rbac_secret_level = "agent_review"
allowed_secret_metadata = { provider = ["github", "gitlab"] }
```

| Field | Type | Description |
| --- | --- | --- |
| `can_handle_secrets` | bool | Whether this tool ever receives resolved secrets |
| `rbac_secret_level` | enum | `full_access` / `agent_review` / `human_review` / `full_deny` |
| `allowed_secret_metadata` | map | Optional whitelist of metadata key-values this tool may access |

### 3.2 Level Behavior Matrix

| Level | Placeholder resolved? | Orexis invoked? | Human asked? | Audit log? |
| --- | --- | --- | --- | --- |
| `full_access` | Yes, unconditionally | No | No | Yes (info) |
| `agent_review` | Only if Orexis approves | Yes, synchronous | No | Yes (decision + reasoning) |
| `human_review` | Only if Orexis + human approve | Yes, synchronous | Yes, via `ask` | Yes (full chain) |
| `full_deny` | Never | No | No | Yes (denial recorded) |

### 3.3 Per-Secret Override

A secret's scope can further restrict access beyond the tool's RBAC level:

```text
Secret scope: { agents: ["hubris"] }
Tool call from: "kalos"
→ Resolution blocked even if tool has full_access — caller not in scope
```

---

## 4. Orexis Agent Auto-Review

### 4.1 Trigger

When `rbac_secret_level = "agent_review"` and the caller passes scope check.

### 4.2 Review Input

```json
{
  "secret_id": "a3Fk9m",
  "secret_summary": "GitHub deploy key for entelecheia/entelecheia — push access",
  "secret_metadata": { "provider": "github", "key_type": "deploy_key" },
  "requesting_agent": "hubris",
  "requesting_skill": "plan_execute",
  "mcp_tool": "github_push",
  "mcp_tool_description": "Push commits to a GitHub repository",
  "context_snapshot": {
    "current_task": "Fix memory leak in worker pool",
    "current_phase": "Phase 3 — deploy fix to production",
    "container_branch": "cosmos/memory-leak-fix"
  }
}
```

### 4.3 Decision Criteria

Orexis evaluates:

1. **Purpose match**: Does the tool's purpose align with the secret's summary? (`github_push` + github deploy key → yes; `slack_notify` + github deploy key → no)

1. **Context justification**: Does the current execution context justify accessing this secret? (deploy phase + deploy key → yes; code review phase + deploy key → no)

1. **Least privilege**: Is there a less-sensitive alternative? (could the task use a read-only key instead of a push key?)

1. **Scope compliance**: Is the caller within the secret's declared scope?

1. **Pattern anomalies**: Has this agent/skill requested this secret before? In similar contexts? Frequency anomalies?

### 4.4 Decision Output

```json
{
  "decision": "allow",
  "reasoning": "plan_execute in deploy phase requires push access to deliver the fix. Secret scope (hubris/plan_execute) matches caller. No anomalies detected.",
  "risk_level": "low",
  "recommendations": []
}
```

### 4.5 Timeout and Fallback

- Orexis has 5 seconds to respond
- On timeout: treat as `deny`, log warning
- Skill may retry the MCP call (triggers new review)

---

## 5. Audit Trail

### 5.1 Events Logged

| Event | Fields |
| --- | --- |
| `secret.create` | timestamp, agent, skill, secret_id, summary, scope, ttl |
| `secret.resolve` | timestamp, agent, skill, mcp_tool, secret_id, rbac_level, decision, reviewer (orexis/human) |
| `secret.deny` | timestamp, agent, skill, mcp_tool, secret_id, deny_reason |
| `secret.revoke` | timestamp, agent, reason |
| `secret.expire` | timestamp, secret_id, reason (ttl/container_exit) |

### 5.2 What is NOT logged

- The real secret value (never)
- The secret placeholder in skill prompt text (substituted with `[SECRET:a3Fk9m]` in logs)

---

## 6. Interaction with Existing Systems

### 6.1 Orexis layer3_preflight_guard

Extended to scan for leaked secrets in Layer3 agent prompts:

- Detect patterns that look like API keys, tokens, or credentials
- Block agents that embed real secrets in their prompt text
- Verify that any `$secret.XXXXXX` references are valid (point to existing vault entries)

### 6.2 Orexis audit_alignment

Enhanced with a new check category: `secret_access`. Reviews whether secret access patterns comply with organizational policies (e.g., "deploy keys must not be accessed outside deploy phase").

### 6.3 Container Snapshot (EpieiKeia snapshot_store)

Snapshots must NOT include the secret vault. Before snapshot:

1. All placeholders in context are replaced with `[SECRET_REDACTED]`
1. The vault is excluded from the snapshot payload
1. On restore, the vault is empty — skills must re-create needed secrets

---

## 7. Error Handling

| Scenario | Behavior |
| --- | --- |
| Placeholder resolves to unknown ID | McpRouter returns error: `Unknown secret: a3Fk9m`. Skill must recreate or escalate. |
| Orexis review times out | Treat as deny. Log warning. Skill may retry. |
| Secret TTL expired during MCP call | Resolution fails. Skill must recreate. |
| Scope mismatch | Resolution blocked. Error: `Agent kalos not in scope for secret a3Fk9m`. |
| Tool lacks can_handle_secrets | Placeholder passed as literal string `$secret.a3Fk9m` to tool. |
| Human reviewer rejects | Resolution blocked. Error: `Human review denied for secret a3Fk9m`. |

---

## 8. Implementation Phases

### Phase 1: Core Vault + Resolution

- In-memory secret vault per container
- `secret_create` / `secret_revoke` MCP tools (Orexis)
- McpRouter placeholder scanning and resolution
- `can_handle_secrets` flag on MCP tools
- `full_access` and `full_deny` RBAC levels

### Phase 2: Agent Auto-Review

- Orexis `audit_alignment` secret access check
- `agent_review` RBAC level
- Decision criteria engine
- Audit trail logging

### Phase 3: Human Review + Advanced

- `human_review` RBAC level (Orexis `ask` integration)
- Secret delegation across container forks
- `layer3_preflight_guard` secret leak detection
- Snapshot vault exclusion

---

## 9. Open Questions

1. **Placeholder in skill prompt text**: If a skill writes `$secret.XXXXXX` into a `write_to_var` / `write_to_var_json` buffer that later becomes prompt context, should the prompt assembler mask it? (Recommendation: yes — replace with `[SECRET]` before injection into LLM context.)

1. **Bulk resolution**: If one MCP call payload contains 5+ secrets, should Orexis review each individually or as a batch? (Recommendation: batch review with per-secret decisions.)

1. **Secret rotation**: Does the placeholder survive rotation? (Recommendation: yes — the placeholder ID is stable; only the underlying value changes. Rotation does not invalidate the placeholder.)

1. **The `$secret.XXXXXX` vs `$secret_var_XXXXXX$` question**: The closing `$` in the original proposal conflicts with existing `$` prefix syntax. Recommend `$secret.XXXXXX` (dot notation, consistent with `vars.var_name`). Confirm with user.
