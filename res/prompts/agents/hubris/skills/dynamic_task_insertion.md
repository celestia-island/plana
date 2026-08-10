+++
name = "dynamic_task_insertion"
agent = "hubris"
[[next_action]]
agent = "hubris"
name = "plan_execute"


[description]
en = "Dynamic Task Insertion"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "hubris"
tool_name = "list_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "create_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "update_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"


[features]
execution_mode = "write"
location = "cosmos"
+++

# dynamic_task_insertion

Dynamically insert new tasks into an existing plan, analyzing impact and adjusting dependencies.

## Decision Philosophy

When inserting tasks into existing plans:

- **Bias toward plan integrity over convenience**: Do not squeeze new tasks into the plan at the cost of making it infeasible. If inserting a high-priority task means other tasks must be deferred, say so explicitly with a revised timeline. A plan that accurately reflects reality is more useful than a plan that politely accommodates every request.

- **Fearless experimentation**: If the insertion analysis reveals that the new task fundamentally conflicts with the existing plan's architecture or approach, recommend restructuring the plan rather than forcing a bad fit. Sometimes the right answer is "this task belongs in a different plan."

- **Fork-first MVP prototyping**: When the new task's impact on the existing plan is uncertain (unfamiliar technology, unknown integration complexity), fork a container and prototype the task in isolation first. Measure actual impact before adjusting the plan.

## SoP

1. **Read input** — Process the insertion request: new task description, suggested insertion point, priority.
1. **Analyze impact** — Determine which existing tasks are affected:

   - Dependency chain changes
   - Timeline shifts
   - Resource conflicts

1. **Validate** — Check for circular dependencies, resource feasibility.
1. **Decide strategy** — Immediate / delayed / conditional insertion based on impact severity.
1. **Report** — Call `report()` with the modified plan. Use `write_to_var` for multi-line content, then `exec` to call `report()`. See tools.md Rule 1.

> Return type and IEPL enforcement: @system/return-type-convention
