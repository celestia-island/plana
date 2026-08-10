+++
name = "inter_agent_messaging"
agent = "philia"

[description]
en = "Unified inter-agent messaging: send inquiries to other agents for decision guidance and information sync, and route incoming cross-container messages. Combines former agent_inquiry (SkoPeo) and route_incoming_message (Philia)."

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_list"

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_list"

[[related_tools]]
agent_name = "neikos"
tool_name = "container_fork"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "list_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "update_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "create_todo"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_skills]]
agent_name = "skopeo"
tool_name = "task_coordinate"

[[related_skills]]
agent_name = "philia"
tool_name = "conflict_parallel_resolve"

[features]
execution_mode = "write"
location = "cosmos"
+++

# inter_agent_messaging

Unified inter-agent communication: send structured inquiries to peers/parents for guidance, and route incoming cross-container messages to the correct handler.

## Decision Philosophy

- **Protocol over ad-hoc communication**: Every inter-agent message follows a typed protocol (`inquiry_type`, priority, context payload). Ad-hoc messages degrade into role-play without structured accountability. The protocol is the contract between agents.
- **Route fast, delegate deep**: Routing decisions MUST complete quickly — this is dispatch, not work execution. When a message requires substantial work, fork a container rather than blocking the router.
- **Timeout always**: Every inquiry has a timeout with a defined fallback. An agent that waits forever for a reply is a dead agent.

## SoP

### Part A: Sending Inquiries

1. **Receive inquiry request** — Parse target agent, inquiry type (`decision_guidance`, `progress_check`, `resource_request`, `conflict_resolution`, `information_sync`), question content, urgency.
1. **Validate target** — Confirm reachability. Detect and block circular inquiry chains. If unreachable, fall back to cached responses or suggest retry.
1. **Collect context** — Assemble current task progress, constraints, options, and prior related inquiries.
1. **Send inquiry** — Call `deliver_message()` with target badge, question, context payload, and priority. Record delivery ID and timestamp.
1. **Await and validate** — Poll via `consume_injected_prompts()` within timeout. On timeout: use local decision, cached answer, or escalate via `report_human()`. Validate reply completeness and relevance.
1. **Deliver** — Pass result to requesting agent/workflow. Archive the inquiry/reply pair for future pattern extraction.

### Part B: Routing Incoming Messages

1. **Receive context** — You are given: `last_skill`, `last_report`, `next_action`, `pending_messages[]` (each with `source_container`, `source_branch`, `message_type`, `content`, `suggested_skill`).
1. **Assess relevance** — For each message: related to current work? Unrelated? Context supplement? Question? Use `llm_chat()` for complex analysis.
1. **Route**:

   - **Unrelated work** → Fork new container via `container_fork()`, mark TODO as delegated.
   - **Related supplement** → Merge into context via `create_todo()`, append to next skill's input.
   - **Question** → Answer inline using current context. Mark TODO completed.
   - **Ambiguous** → Ask human via `report_human()`.

1. **Report** via `report()`.

> Return type and IEPL enforcement: @system/return-type-convention
