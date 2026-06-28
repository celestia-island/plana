+++
title = "Multi-Agent Framework Competitive Analysis"
description = """Date: May 12, 2026 (updated after full source code audit of 43 crates × 1500+ source files)"""
lang = "en"
category = "design"
subcategory = "core"
+++

# Multi-Agent Framework Competitive Analysis

**Date**: May 12, 2026 (updated after full source code audit of 43 crates × 1500+ source files)
**Context**: Structured comparison against Entelecheia（玄枢） design dimensions.

> Current-state note: references to Entelecheia in this document mix present code reality and intended architecture. Read "vs. Entelecheia" sections as comparisons against Entelecheia's design goals, not as claims that every capability is fully shipped today. For current implementation reality, prioritize the appendix here and the 2026-05-13 diagnosis report.

---

## 1. CrewAI

**Repo**: [crewAIInc/crewAI](https://github.com/crewAIInc/crewAI)
**Language**: Python
**License**: MIT
**Size**: ~23k+ stars. Independent of LangChain.

### Architecture

- **Agents**: Defined via YAML (role, goal, backstory) or Python `Agent` class. Each agent wraps an LLM with tool access.
- **Orchestration**: Two modes:
  - **Crews**: Team of agents with sequential or hierarchical processes. Sequential runs tasks in order; hierarchical assigns a "manager" agent for delegation.
  - **Flows**: Event-driven DAG with `@start`, `@listen`, `@router` decorators. Pydantic-typed state. Supports `and_`/`or_` condition combinators.
- **Communication**: Message passing through the Crew/Flow runtime. Agents produce structured output (`output_pydantic`, `output_json`).
- **Process types**: `Process.sequential` and `Process.hierarchical`.

### Tool Exposure

- Built-in tools via `crewai[tools]` package (SerperDev, etc.). Custom tools as Python functions.
- MCP (Model Context Protocol) support documented.
- Tools are assigned per-agent at definition time.
- No explicit limit on tools per LLM call — all assigned tools are exposed in every turn.

### Security Model

- **No sandboxing**. Agents run in the same Python process as the orchestration.
- Enterprise "AMP Suite" offers a control plane with observability and access controls (proprietary).
- Human-in-the-loop via `human_input=True` on tasks.
- No code execution isolation mentioned.

### Memory/Context

- Short-term: Agent memory via conversation history.
- Long-term: Optional memory stores enabled via `memory=True` on agents.
- No explicit context compaction or token management — relies on LLM context window.
- Checkpointing mentioned in docs but details are sparse in OSS.

### Unique Features

- **Flows + Crews synergy**: Combine autonomous agent teams with precise event-driven workflows.
- **YAML-first configuration**: Agents and tasks defined declaratively, suitable for non-developers.
- **Large community**: 100k+ certified developers through `learn.crewai.com`.
- **Performance claims**: 5.76x faster than LangGraph in certain QA tasks (self-reported).

### Potential Gaps Relative to Entelecheia's Design Goals

- No code execution sandboxing or isolation.
- No formal security model for tool execution.
- Memory is relatively simple — no hierarchical context management or archiving.
- Single-process Python runtime limits scalability across machines.
- No native browser/shell integration for agent coding tasks.
- Orchestration is Python-only (no multi-language runtime).

---

## 2. LangGraph

**Repo**: [langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)
**Language**: Python (also JS/TS via `langgraphjs`)
**License**: MIT

### Architecture

- **Graph-based state machine**: Nodes (agents/functions) and edges (transitions) form a Pregel/Beam-inspired directed graph.
- **Agents**: Nodes can be LLM calls, tool executions, or any Python function. Not strongly typed as "agents" — more like functions in a graph.
- **Orchestration**: `StateGraph` with typed state schema. Nodes read/write state. Conditional edges for branching. Subgraphs for composition.
- **Communication**: State object is the single source of truth. Messages appended to state lists.

### Tool Exposure

- Tools are LangChain tools or arbitrary callables bound to graph nodes.
- All tools available in a node are exposed to the LLM in that step.
- No built-in tool throttle; developers manage what tools are passed per-node.

### Security Model

- **No sandboxing**. Code execution is the developer's responsibility.
- Human-in-the-loop via `interrupt()` — pauses graph execution, allows state inspection/modification.
- Durable execution: state persisted, can resume after failures (checkpointing).
- No isolation primitives for tool execution or file system access.

### Memory/Context

- **Short-term**: Working memory via state (message lists).
- **Long-term**: Persistent memory across sessions via `Store` abstraction (key-value with embeddings).
- Context compaction not built-in — developers manage state size.
- Checkpointing via `MemorySaver` or `SqliteSaver`.

### Unique Features

- **Durable execution**: Automatically resumes from checkpoint after failures/timeouts — ideal for long-running agents.
- **Human-in-the-loop via interrupts**: Powerful pattern for approval workflows.
- **LangSmith integration**: Deep observability, tracing, and evaluation.
- **LangSmith Deployment**: Production deployment platform with visual prototyping.
- **Deep Agents**: New sub-project for agents that plan, use subagents, and leverage file systems.
- **LangChain ecosystem**: Seamless integration with LangChain tools, models, and components.

### Potential Gaps Relative to Entelecheia's Design Goals

- Tightly coupled to LangChain ecosystem (though "can be used without LangChain").
- No security model — no sandboxing, no permission system.
- Low-level framework — requires significant boilerplate for agent interactions.
- No native multi-agent communication protocol (A2A).
- State graph approach can become unwieldy for complex agent interactions.
- No built-in support for code execution isolation.

---

## 3. MetaGPT

**Repo**: [`FoundationAgents`/MetaGPT](https://github.com/`FoundationAgents`/MetaGPT)
**Language**: Python
**License**: MIT
**Research**: Published at ICLR 2024

### Architecture

- **SOP-based multi-agent**: Models a software company with predefined roles (PM, Architect, Engineer, etc.).
- **Agents (Roles)**: Each `Role` has a profile, goal, constraints, and a set of `Action`s. Roles use ReAct loops (think → act) with three modes: `REACT`, `BY_ORDER`, `PLAN_AND_ACT`.
- **Orchestration**: `Team` class hires roles, invests budget, runs rounds. `Environment` manages message passing between roles via publish-subscribe.
- **Communication**: Message-based pub/sub through Environment. Roles subscribe to specific message tags.
- **Core philosophy**: `Code = SOP(Team)` — materialized Standard Operating Procedures.

### Tool Exposure

- Actions are predefined Python classes (`WriteCode`, `DesignAPI`, `DebugError`, etc.) — ~40+ action types.
- Each role gets specific actions assigned at construction.
- Tools include: web search engines (Serper, SerpAPI, DuckDuckGo, Google, Bing), web browsers (Playwright, Selenium), image generation (DALL-E), document stores (Chroma, FAISS, Milvus, LanceDB, Qdrant).
- LLM sees only the action schemas for its currently assigned actions, not the full tool set.

### Security Model

- **No sandboxing**. Code generated and executed in the same environment.
- Dockerfile provided but for deployment, not per-task isolation.
- Budget tracking: `investment` parameter caps total LLM API cost, raises exception when exceeded.
- No human-in-the-loop during execution.
- No tool execution sandboxing or permission model.

### Memory/Context

- **Short-term**: `Memory` class in `RoleContext` — ordered list of messages per role.
- **Long-term**: `LongTermMemory` and `BrainMemory` classes for persistent knowledge.
- **Working memory**: Separate working memory for planner operations.
- **Message buffer**: Async message queue with filtering by subscribed tags.
- Context compression not explicitly handled — roles observe filtered subset of messages.

### Unique Features

- **Full SDLC emulation**: Models entire software company with SOPs — user stories, requirements, design docs, code, tests.
- **Multiple document stores**: 5+ vector DB options.
- **Extensive provider support**: 12+ LLM providers (OpenAI, Azure, Anthropic, Gemini, Ollama, Bedrock, etc.).
- **Data Interpreter**: Specialized agent for data science tasks.
- **Research output**: Multiple published papers (AFlow, SPO, SELA, FACT, Data Interpreter).
- **MGX**: Natural language programming product built on top.

### Potential Gaps Relative to Entelecheia's Design Goals

- Rigid SOP — roles and actions are predefined; custom roles require coding.
- No security sandboxing whatsoever.
- Single-machine architecture — no distributed agent deployment.
- No browser-based agent control (CLI/API only).
- Memory model is fairly basic per-role queues.
- No MCP/A2A inter-agent protocol support.
- Budget tracking is cost-only, not resource/security-based.

---

## 4. ChatDev 2.0 (DevAll)

**Repo**: [OpenBMB/ChatDev](https://github.com/OpenBMB/ChatDev)
**Language**: Python (backend), Vue 3 (frontend)
**License**: Apache 2.0
**Research**: Multiple NeurIPS/arxiv papers

### Architecture

- **Zero-code multi-agent platform**: Agents and workflows defined entirely in YAML configuration. No coding required.
- **YAML-driven workflow DAG**: Nodes define agents, edges define message flow. Supports subgraphs. Visual drag-and-drop canvas in web UI.
- **Core modules**: `runtime/` (agent execution), `workflow/` (DAG orchestration), `entity/` (configuration), `server/` (FastAPI + WebSocket), `frontend/` (Vue 3 web console).
- **Agents**: Defined in YAML node configs with prompts, LLM config, tools, and memory settings.
- **Orchestration**: Multiple executor types: sequential, DAG, parallel, cycle, dynamic-edge. Topology builder converts YAML configs into executable graphs.

### Tool Exposure

- **Function calling system**: `functions/function_calling/` contains built-in tools (`code_executor`, file, weather, web, video, `deep_research`, uv, user).
- **Custom tool registration**: Python functions in `functions/` directory are auto-discovered.
- **MCP support**: `mcp_example/mcp_server.py` demonstrates MCP integration.
- Tools are assigned per-node in YAML config.

### Security Model

- **Code execution**: Dedicated `code_executor.py` with configurable execution parameters.
- Docker Compose deployment available.
- **Human-in-the-loop**: `demo_human.yaml` workflow, user input nodes, confirmation flows.
- **WebSocket streaming**: Real-time log monitoring and artifact inspection.
- No explicit sandboxing/isolation model for agents.

### Memory/Context

- **Multiple memory backends**: Simple memory, `mem0` memory (persistent, learnable), file-based memory.
- **Memory configuration in YAML**: `store`, `context_window_size`, memory type per-node.
- **Context reset nodes**: Explicit `context_reset` workflow nodes.
- **Memory embedding consistency**: Tests for embedding consistency across sessions.
- **Workspace scanning**: `workspace_scanner.py` for file-based context injection.

### Unique Features

- **Zero-code**: Build multi-agent systems without writing Python code — YAML + web UI.
- **Drag-and-drop web console**: Visual workflow designer, real-time launch monitoring.
- **Rich workflow templates**: Data visualization, 3D generation (Blender), game dev, deep research, teach video.
- **Subgraph support**: Reusable sub-workflows (`react_agent.yaml`, `reflexion_loop.yaml`).
- **OpenClaw integration**: Can be invoked by OpenClaw coding agents to dynamically create agent teams.
- **Python SDK**: `chatdev` PyPI package for programmatic workflow execution.
- **Research lineage**: MacNet, Puppeteer, IER, experiential co-learning — all built on ChatDev.
- **Multi-agent e-book**: Curated collection of multi-agent research.

### Potential Gaps Relative to Entelecheia's Design Goals

- YAML-centric limits expressiveness for complex logic.
- No security sandboxing for code execution.
- Web console is the primary interface — less suited for headless/embedded use.
- Django-like: opinionated project structure, less flexible than Python-native frameworks.
- No cross-language agent support.
- Community is research-oriented rather than enterprise.

---

## 5. Google ADK (Agent Development Kit)

**Repo**: [google/adk-python](https://github.com/google/adk-python)
**Language**: Python (also Java, Go editions)
**License**: Apache 2.0

### Architecture

- **Code-first**: Agents, tools, and orchestration defined in Python code.
- **Foundational abstractions**: Agent (blueprint), Tool (capability), Runner (engine), Session (conversation state), Memory (cross-session recall), Artifact Service (files).
- **Agent types**: `LlmAgent` (LLM-driven), `LoopAgent`, `SequentialAgent`, `ParallelAgent`, `RemoteA2aAgent`.
- **Multi-agent composition**: Parent agent has `sub_agents` list. Runner handles routing between agents.
- **A2A Protocol**: Native support for Agent-to-Agent communication protocol for remote agents.
- **LangGraph integration**: `langgraph_agent.py` for embedding LangGraph graphs as agents.

### Tool Exposure

- **Rich tool ecosystem**: 50+ built-in tools — Google Search, BigQuery, Bigtable, Spanner, PubSub, Vertex AI Search, MCP tools, OpenAPI tools, LangChain tools, CrewAI tools, computer use, bash, code execution, Google APIs.
- **Tool types**: `FunctionTool`, `AgentTool` (wrap agent as tool), `MCPTool`, `OpenAPITool`, `LangChainTool`, `CrewAiTool`, `SkillToolset`.
- **Tool confirmation (HITL)**: Explicit confirmation flow before tool execution with custom input.
- **Toolbox pattern**: `toolbox_toolset.py` for bundling tools.

### Security Model

- **Code execution sandboxing**: Multiple executors — `container_code_executor.py`, `unsafe_local_code_executor.py`, `vertex_ai_code_executor.py`, `agent_engine_sandbox_code_executor.py`, `gke_code_executor.py`.
- **Auth system**: Full OAuth2 flow, credential management, auth preprocessors, `authenticated_function_tool.py`.
- **Human-in-the-loop**: Tool confirmation, interrupt support.
- **Skills**: Packageable, version-controlled agent capabilities with separate execution context.
- **Agent identity**: `agent_identity/` integration for service accounts.

### Memory/Context

- **Session**: Full conversation history per session, persisted through `SessionService` (in-memory, SQLite, PostgreSQL, Vertex AI).
- **Memory**: Cross-session recall via `MemoryService` — in-memory, Vertex AI Memory Bank, Vertex AI RAG.
- **Context compaction**: `compaction.py` and `llm_event_summarizer.py` for automatic context summarization.
- **Artifact service**: Separate handling of non-textual data (files, images).
- **Context caching**: `gemini_context_cache_manager.py` for Gemini-specific context caching.
- **Rewind**: Ability to rewind session to before a previous invocation.

### Unique Features

- **Multi-language support**: Python, Java, Go editions of ADK.
- **Production deployment**: `adk deploy` to Cloud Run, Vertex AI Agent Engine, GKE.
- **A2A protocol**: Born from Google — native multi-vendor agent communication.
- **ADK Web**: Built-in development/debugging UI with event tracing.
- **Evaluation framework**: Multi-layer testing (unit, integration, evals) with trajectory scoring.
- **Plugin system**: Debug logging, context filtering, multimodal results, save artifacts, BigQuery analytics.
- **Planner system**: Built-in planner with plan-then-execute pattern.
- **Vibe coding support**: `llms.txt` and `llms-full.txt` for LLM context about ADK.
- **Skills system**: Reusable, packageable agent capabilities.
- **Agent optimization**: `agent_optimizer.py` and GEPA prompt optimizer.

### Potential Gaps Relative to Entelecheia's Design Goals

- Strong Google Cloud dependency for production features.
- Gemini-optimized (though model-agnostic).
- Complex codebase with many abstraction layers.
- FastAPI-only for API serving (no alternative frameworks).
- No CLI-native agent experience (web UI focused).
- Memory/context handling is Gemini-cache-aware, less generic for other models.

---

## 6. OpenAI Swarm (experimental, now deprecated)

**Repo**: [openai/swarm](https://github.com/openai/swarm)
**Language**: Python
**License**: MIT
**Status**: Superceded by [OpenAI Agents SDK](https://github.com/openai/openai-agents-python)

### Architecture

- **Minimalist primitives**: `Agent` (instructions + functions) and handoffs (return another Agent from a function).
- **Core loop** (~300 lines in `swarm/core.py`):

  1. Get completion from current Agent
  1. Execute tool calls, append results
  1. Switch Agent if function returns an Agent
  1. Update context variables
  1. Repeat until no more tool calls or `max_turns` reached

- **Agents**: Just a name, model, instructions (string or callable), list of functions. No agent hierarchy — flat delegation via handoffs.
- **Communication**: Chat Completions API messages. Stateless between `client.run()` calls.

### Tool Exposure

- Tools are plain Python functions. Auto-schema from type hints and docstrings.
- All functions assigned to an Agent are exposed to each LLM call.
- If a function returns an `Agent`, execution transfers (handoff).
- `context_variables` parameter auto-populated if defined in function signature.

### Security Model

- **None**. No sandboxing, no isolation, no permission model. Tools run in the caller's process.
- Educational/experimental project explicitly not for production.

### Memory/Context

- **Stateless**: No state between `client.run()` calls. User must pass `messages` and get them back.
- **Context variables**: Simple dict passed through function calls — can be read/written by tool functions.
- No memory, no session persistence, no context compaction.

### Unique Features

- **Extreme simplicity**: Entire framework is ~4 source files. Great for learning agent orchestration.
- **Handoff pattern**: Elegant — an Agent is just "instructions + tools" and agents delegate by returning another Agent from a tool function.
- **Streaming support**: Built-in streaming with `{"delim":"start"}` / `{"delim":"end"}` markers for agent boundaries.

### Potential Gaps Relative to Entelecheia's Design Goals

- Deprecated (succeeded by OpenAI Agents SDK).
- No security model whatsoever.
- No state/persistence — entirely stateless.
- OpenAI-only (Chat Completions API).
- No multi-agent communication beyond handoffs.
- Experimental/educational — not a production framework.

---

## 7. Cline

**Repo**: [cline/cline](https://github.com/cline/cline)
**Language**: TypeScript (VS Code extension) + Go (CLI)
**License**: Apache 2.0

### Architecture

- **VS Code extension** + standalone CLI. The extension is the primary interface; CLI is newer.
- **Core loop** (in `src/core/`):

  1. Parse user task (text + images)
  1. Analyze workspace (ASTs, regex search, file reads)
  1. Execute tools in a loop: file create/edit, terminal commands, browser actions
  1. Monitor outputs (linter errors, terminal output, browser screenshots)
  1. Auto-fix issues, iterate until task complete

- **Tool set**: File operations (create, edit, diff), terminal commands (with shell integration), browser (headless, click/type/scroll), MCP tools.
- **System prompt**: Detailed prompt engineering with context management instructions.

### Tool Exposure

- Fixed set of built-in tools: `read_file`, `write_to_file`, `replace_in_file`, `execute_command`, `browser_action`, `use_mcp_tool`, etc.
- **MCP extension**: Can create/install new MCP servers on demand ("add a tool that..."). Community MCP servers also supported.
- **@-mentions**: `@url`, `@problems`, `@file`, `@folder` for context injection (reduce API costs).
- Limited number of tools per call — typically 8-12 built-in + MCP tools.

### Security Model

- **Human-in-the-loop for all actions**: Every file change and terminal command must be approved by the user in the GUI.
- **Checkpoint system**: Workspace snapshots before each step. Can diff/restore at any point.
- **Permission system**: `CommandPermissionController` with allow/deny lists.
- **Cline Ignore**: `.clineignore` file for excluding sensitive files.
- **No sandboxing**: Terminal commands run in user's actual environment — power and risk.
- **Enterprise**: SSO, audit trails, VPC/private link, self-hosted/on-prem.

### Memory/Context

- **Context management**: AST analysis of project, regex search for relevant files, careful selection of what goes into context window.
- **Context compaction**: When context fills, summaries are generated and old conversation compressed.
- **Checkpoints**: Full workspace snapshots for rollback.
- **Task state**: `TaskState.ts` manages conversation memory and workspace state.
- **Loop detection**: `loop-detection.ts` prevents infinite tool-call loops.

### Unique Features

- **Full IDE integration**: Lives inside VS Code, sees your entire workspace.
- **Browser automation**: Claude Computer Use capability for web testing/debugging.
- **Auto-fix loop**: Monitors linter/compiler errors and auto-fixes without user intervention.
- **MCP tool creation**: Can create new MCP servers on the fly based on user requests.
- **Proceed While Running**: Non-blocking terminal command execution.
- **Multi-model**: OpenRouter, Anthropic, OpenAI, Gemini, Bedrock, Azure, Vertex, Cerebras, Groq, Ollama, LM Studio.
- **Go CLI**: Standalone CLI option for environments without VS Code.
- **Evaluation framework**: 3-layer testing (contract, smoke, E2E/bench).

### Potential Gaps Relative to Entelecheia's Design Goals

- VS Code-bound for full experience (CLI is newer and less mature).
- Single-agent architecture — no multi-agent collaboration.
- No defined agent roles/specialization.
- Human approval required for every action (though automatable).
- No long-running autonomous operation without interaction.
- Not a framework — an application. Can't embed as a library.
- TypeScript/Go only, no Python SDK.

---

## 8. Aider

**Repo**: [Aider-AI/aider](https://github.com/Aider-AI/aider)
**Language**: Python
**License**: Apache 2.0

### Architecture

- **Terminal-based pair programming**: CLI tool that edits code in your repo.
- **Core loop** (`base_coder.py`):

  1. Build repo map (Tree-sitter AST-based codebase summary)
  1. Send prompt + repo map + files to LLM
  1. Parse LLM response for edit instructions (unified diffs, search/replace blocks, whole file rewrites)
  1. Apply edits, lint, run tests, auto-fix failures
  1. Git commit with sensible messages

- **Multiple edit formats**: `udiff`, `editblock`, `wholefile`, `search_replace`, `diff_fenced`, `editor_editblock`, `editor_whole`, `patch`, `architect`. Each is a separate coder class with its own prompt template.
- **Architect/Editor mode**: Two-agent pattern — architect plans, editor implements. Proved highly effective in benchmarks.

### Tool Exposure

- **No traditional tool calling**. Aider uses structured text responses (not function calling).
- LLM outputs edit instructions in specific formats (diff blocks, search/replace) that Aider parses and applies.
- Shell commands: LLM can request to run shell commands (user-confirmed).
- Web scraping: Playwright-based browser for reading web pages.
- Voice input: Speech-to-code via microphone.
- Images: Can add screenshots/images to chat for visual context.

### Security Model

- **No sandboxing**. Edits applied directly to your files. Git provides safety net.
- **User confirmation**: Shell commands require explicit user approval.
- **Git integration**: All changes auto-committed — easy rollback.
- **Watch mode**: Can watch for file changes and auto-re-apply if tests fail.
- **Linting/testing**: Auto-runs configured linters and test suites after changes.
- No permission system, no isolation, no role-based access.

### Memory/Context

- **Repo map**: Tree-sitter AST analysis builds a concise map of the entire codebase (function signatures, class definitions, import relationships). This fits the codebase structure into context without including all source.
- **Chat history**: Full conversation in context.
- **File selection**: LLM requests specific files via syntax — only those files' contents are added to context.
- **Context window management**: When approaching context limit, Aider drops older conversation turns and includes summaries.
- **No long-term memory**: Stateless between sessions. Each `aider` launch starts fresh.

### Unique Features

- **Repo map**: AST-based codebase understanding that outperforms embedding-based RAG for code tasks.
- **Multiple edit formats**: Adapts to what works best for each model (some models do better with udiff, others with search/replace, etc.).
- **Architect/Editor mode**: Two-step process with separate LLM calls for planning and execution.
- **Polyglot**: 100+ programming languages via Tree-sitter.
- **LLM leaderboards**: Maintains public benchmarks for code editing across models.
- **Voice coding**: Speech-to-code directly in terminal.
- **Copy-paste mode**: Works with web chat interfaces for models without API access.
- **Git-aware**: Auto-commits, sensible commit messages, works with existing git repos.
- **Self-writing**: 88% of Aider's own code was written by Aider.

### Potential Gaps Relative to Entelecheia's Design Goals

- **Single-file focus**: Primarily edits one file at a time (though repo map provides context).
- **No multi-agent system**: Only architect/editor pair. No customizable agent roles.
- **No tool ecosystem**: Can't use APIs, databases, web services — only file editing and shell.
- **No sandboxing**: Direct file system access — powerful but risky.
- **No persistent state**: Each session independent. No learning across sessions.
- **No orchestration primitives**: Not a framework — a standalone tool.
- **Text parsing fragility**: Relies on LLM outputting precise edit formats — can fail on model deviations.
- **Terminal-only**: Not embeddable as a library in most workflows (though Python API exists).

---

## Comparative Summary Table

| Dimension | CrewAI | LangGraph | MetaGPT | ChatDev 2.0 | Google ADK | OpenAI Swarm | Cline | Aider |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Language** | Python | Python/TS | Python | Python/Vue | Python/Java/Go | Python | TS/Go | Python |
| **Framework/Library** | Framework | Framework | Framework | Platform | Framework | Experiment | Application | Application |
| **Architecture** | Crews+Flows | StateGraph | SOP-based Roles | YAML DAG | Runner+Session | Handoff loop | Tool loop | Edit parser |
| **Multi-agent** | Yes (sequential/hierarchical) | Yes (subgraphs) | Yes (role-based) | Yes (YAML nodes) | Yes (sub_agents) | Yes (handoffs) | No (single) | No (single+architect) |
| **Code Sandboxing** | None | None | None | Minimal | Yes (GKE, Container, Vertex AI) | None | None (checkpoint rollback) | None (git rollback) |
| **HITL** | Yes (task flag) | Yes (interrupt) | No | Yes (workflow nodes) | Yes (tool confirm) | No | Yes (every action) | Yes (shell confirm) |
| **Memory Model** | Short + optional long | Short + long (Store) | Short + long + working | Short + mem0 + file | Session + cross-session | None (stateless) | Task state + checkpoints | Chat history only |
| **Context Management** | None explicit | None explicit | Message filtering | Configurable window | Auto-compaction | None | AST analysis + compaction | Repo map + auto-drop |
| **Tool Exposure** | Per-agent | Per-node | Per-role (actions) | Per-YAML-node | 50+ built-in | Per-agent (flat) | 8-12 fixed | Shell + edit formats |
| **Protocol Support** | None | None | None | MCP | A2A, MCP, OpenAPI | None | MCP | None |
| **Model Support** | Multi-model | LangChain ecosystem | 12+ providers | Configurable | Gemini-optimized, multi | OpenAI only | 10+ providers | 10+ providers |
| **Production Ready** | Yes (with AMP) | Yes (LangSmith Deploy) | Limited | Limited | Yes | No | Yes (Enterprise) | Yes |
| **Unique Strength** | Flows+Crews duality | Durable execution | Full SDLC emulation | Zero-code visual builder | A2A protocol, sandboxed execution | Minimalist elegance | IDE integration, browser automation | Repo map, edit formats |
| **Code Size** | ~50+ source files | Large | ~200+ source files | ~150+ source files | ~200+ source files | ~4 source files | ~300+ source files | ~100 source files |

---

## Key Takeaways for Entelecheia's Roadmap

1. **Security/Sandboxing**: Nearly all frameworks lack sandboxed execution. Google ADK is the notable exception with container/Kubernetes-based sandboxing. This is a major differentiating opportunity.

1. **Multi-agent communication**: Only Google ADK has a formal inter-agent protocol (A2A). Most frameworks use ad-hoc message passing. A standardized protocol (like A2A) is a gap.

1. **Memory architectures**: Most frameworks have basic short-term memory. Few have sophisticated hierarchical memory (working, short-term, long-term) with automated context management. MetaGPT's filtering and ADK's compaction are the best examples.

1. **Tool exposure management**: All frameworks expose all tools to the LLM per-call. No framework dynamically subsets tools based on context/state/security level. This is an architectural gap.

1. **Code execution**: Only ADK has production-grade sandboxed code execution. ChatDev has basic code executor. Cline/Aider rely on native environment. This is a critical security gap across the ecosystem.

1. **Evaluation**: ADK and Cline have formal evaluation frameworks. Others rely on ad-hoc testing or research benchmarks. Embedded evaluation is a differentiator.

1. **OpenAI Swarm's deprecation** into the OpenAI Agents SDK signals a market trend toward production-grade frameworks over educational experiments.

1. **LangGraph's durable execution** is uniquely powerful for long-running agents — most frameworks assume short-lived tasks.

1. **ChatDev 2.0's zero-code approach** targets a fundamentally different user persona (non-developers) than most frameworks. This is orthogonal to Entelecheia's developer-first design.

1. **Cline and Aider** are applications, not frameworks. They demonstrate the power of tight tool integration (IDE, git, terminal, browser) but can't be composed into larger agent systems.

---

## Appendix: Entelecheia Current-State Reminder

This appendix is the authoritative correction layer for the comparisons above.

### What is active today

- 12 Layer1 agents are compiled in the workspace.
- 1 Layer2 crate for Web Automation is active.
- Additional specialized agents are archived as plans or partial docs and should not be read as fully shipped runtime modules.

### What is relatively mature

- `packages/scepter`, `packages/shared`, and `packages/tui`
- exec-only tool exposure model
- container-backed execution paths
- encrypted provider-key storage and RBAC-related auth plumbing

### What is still partial

- WebUI compared with TUI
- CLI command coverage
- desktop/mobile integrations (migrated to [shittim-chest](https://github.com/celestia-island/shittim-chest))
- RAG and memory, which currently rely on in-memory documents, hash-based embeddings, and graph traversal instead of a fully integrated ONNX + pgvector stack
- audit completeness and container hardening
