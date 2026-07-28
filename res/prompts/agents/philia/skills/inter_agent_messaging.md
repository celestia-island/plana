+++
name = "inter_agent_messaging"
agent = "philia"

[description]
en = "Unified inter-agent messaging: send inquiries to other agents for decision guidance and information sync, and route incoming cross-container messages. Combines former agent_inquiry (SkoPeo) and route_incoming_message (Philia)."
zh-Hans = "统一的跨 Agent 消息协议：向其他 Agent 发送查询以获取决策指导和信息同步，并路由传入的跨容器消息。合并了原 agent_inquiry（SkoPeo）和 route_incoming_message（Philia）。"
zh-Hant = "統一的跨 Agent 訊息協議：向其他 Agent 發送查詢以獲取決策指導和資訊同步，並路由傳入的跨容器訊息。合併了原 agent_inquiry（SkoPeo）和 route_incoming_message（Philia）。"
ja = "統合されたエージェント間メッセージング：他のエージェントに問い合わせを送信して意思決定ガイダンスと情報同期を行い、受信したクロスコンテナメッセージをルーティングします。"
ko = "통합된 에이전트 간 메시징: 다른 에이전트에게 문의를 보내 의사결정 지침과 정보 동기화를 수행하고, 수신된 크로스 컨테이너 메시지를 라우팅합니다."
fr = "Messagerie inter-agents unifiée : envoyer des demandes à d'autres agents pour obtenir des conseils de décision et synchroniser les informations, et router les messages inter-conteneurs entrants."
es = "Mensajería entre agentes unificada: enviar consultas a otros agentes para orientación de decisiones y sincronización de información, y enrutar mensajes entrantes entre contenedores."
ru = "Единый обмен сообщениями между агентами: отправка запросов другим агентам для руководства по принятию решений и синхронизации информации, а также маршрутизация входящих межконтейнерных сообщений."

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
