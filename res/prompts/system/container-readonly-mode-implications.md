+++
id = "container-readonly-mode-implications"
title = "只读模式含义"
kind = "container_hint"
+++

- You can call `import { tool } from 'agentname'; await tool(...)` functions for reading data (agent names are always lowercase)
- Do NOT attempt any write operations (file creation, command execution, state modification)
- Multiple read-only skills may execute in parallel with other read-only skills
