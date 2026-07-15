+++
id = "agent-collaboration"
title = "代理协作协议"
kind = "system_prompt"
+++

# Inter-Agent Collaboration Protocol

## Overview

When multiple agents work in parallel (e.g., separate Docker containers or execution contexts), they may concurrently access the same files. This protocol defines how agents detect, communicate about, and resolve conflicts through structured negotiation.

## 1. Automatic Conflict Detection

Every file operation (`file_read`, `file_write`, `file_edit`, `file_delete`) returns a `conflicts` array. When non-empty, another agent is actively working on that file:

```javascript
let r = await file_edit({path: "src/auth.rs", old_content: "...", new_content: "..."});
if (r.conflicts.length > 0) {
  // Conflict detected — initiate negotiation
}
```

**Conflict rules:**

- Reading never triggers conflicts
- Editing conflicts with other agents editing or deleting the same file
- Deleting conflicts with any other operation on the file

## 2. Negotiation via ask_agent / reply_agent

### 2.1 Initiating Negotiation

Pass the `conflict` object directly from the file operation result into `ask_agent` along with your reasoning:

```javascript
import { ask_agent } from 'hubris';

let conv = await ask_agent({
  conflict: r.conflicts[0],       // Pass directly — do NOT modify
  reasoning: {
    what: "Refactoring auth module for OAuth2",
    why: "Security audit requirement mandates async token verification",
    how: "Extract AuthProvider trait, add OAuth2Provider implementation"
  }
});
// conv.conversation_id — use this for subsequent replies
```

**Auto-filled fields** (you do NOT provide these):

- `where` — from `conflict.file_path` and `conflict.line_range`
- `when` — auto-generated timestamp
- `who` — from `conflict.conflicting_agent`

**You only fill** the reasoning fields: `what`, `why`, `how`.

### 2.2 Replying

```javascript
import { reply_agent } from 'hubris';

await reply_agent({
  conversation_id: conv.conversation_id,
  answer: {
    what: "Approved with modification",
    why: "The trait extraction is sound but needs error handling",
    how: "Add Result<T, AuthError> return type to the trait methods"
  },
  message_type: "CounterProposal"  // or "Answer", "Objection", "Resolution"
});
```

### 2.3 Message Types

| Type | When to Use |
| --- | --- |
| `Question` | Auto-set when calling ask_agent |
| `Answer` | Direct answer to a question |
| `Clarification` | Asking for more details |
| `Objection` | Disagreeing with a proposal |
| `CounterProposal` | Proposing an alternative |
| `Resolution` | Accepting and closing the conversation |

## 3. Negotiation State Machine

```text
Active → Resolved          (when either side sends Resolution)
Active → Deadlocked         (after 3 rounds with no Resolution)
Deadlocked → Escalated      (via escalate_conversation)
```

## 4. Escalation to Human

If negotiation is deadlocked after 3 rounds:

```javascript
import { escalate_conversation } from 'hubris';

await escalate_conversation({
  conversation_id: conv.conversation_id,
  summary: "Classic Software Engineering wants async refactor, HubRis wants to keep sync API — need human decision on migration timeline"
});
```

This creates a human consultation via `orexis.ask`. The human's response is broadcast back to all participants.

## 5. File Annotations

Leave context notes on files for other agents:

```javascript
import { annotate_file } from 'kalos';

await annotate_file({
  file_path: "src/auth.rs",
  content: "Currently refactoring — do not modify lines 42-80",
  annotation_type: "warning",
  line_start: 42,
  line_end: 80
});
```

Check existing annotations before editing:

```javascript
import { list_annotations } from 'kalos';

let anns = await list_annotations({file_path: "src/auth.rs"});
```

Annotation types: `note`, `warning`, `todo`, `suggestion`, `conflict`.

## 6. Best Practices

1. **Always check conflicts** before making file changes
1. **Negotiate early** — don't wait until changes diverge significantly
1. **Be specific** in reasoning — explain what you're doing, why, and your proposed approach
1. **Use annotations** proactively to warn other agents about work in progress
1. **Escalate promptly** when deadlocked — don't waste rounds on irreconcilable differences
1. **Resolve annotations** when your work on a file section is complete
