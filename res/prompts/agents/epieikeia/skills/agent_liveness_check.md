+++
name = "Agent Liveness Check"
agent = "epieikeia"

[description]
en = "Verify that all 12 Layer1 agents are instantiated, their MCP tools are accessible, and critical tool endpoints respond. Produces a liveness matrix for system health monitoring."
zh-Hans = "验证所有 12 个 Layer1 代理是否已实例化，其 MCP 工具可访问，关键工具端点响应正常。生成系统健康监控的存活矩阵。"
zh-Hant = "驗證所有 12 個 Layer1 代理是否已實例化，其 MCP 工具可訪問，關鍵工具端點響應正常。生成系統健康監控的存活矩陣。"
ja = "12のLayer1エージェントがインスタンス化され、MCPツールにアクセス可能で、重要なツールエンドポイントが応答することを確認します。"
ko = "12개의 Layer1 에이전트가 인스턴스화되었는지, MCP 도구에 액세스할 수 있는지, 중요 도구 엔드포인트가 응답하는지 확인합니다."
fr = "Vérifier que les 12 agents Layer1 sont instanciés, leurs outils MCP accessibles et les endpoints critiques répondent."
es = "Verificar que los 12 agentes Layer1 están instanciados, sus herramientas MCP accesibles y los endpoints críticos responden."
ru = "Проверить, что все 12 агентов Layer1 инстанцированы, их MCP-инструменты доступны и критические эндпоинты отвечают."

[[related_tools]]
agent_name = "orexis"
tool_name = "agent_integrity"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "deliver_message"

[features]
execution_mode = "read"
location = "cosmos"
+++

## Agent Liveness Check

Verifies that the cognitive system's 12-agent architecture is fully operational. Runs periodically as part of YOLO auto-cruise to catch agent failures early.

## Expected Layer1 Roster

| Agent | Role | Critical Tool |
| --- | --- | --- |
| HapLotes | Gateway, message routing | `deliver_message` |
| SkoPeo | Coordination, LLM execution | `llm_chat` |
| HubRis | Planning, task management | `report` |
| KaLos | File/repository operations | `file_read` |
| NeiKos | Container lifecycle | `container_exec` |
| SkeMma | Script execution (sandbox) | `script_exec` |
| ApoRia | Knowledge/RAG helpers | `rag_db_read` |
| EleOs | Web search, information | `web_search` |
| EpieiKeia | Scheduling, maintenance | `deliver_message` |
| OreXis | Security consultation | `security_status` |
| PhiLia | Memory, data-store | `memory_query` |
| PoleMos | Device/SSH/edge | `ssh_exec` |

## SoP

1. **Retrieve expected roster**: Call `agent_integrity` to get the full list of expected Layer1 agents and their roles.

1. **Check instantiation**: For each expected agent, verify it has an active session in the agent manager. An agent that is not instantiated is a critical failure.

1. **Probe tool availability**: For each agent, attempt to call its primary (critical) tool with a minimal no-op request:

   - If the tool responds (even with an error like "empty input"), the tool is accessible → ALIVE
   - If the tool call fails entirely (not registered, timeout), the tool is broken → DEGRADED
   - If the agent itself is missing → DOWN

1. **Build liveness matrix**: Create a status table:

| Agent | Instantiated | Primary Tool | Status |
| --- | --- | --- | --- |
| skopeo | ✓ | `llm_chat` | ALIVE |
| neikos | ✓ | `container_exec` | DEGRADED (tool timeout) |
| polemos | ✗ | — | DOWN |

1. **Compute system health score**: `healthy_count / total_agents * 100`. Thresholds:

   - 100% = HEALTHY (all green)
   - ≥75% = DEGRADED (1-3 agents down)
   - <75% = CRITICAL (too many agents down for reliable operation)

1. **Report**: Use `report()` with the liveness matrix, health score, and any critical failures that need immediate attention.

## Decision Philosophy

- **No side effects**: This skill never starts, stops, or modifies agents. It only observes.
- **Quick probe**: Tool calls use minimal payloads to avoid wasting tokens. The goal is to verify reachability, not perform real work.
- **Conservative scoring**: A degraded tool counts as half-healthy for the score, not fully healthy.
