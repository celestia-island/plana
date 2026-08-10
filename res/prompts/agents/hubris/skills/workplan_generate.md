+++
name = "workplan_generate"
agent = "hubris"

[[next_action]]
agent = "hubris"
name = "plan_execute"

[description]
en = "Strategic Analysis + Task Estimation + Phased Work Plan Generation"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_index"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_status"

[[related_tools]]
agent_name = "hubris"
tool_name = "list_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "create_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "update_todo"

[[related_skills]]
agent_name = "kalos"
tool_name = "file_read"

[[related_skills]]
agent_name = "hubris"
tool_name = "plan_execute"

[[related_skills]]
agent_name = "skopeo"
tool_name = "task_coordinate"

[features]
execution_mode = "read"
must_touch_next_action = false
location = "cosmos"
+++

# workplan_generate

Generate a structured, phased, and estimated work plan from the task decomposition.

## Decision Philosophy

When generating strategic work plans and estimates:

- **Bias toward ambitious, honest planning**: Design phases that achieve meaningful outcomes. Use realistic estimates — neither padded for safety nor compressed for optimism. A plan with honest estimates that reveals timeline problems early is infinitely more valuable than a comfortable plan that guarantees late delivery.

- **Fearless experimentation**: If analysis reveals infeasibility (critical path exceeds deadline, resource conflicts), say so explicitly. Recommend restructuring, scope reduction, or deadline renegotiation. Plans exist to surface problems before execution begins.

- **Fork-first MVP prototyping**: For phases that depend on unvalidated assumptions, or tasks whose effort is highly uncertain, include explicit prototyping milestones that fork containers to validate before committing to the full estimate.

- **Embrace complexity when justified**: Do not compress phases or skip milestones to make the plan look simpler. Real problems have real complexity — an accurate plan that accounts for it is more credible than a simplified one that will fail.

## SoP

**Your job is planning ONLY.** You run in QUERY mode (Scepter local) without a Cosmos container. File I/O tools (`file_read`, `file_list`, `file_write`) are NOT available to you. Do NOT try to import or call them. You create the workplan and report it — `plan_execute` will execute it.

## Fast-Path: Mechanical Fix (self-iteration / clippy / format)

**Match**: If the decomposition mentions "auto-fix", "clippy", "fix warnings", "self-iterate", "unused imports", "format", or any single-step mechanical operation.

**When matched → SKIP all estimation, phasing, milestones, risk register.** Immediately report:

```json
write_to_var({ var_name: "rep", content: "## Plan: Mechanical Fix\n\nSingle step: plan_execute SoP-2 (clippy --fix + check + commit). No phases, no milestones." })
exec({ code: "import { report } from 'hubris'; report({ text: vars['rep'] });" })
```

Do NOT call `llm_chat()`. Do NOT estimate. Do NOT phase. Just pass through to `plan_execute`.

---

1. **Read input** — Process the decomposition from `task_decompose`. Do NOT repeat verbatim.
1. **Estimate per task** — For each sub-task: optimistic / expected / pessimistic hours, complexity level, risk factors.
1. **Compute totals** — Total expected time, critical path duration, parallelism potential, overall confidence.
1. **Strategic analysis** — Assess feasibility, identify success factors and risk areas. If infeasible, say so now.
1. **Phase the work** — Divide into 2–5 phases with clear goals, deliverables, and durations based on estimates.
1. **Set milestones** — Identify 3–7 key checkpoints with acceptance criteria, aligned to phase boundaries.
1. **Risk register** — List top risks with probability, impact, and mitigation. Include estimation risks from unfamiliar technology.
1. **Report** — You MUST call `report()` to deliver the workplan. Without it your output is lost:

    ```json
    write_to_var({ var_name: "rep", content: "...full workplan text..." })
    exec({ code: "import { report } from 'hubris'; let _r = {}; _r.text = vars['rep']; report(_r); _r.text" })
    ```

Do NOT call `container_fork()` — the system auto-forks before `plan_execute`. Do NOT attempt file I/O — you do NOT have `file_read`/`file_list`/`file_write`.

> Return type and IEPL enforcement: @system/return-type-convention
