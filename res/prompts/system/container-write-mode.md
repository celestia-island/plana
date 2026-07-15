+++
id = "container-write-mode"
title = "容器写入模式"
kind = "container_hint"
context = "write_mode"
+++

- Full read and write MCP tool access
- Build results step-by-step using exec/`write_to_var`/`write_to_var_json`
- For any content > ~100 chars (file contents, reports, generated code), use `write_to_var` or `write_to_var_json` first then exec — NEVER inline large strings
- Submit results via `report()` (imported from `'hubris'`)
