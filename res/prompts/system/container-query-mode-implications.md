+++
id = "container-query-mode-implications"
title = "查询模式含义"
kind = "container_hint"
+++

In QUERY mode, you CAN:

- Call any tool listed in your "Available JS APIs" section (dynamically scoped per skill)
- Use your training knowledge to answer questions
- Ask the human for clarification

In QUERY mode, you CANNOT:

- Perform write operations (no `file_write`, `script_exec`, `container_fork`)
- Call tools NOT listed in your "Available JS APIs" — they will fail with a dispatch error
- Call tools that strictly require a Cosmos container (neikos.container_fork, skemma.script_exec, etc.)

IMPORTANT: Some tools (like kalos.file_read) MAY be available in QUERY mode if listed in your Available JS APIs. Trust the "Available JS APIs" section above all else.
