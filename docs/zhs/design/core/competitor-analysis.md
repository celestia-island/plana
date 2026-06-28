+++
title = "多 Agent 框架竞争分析"
description = """日期：2026年5月12日（在对43个crate × 1500+源文件进行完整源代码审计后更新）"""
lang = "zhs"
category = "design"
subcategory = "core"
+++

# 多 Agent 框架竞争分析

**日期**：2026年5月12日（在对43个crate × 1500+源文件进行完整源代码审计后更新）
**背景**：针对 Entelecheia（玄枢）设计维度的结构化对比。

> 当前状态说明：本文档中对 Entelecheia 的引用混合了当前代码现实和预期架构。请将"对比 Entelecheia"章节解读为与 Entelecheia 设计目标的比较，而非声称每个能力今天都已完整交付。有关当前实现现实，请优先参考此处的附录和 2026-05-13 诊断报告。

---

## 1. CrewAI

**仓库**：[crewAIInc/crewAI](https://github.com/crewAIInc/crewAI)
**语言**：Python
**许可证**：MIT
**规模**：~23k+ stars。独立于 LangChain。

### 架构

- **Agent**：通过 YAML（角色、目标、背景故事）或 Python `Agent` 类定义。每个 agent 包装一个 LLM 并具有工具访问权限。
- **编排**：两种模式：
  - **Crews**：具有顺序或分层流程的 agent 团队。顺序模式按顺序运行任务；分层模式指定一个"管理者"agent 进行委派。
  - **Flows**：事件驱动的 DAG，带有 `@start`、`@listen`、`@router` 装饰器。Pydantic 类型化状态。支持 `and_`/`or_` 条件组合器。
- **通信**：通过 Crew/Flow 运行时传递消息。Agent 生成结构化输出（`output_pydantic`、`output_json`）。
- **流程类型**：`Process.sequential` 和 `Process.hierarchical`。

### 工具暴露

- 通过 `crewai[tools]` 包提供内置工具（SerperDev 等）。自定义工具为 Python 函数。
- MCP（模型上下文协议）支持已文档化。
- 工具在定义时按 agent 分配。
- 每次 LLM 调用无明确的工具数量限制——所有分配的工具在每轮中都暴露。

### 安全模型

- **无沙箱**。Agent 在与编排相同的 Python 进程中运行。
- 企业版"AMP Suite"提供具有可观测性和访问控制的控制平面（专有）。
- 通过任务上的 `human_input=True` 实现人类参与循环。
- 未提及代码执行隔离。

### 内存/上下文

- 短期：通过对话历史实现 agent 内存。
- 长期：通过 agent 上启用 `memory=True` 提供可选的内存存储。
- 无显式的上下文压缩或 token 管理——依赖 LLM 上下文窗口。
- 文档中提到检查点，但 OSS 中细节稀少。

### 独特特性

- **Flows + Crews 协同**：将自主 agent 团队与精确的事件驱动工作流结合。
- **YAML优先配置**：Agent 和任务以声明方式定义，适合非开发者。
- **大型社区**：通过 `learn.crewai.com` 拥有 100k+ 认证开发者。
- **性能声明**：在特定 QA 任务中比 LangGraph 快 5.76 倍（自行报告）。

### 相对于 Entelecheia 设计目标的潜在差距

- 无代码执行沙箱或隔离。
- 无工具执行的正式安全模型。
- 内存相对简单——无分层上下文管理或归档。
- 单进程 Python 运行时限制了跨机器可扩展性。
- 无原生浏览器/shell 集成用于 agent 编码任务。
- 编排仅限 Python（无多语言运行时）。

---

## 2. LangGraph

**仓库**：[langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)
**语言**：Python（也通过 `langgraphjs` 提供 JS/TS）
**许可证**：MIT

### 架构

- **基于图的状态机**：节点（agent/函数）和边（转换）形成受 Pregel/Beam 启发的有向图。
- **Agent**：节点可以是 LLM 调用、工具执行或任何 Python 函数。不强制类型化为"agent"——更像是图中的函数。
- **编排**：具有类型化状态模式的 `StateGraph`。节点读/写状态。用于分支的条件边。用于组合的子图。
- **通信**：状态对象是单一事实来源。消息追加到状态列表。

### 工具暴露

- 工具是 LangChain 工具或绑定到图节点的任意可调用对象。
- 节点中可用的所有工具在该步骤中都暴露给 LLM。
- 无内置工具节流；开发者管理每个节点传递哪些工具。

### 安全模型

- **无沙箱**。代码执行是开发者的责任。
- 通过 `interrupt()` 实现人类参与循环——暂停图执行，允许状态检查/修改。
- 持久执行：状态持久化，可在失败后恢复（检查点）。
- 无工具执行或文件系统访问的隔离原语。

### 内存/上下文

- **短期**：通过状态（消息列表）的工作内存。
- **长期**：通过 `Store` 抽象（带嵌入的键值）跨会话持久内存。
- 上下文压缩未内置——开发者管理状态大小。
- 通过 `MemorySaver` 或 `SqliteSaver` 实现检查点。

### 独特特性

- **持久执行**：在失败/超时后自动从检查点恢复——非常适合长时间运行的 agent。
- **通过中断实现人类参与循环**：审批工作流的强大模式。
- **LangSmith 集成**：深度可观测性、追踪和评估。
- **LangSmith 部署**：具有可视化原型制作的生产部署平台。
- **Deep Agents**：用于计划、使用子 agent 和利用文件系统的 agent 的新子项目。
- **LangChain 生态系统**：与 LangChain 工具、模型和组件无缝集成。

### 相对于 Entelecheia 设计目标的潜在差距

- 与 LangChain 生态系统紧密耦合（尽管"可以不使用 LangChain 使用"）。
- 无安全模型——无沙箱，无权限系统。
- 低级框架——需要大量样板代码进行 agent 交互。
- 无原生多 agent 通信协议（A2A）。
- 状态图方法对于复杂的 agent 交互可能变得难以处理。
- 无内置代码执行隔离支持。

---

## 3. MetaGPT

**仓库**：[`FoundationAgents`/MetaGPT](https://github.com/`FoundationAgents`/MetaGPT)
**语言**：Python
**许可证**：MIT
**研究**：发表于 ICLR 2024

### 架构

- **基于 SOP 的多 agent**：使用预定义角色（PM、架构师、工程师等）建模软件公司。
- **Agent（角色）**：每个 `Role` 具有配置文件、目标、约束和一组 `Action`。角色使用 ReAct 循环（思考 → 行动），具有三种模式：`REACT`、`BY_ORDER`、`PLAN_AND_ACT`。
- **编排**：`Team` 类雇佣角色，投入预算，运行轮次。`Environment` 通过发布-订阅管理角色间的消息传递。
- **通信**：通过 Environment 进行基于消息的发布/订阅。角色订阅特定的消息标签。
- **核心哲学**：`Code = SOP(Team)` —— 物化的标准操作程序。

### 工具暴露

- 行动是预定义的 Python 类（`WriteCode`、`DesignAPI`、`DebugError` 等）——约 40+ 种行动类型。
- 每个角色在构建时分配特定的行动。
- 工具包括：网络搜索引擎（Serper、SerpAPI、DuckDuckGo、Google、Bing）、网络浏览器（Playwright、Selenium）、图像生成（DALL-E）、文档存储（Chroma、FAISS、Milvus、LanceDB、Qdrant）。
- LLM 仅看到其当前分配行动的行动模式，而非完整工具集。

### 安全模型

- **无沙箱**。代码生成并在同一环境中执行。
- 提供 Dockerfile 但用于部署，而非每任务隔离。
- 预算追踪：`investment` 参数限制总 LLM API 成本，超出时抛出异常。
- 执行期间无人类参与循环。
- 无工具执行沙箱或权限模型。

### 内存/上下文

- **短期**：`RoleContext` 中的 `Memory` 类——每个角色的有序消息列表。
- **长期**：`LongTermMemory` 和 `BrainMemory` 类用于持久知识。
- **工作内存**：用于规划器操作的独立工作内存。
- **消息缓冲区**：通过订阅标签过滤的异步消息队列。
- 上下文压缩未显式处理——角色观察过滤后的消息子集。

### 独特特性

- **完整 SDLC 仿真**：使用 SOP 建模整个软件公司——用户故事、需求、设计文档、代码、测试。
- **多种文档存储**：5+ 向量数据库选项。
- **广泛的提供商支持**：12+ LLM 提供商（OpenAI、Azure、Anthropic、Gemini、Ollama、Bedrock 等）。
- **Data Interpreter**：用于数据科学任务的专用 agent。
- **研究产出**：多篇已发表论文（AFlow、SPO、SELA、FACT、Data Interpreter）。
- **MGX**：基于此构建的自然语言编程产品。

### 相对于 Entelecheia 设计目标的潜在差距

- 刚性 SOP——角色和行动是预定义的；自定义角色需要编码。
- 完全无安全沙箱。
- 单机架构——无分布式 agent 部署。
- 无基于浏览器的 agent 控制（仅 CLI/API）。
- 内存模型是相当基础的每角色队列。
- 无 MCP/A2A agent 间协议支持。
- 预算追踪仅针对成本，而非资源/安全。

---

## 4. ChatDev 2.0 (DevAll)

**仓库**：[OpenBMB/ChatDev](https://github.com/OpenBMB/ChatDev)
**语言**：Python（后端），Vue 3（前端）
**许可证**：Apache 2.0
**研究**：多篇 NeurIPS/arxiv 论文

### 架构

- **零代码多 agent 平台**：Agent 和工作流完全在 YAML 配置中定义。无需编码。
- **YAML 驱动的工作流 DAG**：节点定义 agent，边定义消息流。支持子图。Web UI 中的可视化拖放画布。
- **核心模块**：`runtime/`（agent 执行）、`workflow/`（DAG 编排）、`entity/`（配置）、`server/`（FastAPI + WebSocket）、`frontend/`（Vue 3 Web 控制台）。
- **Agent**：在 YAML 节点配置中定义，包含提示、LLM 配置、工具和内存设置。
- **编排**：多种执行器类型：顺序、DAG、并行、循环、动态边。拓扑构建器将 YAML 配置转换为可执行图。

### 工具暴露

- **函数调用系统**：`functions/function_calling/` 包含内置工具（`code_executor`、file、weather、web、video、`deep_research`、uv、user）。
- **自定义工具注册**：`functions/` 目录中的 Python 函数自动发现。
- **MCP 支持**：`mcp_example/mcp_server.py` 演示了 MCP 集成。
- 工具在 YAML 配置中按节点分配。

### 安全模型

- **代码执行**：专用的 `code_executor.py`，具有可配置的执行参数。
- Docker Compose 部署可用。
- **人类参与循环**：`demo_human.yaml` 工作流、用户输入节点、确认流程。
- **WebSocket 流式传输**：实时日志监控和产物检查。
- 无显式的 agent 沙箱/隔离模型。

### 内存/上下文

- **多种内存后端**：简单内存、`mem0` 内存（持久、可学习）、基于文件的内存。
- **YAML 中的内存配置**：`store`、`context_window_size`、每节点内存类型。
- **上下文重置节点**：显式的 `context_reset` 工作流节点。
- **内存嵌入一致性**：跨会话嵌入一致性测试。
- **工作区扫描**：`workspace_scanner.py` 用于基于文件的上下文注入。

### 独特特性

- **零代码**：无需编写 Python 代码即可构建多 agent 系统——YAML + Web UI。
- **拖放 Web 控制台**：可视化工作流设计器、实时启动监控。
- **丰富的工作流模板**：数据可视化、3D 生成（Blender）、游戏开发、深度研究、教学视频。
- **子图支持**：可复用的子工作流（`react_agent.yaml`、`reflexion_loop.yaml`）。
- **OpenClaw 集成**：可被 OpenClaw 编码 agent 调用以动态创建 agent 团队。
- **Python SDK**：`chatdev` PyPI 包用于编程式工作流执行。
- **研究谱系**：MacNet、Puppeteer、IER、体验式协同学习——均基于 ChatDev 构建。
- **多 agent 电子书**：精选的多 agent 研究合集。

### 相对于 Entelecheia 设计目标的潜在差距

- 以 YAML 为中心限制了复杂逻辑的表达能力。
- 无代码执行的安全沙箱。
- Web 控制台是主要界面——不太适合无头/嵌入式使用。
- 类似 Django：有主见的项目结构，不如 Python 原生框架灵活。
- 无跨语言 agent 支持。
- 社区以研究为导向而非企业。

---

## 5. Google ADK（Agent Development Kit）

**仓库**：[google/adk-python](https://github.com/google/adk-python)
**语言**：Python（也提供 Java、Go 版本）
**许可证**：Apache 2.0

### 架构

- **代码优先**：Agent、工具和编排在 Python 代码中定义。
- **基础抽象**：Agent（蓝图）、Tool（能力）、Runner（引擎）、Session（对话状态）、Memory（跨会话回忆）、Artifact Service（文件）。
- **Agent 类型**：`LlmAgent`（LLM 驱动）、`LoopAgent`、`SequentialAgent`、`ParallelAgent`、`RemoteA2aAgent`。
- **多 agent 组合**：父 agent 具有 `sub_agents` 列表。Runner 处理 agent 间的路由。
- **A2A 协议**：原生支持远程 agent 的 Agent-to-Agent 通信协议。
- **LangGraph 集成**：`langgraph_agent.py` 用于将 LangGraph 图嵌入为 agent。

### 工具暴露

- **丰富的工具生态系统**：50+ 内置工具——Google Search、BigQuery、Bigtable、Spanner、PubSub、Vertex AI Search、MCP 工具、OpenAPI 工具、LangChain 工具、CrewAI 工具、计算机使用、bash、代码执行、Google API。
- **工具类型**：`FunctionTool`、`AgentTool`（将 agent 包装为工具）、`MCPTool`、`OpenAPITool`、`LangChainTool`、`CrewAiTool`、`SkillToolset`。
- **工具确认（HITL）**：工具执行前带有自定义输入的显式确认流程。
- **工具箱模式**：`toolbox_toolset.py` 用于捆绑工具。

### 安全模型

- **代码执行沙箱**：多种执行器——`container_code_executor.py`、`unsafe_local_code_executor.py`、`vertex_ai_code_executor.py`、`agent_engine_sandbox_code_executor.py`、`gke_code_executor.py`。
- **认证系统**：完整的 OAuth2 流程、凭据管理、认证预处理器、`authenticated_function_tool.py`。
- **人类参与循环**：工具确认、中断支持。
- **技能**：可打包、版本控制的 agent 能力，具有独立的执行上下文。
- **Agent 身份**：`agent_identity/` 用于服务账户的集成。

### 内存/上下文

- **会话**：每会话完整对话历史，通过 `SessionService` 持久化（内存、SQLite、PostgreSQL、Vertex AI）。
- **内存**：通过 `MemoryService` 跨会话回忆——内存、Vertex AI Memory Bank、Vertex AI RAG。
- **上下文压缩**：`compaction.py` 和 `llm_event_summarizer.py` 用于自动上下文摘要。
- **产物服务**：单独处理非文本数据（文件、图像）。
- **上下文缓存**：`gemini_context_cache_manager.py` 用于 Gemini 特定的上下文缓存。
- **回退**：能够将会话回退到之前调用之前。

### 独特特性

- **多语言支持**：Python、Java、Go 版本的 ADK。
- **生产部署**：`adk deploy` 到 Cloud Run、Vertex AI Agent Engine、GKE。
- **A2A 协议**：源自 Google——原生多供应商 agent 通信。
- **ADK Web**：内置开发/调试 UI，带有事件追踪。
- **评估框架**：多层测试（单元、集成、评估）及轨迹评分。
- **插件系统**：调试日志、上下文过滤、多模态结果、保存产物、BigQuery 分析。
- **规划器系统**：内置规划器，具有计划-然后-执行模式。
- **Vibe 编码支持**：`llms.txt` 和 `llms-full.txt` 用于 LLM 了解 ADK 上下文。
- **技能系统**：可复用、可打包的 agent 能力。
- **Agent 优化**：`agent_optimizer.py` 和 GEPA 提示优化器。

### 相对于 Entelecheia 设计目标的潜在差距

- 生产功能对 Google Cloud 有强依赖。
- Gemini 优化（尽管模型无关）。
- 复杂的代码库，具有许多抽象层。
- 仅 FastAPI 用于 API 服务（无替代框架）。
- 无 CLI 原生 agent 体验（以 Web UI 为重点）。
- 内存/上下文处理是 Gemini 缓存感知的，对其他模型不太通用。

---

## 6. OpenAI Swarm（实验性，现已弃用）

**仓库**：[openai/swarm](https://github.com/openai/swarm)
**语言**：Python
**许可证**：MIT
**状态**：已被 [OpenAI Agents SDK](https://github.com/openai/openai-agents-python) 取代

### 架构

- **极简原语**：`Agent`（指令 + 函数）和交接（从函数返回另一个 Agent）。
- **核心循环**（`swarm/core.py` 中约 300 行）：

  1. 从当前 Agent 获取补全
  1. 执行工具调用，追加结果
  1. 如果函数返回 Agent，则切换 Agent
  1. 更新上下文变量
  1. 重复直到没有更多工具调用或达到 `max_turns`

- **Agent**：仅名称、模型、指令（字符串或可调用）、函数列表。无 agent 层级——通过交接进行平级委派。
- **通信**：Chat Completions API 消息。在 `client.run()` 调用之间无状态。

### 工具暴露

- 工具是普通 Python 函数。从类型提示和文档字符串自动生成模式。
- 分配给 Agent 的所有函数在每次 LLM 调用中都暴露。
- 如果函数返回 `Agent`，执行转移（交接）。
- 如果在函数签名中定义，`context_variables` 参数自动填充。

### 安全模型

- **无**。无沙箱、无隔离、无权限模型。工具在调用者进程中运行。
- 明确不是用于生产的教育/实验项目。

### 内存/上下文

- **无状态**：`client.run()` 调用之间无状态。用户必须传递 `messages` 并接收它们。
- **上下文变量**：通过函数调用传递的简单字典——可被工具函数读/写。
- 无内存、无会话持久化、无上下文压缩。

### 独特特性

- **极度简洁**：整个框架约 4 个源文件。非常适合学习 agent 编排。
- **交接模式**：优雅——Agent 只是"指令 + 工具"，agent 通过从工具函数返回另一个 Agent 进行委派。
- **流式支持**：内置流式传输，带有 `{"delim":"start"}` / `{"delim":"end"}` 标记用于 agent 边界。

### 相对于 Entelecheia 设计目标的潜在差距

- 已弃用（被 OpenAI Agents SDK 取代）。
- 完全无安全模型。
- 无状态/持久化——完全无状态。
- 仅限 OpenAI（Chat Completions API）。
- 除交接外无多 agent 通信。
- 实验/教育性质——非生产框架。

---

## 7. Cline

**仓库**：[cline/cline](https://github.com/cline/cline)
**语言**：TypeScript（VS Code 扩展）+ Go（CLI）
**许可证**：Apache 2.0

### 架构

- **VS Code 扩展** + 独立 CLI。扩展是主要界面；CLI 较新。
- **核心循环**（在 `src/core/` 中）：

  1. 解析用户任务（文本 + 图像）
  1. 分析工作区（AST、正则搜索、文件读取）
  1. 在循环中执行工具：文件创建/编辑、终端命令、浏览器操作
  1. 监控输出（linter 错误、终端输出、浏览器截图）
  1. 自动修复问题，迭代直到任务完成

- **工具集**：文件操作（创建、编辑、diff）、终端命令（带 shell 集成）、浏览器（无头、点击/输入/滚动）、MCP 工具。
- **系统提示**：详细的提示工程，包含上下文管理指令。

### 工具暴露

- 固定的内置工具集：`read_file`、`write_to_file`、`replace_in_file`、`execute_command`、`browser_action`、`use_mcp_tool` 等。
- **MCP 扩展**：可以按需创建/安装新的 MCP 服务器（"添加一个工具来实现……"）。也支持社区 MCP 服务器。
- **@-提及**：`@url`、`@problems`、`@file`、`@folder` 用于上下文注入（降低 API 成本）。
- 每次调用的工具数量有限——通常 8-12 个内置 + MCP 工具。

### 安全模型

- **所有操作都需人类参与循环**：每次文件更改和终端命令必须在 GUI 中由用户批准。
- **检查点系统**：每步之前的工作区快照。可随时 diff/恢复。
- **权限系统**：`CommandPermissionController` 具有允许/拒绝列表。
- **Cline Ignore**：`.clineignore` 文件用于排除敏感文件。
- **无沙箱**：终端命令在用户的实际环境中运行——既有权力也有风险。
- **企业版**：SSO、审计追踪、VPC/专用链接、自托管/本地部署。

### 内存/上下文

- **上下文管理**：项目的 AST 分析、正则搜索相关文件、仔细选择进入上下文窗口的内容。
- **上下文压缩**：当上下文填满时，生成摘要并压缩旧对话。
- **检查点**：用于回滚的完整工作区快照。
- **任务状态**：`TaskState.ts` 管理对话内存和工作区状态。
- **循环检测**：`loop-detection.ts` 防止无限工具调用循环。

### 独特特性

- **完整 IDE 集成**：驻留在 VS Code 内部，可看到整个工作区。
- **浏览器自动化**：Claude 计算机使用能力用于 Web 测试/调试。
- **自动修复循环**：监控 linter/编译器错误并无需用户干预自动修复。
- **MCP 工具创建**：可根据用户请求即时创建新的 MCP 服务器。
- **运行时继续**：非阻塞终端命令执行。
- **多模型**：OpenRouter、Anthropic、OpenAI、Gemini、Bedrock、Azure、Vertex、Cerebras、Groq、Ollama、LM Studio。
- **Go CLI**：用于无 VS Code 环境的独立 CLI 选项。
- **评估框架**：3 层测试（契约、冒烟、端到端/基准）。

### 相对于 Entelecheia 设计目标的潜在差距

- 完整体验绑定到 VS Code（CLI 较新且不太成熟）。
- 单 agent 架构——无多 agent 协作。
- 无定义的 agent 角色/专业化。
- 每个操作都需要人类批准（尽管可自动化）。
- 无交互的长时间自主动作。
- 非框架——是一个应用程序。不能作为库嵌入。
- 仅 TypeScript/Go，无 Python SDK。

---

## 8. Aider

**仓库**：[Aider-AI/aider](https://github.com/Aider-AI/aider)
**语言**：Python
**许可证**：Apache 2.0

### 架构

- **基于终端的结对编程**：在您的仓库中编辑代码的 CLI 工具。
- **核心循环**（`base_coder.py`）：

  1. 构建仓库映射（基于 Tree-sitter AST 的代码库摘要）
  1. 发送提示 + 仓库映射 + 文件给 LLM
  1. 解析 LLM 响应的编辑指令（统一 diff、搜索/替换块、整个文件重写）
  1. 应用编辑、lint、运行测试、自动修复失败
  1. 使用合理的消息进行 git 提交

- **多种编辑格式**：`udiff`、`editblock`、`wholefile`、`search_replace`、`diff_fenced`、`editor_editblock`、`editor_whole`、`patch`、`architect`。每种都是一个单独的编码器类，具有自己的提示模板。
- **架构师/编辑者模式**：双 agent 模式——架构师计划，编辑者实现。在基准测试中证明非常有效。

### 工具暴露

- **无传统工具调用**。Aider 使用结构化文本响应（而非函数调用）。
- LLM 输出特定格式的编辑指令（diff 块、搜索/替换），由 Aider 解析并应用。
- Shell 命令：LLM 可以请求运行 shell 命令（用户确认）。
- Web 爬取：基于 Playwright 的浏览器用于读取网页。
- 语音输入：通过麦克风实现语音转代码。
- 图像：可以向聊天添加截图/图像以获取视觉上下文。

### 安全模型

- **无沙箱**。编辑直接应用于您的文件。Git 提供安全网。
- **用户确认**：Shell 命令需要明确的用户批准。
- **Git 集成**：所有更改自动提交——易于回滚。
- **监听模式**：可以监听文件更改并在测试失败时自动重新应用。
- **Lint/测试**：更改后自动运行配置的 linter 和测试套件。
- 无权限系统、无隔离、无基于角色的访问。

### 内存/上下文

- **仓库映射**：Tree-sitter AST 分析构建整个代码库的简洁映射（函数签名、类定义、导入关系）。这将代码库结构放入上下文，而不包含所有源代码。
- **聊天历史**：上下文中的完整对话。
- **文件选择**：LLM 通过语法请求特定文件——仅这些文件的内容被添加到上下文。
- **上下文窗口管理**：当接近上下文限制时，Aider 丢弃较早的对话轮次并包含摘要。
- **无长期内存**：会话间无状态。每次 `aider` 启动都是全新的。

### 独特特性

- **仓库映射**：基于 AST 的代码库理解，在代码任务中优于基于嵌入的 RAG。
- **多种编辑格式**：适应每个模型的最佳工作方式（某些模型用 udiff 效果更好，其他用 search/replace 等）。
- **架构师/编辑者模式**：具有独立 LLM 调用的两步流程，分别用于规划和执行。
- **多语言**：通过 Tree-sitter 支持 100+ 编程语言。
- **LLM 排行榜**：维护跨模型代码编辑的公共基准测试。
- **语音编码**：终端中直接语音转代码。
- **复制粘贴模式**：适用于无法访问 API 的模型的 Web 聊天界面。
- **Git 感知**：自动提交、合理的提交消息、与现有 git 仓库协作。
- **自我编写**：Aider 自身代码的 88% 由 Aider 编写。

### 相对于 Entelecheia 设计目标的潜在差距

- **单文件焦点**：主要一次编辑一个文件（尽管仓库映射提供上下文）。
- **无多 agent 系统**：仅架构师/编辑者对。无可自定义的 agent 角色。
- **无工具生态系统**：不能使用 API、数据库、Web 服务——仅文件编辑和 shell。
- **无沙箱**：直接文件系统访问——强大但有风险。
- **无持久状态**：每个会话独立。无跨会话学习。
- **无编排原语**：不是框架——是一个独立工具。
- **文本解析脆弱性**：依赖 LLM 输出精确的编辑格式——模型偏差可能导致失败。
- **仅终端**：在大多数工作流中不能作为库嵌入（尽管存在 Python API）。

---

## 对比摘要表

| 维度 | CrewAI | LangGraph | MetaGPT | ChatDev 2.0 | Google ADK | OpenAI Swarm | Cline | Aider |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **语言** | Python | Python/TS | Python | Python/Vue | Python/Java/Go | Python | TS/Go | Python |
| **框架/库** | 框架 | 框架 | 框架 | 平台 | 框架 | 实验 | 应用 | 应用 |
| **架构** | Crews+Flows | StateGraph | 基于SOP的角色 | YAML DAG | Runner+Session | 交接循环 | 工具循环 | 编辑解析器 |
| **多agent** | 是（顺序/分层） | 是（子图） | 是（基于角色） | 是（YAML节点） | 是（sub_agents） | 是（交接） | 否（单agent） | 否（单agent+架构师） |
| **代码沙箱** | 无 | 无 | 无 | 最小 | 是（GKE、容器、Vertex AI） | 无 | 无（检查点回滚） | 无（git回滚） |
| **HITL** | 是（任务标志） | 是（中断） | 否 | 是（工作流节点） | 是（工具确认） | 否 | 是（每操作） | 是（shell确认） |
| **内存模型** | 短期+可选长期 | 短期+长期（Store） | 短期+长期+工作 | 短期+mem0+文件 | 会话+跨会话 | 无（无状态） | 任务状态+检查点 | 仅聊天历史 |
| **上下文管理** | 无显式 | 无显式 | 消息过滤 | 可配置窗口 | 自动压缩 | 无 | AST分析+压缩 | 仓库映射+自动丢弃 |
| **工具暴露** | 每agent | 每节点 | 每角色（行动） | 每YAML节点 | 50+内置 | 每agent（平级） | 8-12固定 | Shell+编辑格式 |
| **协议支持** | 无 | 无 | 无 | MCP | A2A、MCP、OpenAPI | 无 | MCP | 无 |
| **模型支持** | 多模型 | LangChain生态 | 12+提供商 | 可配置 | Gemini优化、多模型 | 仅OpenAI | 10+提供商 | 10+提供商 |
| **生产就绪** | 是（含AMP） | 是（LangSmith Deploy） | 有限 | 有限 | 是 | 否 | 是（企业版） | 是 |
| **独特优势** | Flows+Crews二元性 | 持久执行 | 完整SDLC仿真 | 零代码可视化构建器 | A2A协议、沙箱执行 | 极简优雅 | IDE集成、浏览器自动化 | 仓库映射、编辑格式 |
| **代码规模** | ~50+源文件 | 大 | ~200+源文件 | ~150+源文件 | ~200+源文件 | ~4源文件 | ~300+源文件 | ~100源文件 |

---

## Entelecheia 路线图的关键收获

1. **安全/沙箱**：几乎所有框架都缺乏沙箱执行。Google ADK 是基于容器/Kubernetes 的沙箱的显著例外。这是一个主要的差异化机会。

1. **多 agent 通信**：仅 Google ADK 有正式的 agent 间协议（A2A）。大多数框架使用临时的消息传递。标准化协议（如 A2A）是一个缺口。

1. **内存架构**：大多数框架具有基本的短期内存。少数具有复杂的分层内存（工作、短期、长期）及自动上下文管理。MetaGPT 的过滤和 ADK 的压缩是最佳示例。

1. **工具暴露管理**：所有框架在每次调用时将全部工具暴露给 LLM。没有框架根据上下文/状态/安全级别动态子集化工具。这是一个架构缺口。

1. **代码执行**：仅 ADK 具有生产级沙箱代码执行。ChatDev 有基本的代码执行器。Cline/Aider 依赖原生环境。这是整个生态系统的关键安全缺口。

1. **评估**：ADK 和 Cline 有正式的评估框架。其他依赖临时测试或研究基准测试。嵌入式评估是一个差异化因素。

1. **OpenAI Swarm 的弃用**转为 OpenAI Agents SDK 标志着从教育实验向生产级框架的市场趋势。

1. **LangGraph 的持久执行**对于长时间运行的 agent 具有独特的强大能力——大多数框架假设短时任务。

1. **ChatDev 2.0 的零代码方法**面向与大多数框架根本不同的用户画像（非开发者）。这与 Entelecheia 的开发者优先设计是正交的。

1. **Cline 和 Aider** 是应用程序，不是框架。它们展示了紧密工具集成（IDE、git、终端、浏览器）的力量，但不能组合成更大的 agent 系统。

---

## 附录：Entelecheia 当前状态提醒

本附录是上述比较的权威修正层。

### 当前活跃的内容

- 12 个 Layer1 agent 在工作区中编译。
- 1 个用于 Web Automation 的 Layer2 crate 处于活跃状态。
- 额外的专用 agent 被归档为计划或部分文档，不应被解读为已完全发布的运行时模块。

### 相对成熟的内容

- `packages/scepter`、`packages/shared` 和 `packages/tui`
- 仅执行工具暴露模型
- 容器支持的执行路径
- 加密的提供商密钥存储和 RBAC 相关的认证管道

### 仍然不完整的内容

- WebUI 与 TUI 相比
- CLI 命令覆盖
- 桌面/移动集成（已迁移至 [shittim-chest](https://github.com/celestia-island/shittim-chest)）
- RAG 和内存，目前依赖内存文档、基于哈希的嵌入和图遍历，而非完全集成的 ONNX + pgvector 技术栈
- 审计完整性和容器加固
