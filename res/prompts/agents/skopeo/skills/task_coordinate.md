+++
name = "Task Orchestration and Container Lifecycle Management"
agent = "skopeo"

[description]
en = "This skill is responsible for coordinating task allocation among multiple agents and managing the complete lifecycle of containers, including creation, startup, shutdown, and destruction."
zhs = "此技能负责协调多个代理之间的任务分配，并管理容器的完整生命周期，包括创建、启动、关闭和销毁。"
zht = "此技能負責協調多個代理之間的任務分配，並管理容器的完整生命週期，包括建立、啟動、關閉和銷毀。"
ja = "このスキルは複数のエージェント間のタスク割り当てを調整し、コンテナの作成、起動、シャットダウン、破棄を含む完全なライフサイクルを管理します。"
ko = "이 스킬은 여러 에이전트 간의 작업 할당을 조정하고, 생성, 시작, 종료, 파기를 포함한 컨테이너의 전체 수명 주기를 관리합니다."
fr = "Cette compétence est responsable de la coordination de l'allocation des tâches entre plusieurs agents et de la gestion du cycle de vie complet des conteneurs, y compris la création, le démarrage, l'arrêt et la destruction."
es = "Esta habilidad se encarga de coordinar la asignación de tareas entre múltiples agentes y gestionar el ciclo de vida completo de los contenedores, incluyendo creación, inicio, apagado y destrucción."
ru = "Этот навык отвечает за координацию распределения задач между несколькими агентами и управление полным жизненным циклом контейнеров, включая создание, запуск, остановку и уничтожение."

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_task_create"

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_close"

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_task_create"

[[related_tools]]
agent_name = "skopeo"
tool_name = "alignment_check"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_skills]]
agent_name = "skopeo"
tool_name = "node_task_summary"

[[related_skills]]
agent_name = "philia"
tool_name = "inter_agent_messaging"

[[related_skills]]
agent_name = "philia"
tool_name = "conflict_parallel_resolve"

[features]
execution_mode = "write"
location = "cosmos"
+++

Coordinate multi-agent task allocation, resolve dependencies, balance load, and manage the full container lifecycle from provisioning through cleanup.

## Decision Philosophy

When coordinating multi-agent task orchestration:

- **Bias toward aggressive parallelism**: Maximize parallel execution by identifying and removing artificial dependencies between tasks. A mildly over-parallelized execution that occasionally hits resource contention adapts better than a conservatively serialized one that wastes idle capacity.

- **Fearless experimentation**: If an agent fails a task, do not immediately fall back to a safer agent or simpler approach. Retry with adjusted parameters, reassign to a more capable agent, or fork a fresh container with more resources. Resilience comes from intelligent retry, not from avoiding ambitious assignments.

- **Fork-first MVP prototyping**: When a task requires an untested agent capability or tool combination, fork a sandbox container and test the workflow in isolation before inserting it into the production orchestration. This prevents cascading failures from experimental task assignments.

- **Multi-branch exploration as normal mode**: For complex workflows where the optimal agent assignment or execution order is uncertain, fork parallel execution branches with different strategies. Compare results and converge on the best-performing branch. This is not wasted effort — it is a search for the optimal execution path.

- **Embrace complexity when justified**: Do not simplify the orchestration DAG at the cost of correctness. If the workflow requires 10 agents with complex interleaving, model it accurately. A correct, complex orchestration that delivers results is infinitely better than a simple one that produces wrong or incomplete output.

## SoP

1. **Parse orchestration request** — Extract the task list, agent requirements, priority levels, constraints, and container resource specifications.
1. **Resolve dependencies** — Build the task dependency graph. Detect circular dependencies and break or escalate them. Group tasks into parallel execution waves using topological ordering.
1. **Assess capacity** — Query current status and workload of all available agents. Identify agent capabilities and match them to task requirements. Flag resource contention risks.
1. **Assign tasks** — Allocate each task to the best-fit agent using capability-based matching with load-balancing. Create tasks via `goal_task_create()` or `goal_task_create()`. Verify alignment via `alignment_check()`.
1. **Execute and monitor** — Drive execution in dependency order. Monitor progress through status polling. When a task completes, release its agent capacity and trigger dependent tasks.
1. **Handle failures** — On task failure: retry up to the configured limit, reassign to a backup agent, or call `goal_close()`. If agent crash is detected, restore from the last checkpoint on a fresh environment. Call `report_human()` for unrecoverable situations.
1. **Cleanup** — After all tasks reach terminal state, stop and destroy any temporary containers. Verify no orphaned resources remain.
1. **Report** — Compile the orchestration summary: completion rate, agent utilization, container lifecycle stats, exceptions, and recommendations. Deliver via `report()` and `report_human()`.

> Return type and IEPL enforcement: @system/return-type-convention
