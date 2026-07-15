+++
id = "tool-desc-exec-code-param"
title = "exec 代码参数工具描述"
kind = "tool_description"
tool = "exec_code_param"
+++

A complete, valid JavaScript program. Runs in a sealed Boa JS sandbox (NOT Node.js/Deno/Bun).

**MCP tools require import:** All tools are ES module exports, NOT globals. Always use:

```js
import { tool_name } from 'agent_name';
```

Calling `tool_name({...})` without importing will cause `ReferenceError`.

Forbidden: require, import() (dynamic), import.meta, process.*, global.*, fs, path, os, http, Buffer, setTimeout, fetch, __dirname, eval(). Available: ES2020+ built-ins (JSON, Date, Promise, console.log, Math, Array, Object, String, Map, Set, RegExp, Error, etc.) plus static ES module imports from registered agent modules.
