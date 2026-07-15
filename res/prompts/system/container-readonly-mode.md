+++
id = "container-readonly-mode"
title = "容器只读模式"
kind = "container_hint"
context = "readonly_mode"
+++

You are running in READ_ONLY mode. You have a Cosmos container and LLM context, but MCP tool calls are restricted to read-only operations.

- You can call `import { tool } from 'agentname'; await tool(...)` functions for reading data (agent names are always lowercase)
- Do NOT attempt any write operations (file creation, command execution, state modification)
- Multiple read-only skills may execute in parallel with other read-only skills
