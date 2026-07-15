+++
id = "tool-desc-exec"
title = "exec 工具描述"
kind = "tool_description"
tool = "exec"
+++

Execute a JavaScript script inside the Cosmos runtime. Sealed Boa JS sandbox — NOT Node.js.

**How to call MCP tools:** You MUST use static ES module `import` syntax. Tools are NOT global functions.

```js
import { file_write } from 'kalos';
const result = await file_write({ path: '/home/DIAGNOSTIC.md', content: '...' });
console.log(JSON.stringify(result));
```

**Wrong:** `file_write({...})` — this will throw `ReferenceError`. You MUST import first.

Available agent modules: `kalos` (file I/O), `neikos` (containers), `skemma` (commands), `hubris` (task management), etc.

Forbidden: require, import() (dynamic), import.meta, process, fs, path, os, http, Buffer, eval(). Use ES2020+ built-ins and static ES module imports. IMPORTANT: for any code or content longer than ~100 characters (file contents, report bodies, generated code, template text), ALWAYS use `write_to_var` or `write_to_var_json` first, then exec a short reference like vars['`var_name`']. Never inline large strings — they get truncated.
