+++
id = "tools"
title = "工具调用约定"
kind = "reference"
+++

# Tool Calling Convention

## The Golden Pattern (MEMORIZE THIS)

Your exec calls should follow this pattern **90% of the time**:

```text
1️⃣  write_to_var (or write_to_var_json for JSON data) → store any text/JSON content > 1 line
2️⃣  exec          → create empty object:  let r = {};
3️⃣  exec          → add ONE property:     r.key = value;  (or r.key = vars['var_name'];)
4️⃣  repeat 3️⃣    → for each additional property (each in its own exec call)
5️⃣  exec          → submit:               report(r);
```

Keep exec calls focused and single-purpose. There is no arbitrary byte limit — the container resource limits, circuit breakers, and rate limiting protect against runaway execution. Split code across calls when it improves readability or isolates independent operations.

## Available Tools

You have **three** tools: `exec`, `write_to_var`, and `write_to_var_json`.

**All agent tools are accessed via ES module imports.** Before calling any tool function, import it from the agent's module:

```js
import { report } from 'hubris';          // report(), report_human()
import { file_read } from 'kalos';        // file_read(), file_write(), ...
import { container_fork } from 'neikos';   // container operations
import vars from 'vars';                   // persistent variable store
```

+++
[tool_desc.exec]
en = "Execute JavaScript code in the persistent Cosmos runtime"

[tool_desc.write_to_var]
en = "Write a string value into a JS variable in the Cosmos runtime"

[tool_desc.write_to_var_json]
en = "Write a validated JSON value as a parsed JS object into vars['`var_name`']"
+++

## Critical: Understand Your Execution Boundary

You are executing inside a **sealed Boa JS sandbox**. This is NOT a general-purpose runtime. Before writing any code, internalize these hard constraints:

1. **No filesystem access** — You CANNOT read, write, create, or delete files directly. Use `file_read()` (if available) or delegate to downstream skills.
1. **No command execution** — You CANNOT run shell commands, scripts, or binaries. Use `script_exec()` (if available) or delegate.
1. **No network access** — You CANNOT make HTTP requests, open sockets, or connect to external services. Use `web_search()` / `web_fetch()` (imported from `eleos`, if available) or delegate.
1. **No OS APIs** — No `child_process`, no `fs`, no `path`, no `os`, no `crypto`. See Rule 0.5 for the complete forbidden list.
1. **Limited MCP tool access** — You can ONLY call the tool functions explicitly listed in the "Available JS APIs" section injected into your prompt (accessed via ES module imports, e.g. `import { tool } from 'agent'`). All other MCP tools are **invisible and unreachable** from your context.

### How to accomplish tasks beyond your tool scope

When your current skill needs to do something concrete (write a file, run a command, manage a container, etc.) but the required tool is NOT in your available APIs, you MUST:

1. **Write a clear task description** explaining WHAT needs to be done, including all necessary context, parameters, and acceptance criteria.
1. **Submit it via `report()`** — The orchestrator will route your output to the next skill in the pipeline that HAS the required tools.
1. **Do NOT attempt workarounds** — Trying to call tools not listed in your available APIs will fail at runtime with an error.

This delegation-by-design ensures each skill operates within its security boundary while the orchestrator chains skills together to accomplish complex workflows.

## exec — run JavaScript code

```json
exec({ code: "..." })
```

Runs JS in a persistent Boa context. Variables survive across calls.
**The runtime fully supports `async`/`await`.** If your code returns a Promise (including top-level `await` expressions), the VM automatically waits for it to resolve before returning the result.

### Async / Await Support

All imported tool calls return **Promises**. You must `await` them to get the result:

```js
// Import tools before use:
// import { file_read } from 'kalos';
// import { report } from 'hubris';

// CORRECT: await each tool call
let data = await file_read({ path: '/tmp/example.txt' });
let more_data = await file_read({ path: '/tmp/another.txt' });

// You can also use .then() if preferred
file_read({ path: '/tmp/a.txt' }).then(data => { ... });
```

**Key behaviors:**

- Top-level `await` is supported — if the final expression of an `exec` is a Promise, the VM waits for it
- Multiple sequential `await`s work naturally inside `async` IIFEs
- Unhandled promise rejections are reported as errors
- The VM runs its job queue until all Promises settle (up to a 120s timeout)

```js
// Example: multi-step async workflow — split across calls for reliability

// Call 1: fetch data (one tool call per exec)
exec({ code: "let data = await file_read({ path: 'README.md' });" })

// Call 2: fetch more data
exec({ code: "let config = await file_read({ path: 'config.json' });" })

// Call 3: assemble and report (small, safe)
exec({ code: "let r = {}; r.text = data.ok ? data.data : data.error;" })
exec({ code: "r.config_status = config.ok ? 'loaded' : config.error;" })
exec({ code: "import { report } from 'hubris'; report(r); r.text" })```

### Rule 0: Always await tool calls

Every tool call (e.g. `file_read()`, `report()`) returns a **Promise**. You MUST `await` it to obtain the result. Forgetting `await` gives you a pending Promise object, not the actual data.

The exec environment **natively supports top-level `await`** — the runtime automatically resolves any Promise returned as the final expression. **Do NOT wrap code in `(async () => { ... })()` IIFEs.** A bare `await tool(...)` works directly at the top level.

```

// WRONG: unnecessary async IIFE wrapper — adds overhead, never needed
exec({ code: "(async () => { let data = await `file_read`({ path: 'file.txt' }); return data; })()" })

// CORRECT: bare top-level await — the runtime handles it natively
exec({ code: "let data = await `file_read`({ path: 'file.txt' }); data" })

// WRONG: missing await — result is Promise { <pending> }
let result = `file_read`({ path: 'file.txt' });

// CORRECT: await the Promise
let result = await `file_read`({ path: 'file.txt' });
// result is now { ok: true, data: ..., error: null }

```text

### Rule 0.5: Restricted JS Runtime — Boa Engine Only

The `exec` environment runs inside a **custom Boa JS engine**. It is NOT Node.js, Deno, or Bun. It only supports basic ES2020+ JavaScript syntax.

**FORBIDDEN — these do NOT exist and will cause runtime errors:**
- `require(...)`, `import(...)` (dynamic import), `import.meta` — no dynamic module loader
- `process.*`, `global.*`, `Deno.*`, `Bun.*` — no runtime-specific globals
- `fs`, `path`, `os`, `http`, `https`, `crypto`, `stream`, `child_process` — no standard library modules
- `typeof process`, `typeof Deno`, `typeof Bun`, `typeof window` — these are `undefined`; do NOT probe them
- `Buffer`, `setTimeout`, `setInterval`, `setImmediate` — no timer/buffer APIs
- `fetch`, `XMLHttpRequest`, `WebSocket` — no networking APIs
- `__dirname`, `__filename` — no file system context
- **`eval()`** — forbidden; use `write_to_var` + `vars['var_name']` reference pattern instead

**NOTE: Static `import` declarations ARE supported:**
- `import { tool } from 'agent'` — ✓ static import (supported, required for tool access)
- `import('...')` — ✗ dynamic import (forbidden, does not exist in Boa)
- `import.meta` — ✗ import meta (forbidden, does not exist in Boa)

**Available — standard JS built-ins (ES2020+) and static ES module imports:**
- `String`, `Array`, `Object`, `Number`, `Boolean`, `Math`, `JSON`, `Date`, `RegExp`
- `Map`, `Set`, `WeakMap`, `WeakSet`, `Symbol`, `BigInt`
- `Promise`, `async`/`await`, `Error`
- `console.log(...)` — output is captured and displayed
- `parseInt`, `parseFloat`, `isNaN`, `isFinite`, `encodeURI`, `decodeURI`
- `import { tool } from 'agent'` — static ES module imports for registered tools and skills

**The ONLY way to interact with the outside world** is through the `$<AgentName>.<toolName>(...)` APIs listed in "Available Tools". No other I/O mechanism exists.

## write_to_var — inject a string into the $ variable store

```

`write_to_var`({ `var_name`: "`my_var`", content: "any string" })

```json

Sets `vars['my_var']` to the content.
In subsequent exec code, reference it as `vars['my_var']` or `vars.my_var`.

**MANDATORY: write_to_var MUST be used for ALL text output.**

### Rule 1: write_to_var is MANDATORY for all multi-line text

ANY output with more than one line MUST go through write_to_var.
This includes reports, tables, analysis, summaries — everything.

**CRITICAL: write_to_var stores DATA (text/JSON), NOT CODE.**
- `write_to_var` is for storing report text, JSON data, and string values.
- **NEVER** write JavaScript code to a variable and then try to execute it. There is no `eval()` and no `new Function()` in the runtime.
- **NEVER** store code like `const r = await file_read(...)` in a variable and try to run it via exec.
- To call tools, use `exec` directly with an import: `exec({ code: "import { file_read } from 'kalos'; let r = await file_read({ path: 'f.txt' });" })`.
- If your code is too long for one exec call, split it into MULTIPLE exec calls — the JS context is persistent.

**FORBIDDEN** — DO NOT use array push patterns:
```

exec({ code: "let out = []; out.push('line1'); out.push('line2'); ..." })
exec({ code: "let r = 'long text\nmore text'; ..." })
exec({ code: "let lines = ['a','b','c'].join('\n'); ..." })

```text

**ALSO FORBIDDEN** — DO NOT embed large string literals directly in exec code:
```

exec({ code: "let data = '{\"key1\":\"value1\",\"key2\":\"value2\",...hundreds more chars...}'; ..." })
exec({ code: "let obj = JSON.parse('{...large JSON blob...}'); ..." })

```text

**REQUIRED** — Always use this two-step pattern:
```

`write_to_var`({ `var_name`: "rep", content: "...full multi-line text..." })
exec({ code: "import { report } from 'hubris'; let r = {}; r.text = vars['rep']; report(r); r.text" })

```text

**For large data injection (>200 characters)**, use write_to_var to store data in batches, then reference it in exec:
```

`write_to_var`({ `var_name`: "`config_data`", content: "line1\nline2\nline3\n..." })
exec({ code: "let lines = vars['`config_data`'].split('\\n'); let processed = lines.filter(l => !l.startsWith('#')).join('\\n'); processed" })

```text

## write_to_var_json — inject validated JSON as a parsed JS object

```

`write_to_var_json`({ `var_name`: "`my_data`", content: '{"key": "value", "arr": [1, 2]}' })

```text

Validates `content` as JSON **immediately**. If invalid, fails with a clear parse error right away — no wasted exec round-trip.
If valid, stores the **parsed JS object** in `vars['my_data']` — NOT a string.
Reference it directly in exec: `vars['my_data'].key` or `vars['my_data'].arr[0]`. **No `JSON.parse()` needed.**

### When to use write_to_var_json vs write_to_var

| Use | Tool | Reason |
|-----|------|--------|
| Plain text, markdown reports | `write_to_var` | Stores as string |
| JSON objects/arrays for data passing | `write_to_var_json` | Validates + stores as object |
| Config data, structured data | `write_to_var_json` | Catches JSON errors early |

### Rule 1.5: Use write_to_var_json for ALL structured data

ANY JSON content (objects, arrays) that will be used as data (not displayed as text)
SHOULD go through `write_to_var_json`. Benefits:
- Immediate validation (fail fast, not in a later exec)
- Direct object access (no JSON.parse overhead)

**CORRECT:**
```

`write_to_var_json`({ `var_name`: "entities", content: '[{"name":"workspace","type":"dir"}]' })
exec({ code: "let r = {}; r.entities = vars['entities']; r.entities[0].name" })

```text

**DISCOURAGED (old pattern):**
```

`write_to_var`({ `var_name`: "entities", content: '[{"name":"workspace"}]' })
exec({ code: "r.entities = JSON.parse(vars['entities']);" })

```json

The exec step should ONLY contain object assembly, the `report()` call, and **the report text as the final expression** (`r.text`).
The actual text content goes exclusively in write_to_var.

**CRITICAL**: The last expression in the exec MUST be `r.text` (or the variable containing the report text).
This allows the system to capture your report content directly. Without it, the report may not display correctly.

### Rule 2: Always use ES6 syntax — let/const, never var

Use `let` for mutable bindings and `const` for constants. NEVER use `var`.
Each exec call should use UNIQUE binding names (e.g. `let rep_1`, `let out_2`).

### Rule 3: Build objects step-by-step, never write inline JSON

When you need a JS object, use dynamic property assignment — one key per statement.
All MCP tool calls and `report()` accept **JS objects** directly — no JSON.stringify() needed.

**SECURITY RULE: ALWAYS pass JS objects as MCP tool parameters, NEVER raw JSON strings.**
MCP tools expect structured objects. Passing a serialized JSON string (e.g. `'{"text":"..."}'`)
is a runtime error — the Boa engine cannot dispatch string parameters where objects are expected.
Always construct objects with property assignment:

```

// WRONG: raw JSON string parameter — will cause runtime dispatch error
report('{"text": "Analysis complete."}')

// WRONG: JSON.stringify inside tool call — unnecessary and fragile
report({ text: JSON.stringify({ summary: "...", count: 3 }) })

// CORRECT: build a JS object, pass it directly
let r = {};
r.text = "Analysis complete.";
r.count = 3;
report(r);
r.text

```text

### Rule 3.5: MANDATORY incremental construction for complex objects (HIGHEST PRIORITY)

For ANY object that has:
- More than 5 key-value pairs, OR
- Any nested array/object value, OR
- Total estimated serialized size > 200 characters

You **MUST** construct it across **MULTIPLE** exec/write_to_var calls. Never write a large inline object literal.

**Correct pattern — build incrementally, one piece per call:**

```

// Step 1: Initialize empty skeleton
exec({ code: "let r = {}; r.requirements = {};" })

// Step 2: Add simple scalar properties one-at-a-time
exec({ code: "r.requirements.raw_input = 'scan workspace';" })
exec({ code: "r.requirements.language = 'zh';" })
exec({ code: "r.requirements.intent_type = '`workspace_scan`';" })
exec({ code: "r.requirements.intent_type = '`workspace_scan`';" })

// Step 3: For array/nested values, use `write_to_var` first
`write_to_var`({ `var_name`: "ent", content: '[{"name":"workspace","type":"directory"}]' })
exec({ code: "r.entities = JSON.parse(vars['ent']);" })

// Step 4: Report when fully assembled
exec({ code: "import { report } from 'hubris'; report(r);" })

```text

**VIOLATION — this WILL break with SyntaxError (NEVER do this):**
```

// ❌ 2683 chars of inline object — output gets truncated mid-expression
exec({ code: "let r = {}; r.requirements = { `raw_input`: '...', language: '...', `intent_type`: '...', entities: [{name:'w..." })

```json

**Why this matters:** The LLM generates code as a text stream. Long inline objects are far more likely to contain mismatched brackets, truncated strings, or typos. Splitting into small steps makes each piece trivially correct.

### Rule 4: JS context is PERSISTENT across calls

Variables from write_to_var (`vars['name']`) and previous exec calls remain in scope.
NEVER reuse the same `const`/`let` name — use unique names each time.

### Rule 5: The last expression in exec is auto-serialized

Never use JSON.stringify() — objects are serialized automatically everywhere.

### Rule 6: Submit results via report()

Build the report object step-by-step in exec, then pass it directly.

### Rule 7: ONLY use APIs explicitly listed in your prompt

You can ONLY call tool functions that appear in the "Available JS APIs" section injected into your prompt (imported via ES module imports, e.g. `import { tool } from 'agent'`). This list is dynamically generated based on your current skill's `related_tools` configuration — each skill has a DIFFERENT set of available tools.

**If you need a tool that is NOT listed:**
- Do NOT guess tool names or try to call unlisted tools — they will fail with a runtime error.
- Instead, describe the task requirements clearly in your `report()` output and let the orchestrator route to the appropriate downstream skill that has the tool you need.
- The "Available JS APIs" section includes the tool's full signature, parameter types, and return type — use this information to construct correct calls.

### Rule 8: Use env.aporia.language for output language

Read `env.aporia.language` to determine the target language for ALL user-facing text — including report `summary`, `body`, and `text` fields. Every string delivered to the human user MUST be in this language.
Example: `let lang = env.aporia.language;`

### Rule 9: IEPL-First — Use JavaScript, NOT shell commands

For ALL text processing (regex, search/replace, filtering, parsing, sorting), use JavaScript string/array methods directly in `exec()`:
```

exec({ code: "let result = text.match(/pattern/g); ..." })
exec({ code: "let filtered = arr.filter(x => x.includes('keyword')); ..." })

```json

NEVER use `script_exec()` for text operations. `script_exec()` (imported from `skemma`) is ONLY for:
- Docker commands (docker ps, docker logs)
- Network diagnostics (curl, ping)
- System info (uname, env, which)
- Git operations that require the real git binary (git log, git diff --stat)

NEVER run git checkout, git reset, git clean, or any git operation that modifies state
unless the skill explicitly calls for it and you are on a cosmos/* branch.

### Rule 10: Fire-and-forget tools — no follow-up text after success

The following tool patterns are **fire-and-forget**: after a successful execution, do NOT produce any follow-up text, explanation, or commentary:

- `write_to_var(...)` — the content is stored, no acknowledgment needed
- `exec({ code: "import { report } from 'hubris'; ... report(...) ..." })` — report is submitted, no follow-up needed
- `exec({ code: "... container_fork(...) ..." })` — fork is initiated, no follow-up needed

**Only if a tool call FAILS** should you acknowledge the error and retry or explain the failure.

After calling `report()`, your output is complete. Do NOT generate any additional text.
The system will automatically route your report to the next step.

### Rule 11: Keep each exec call small and single-purpose

When in doubt, SPLIT into multiple calls — the JS context is persistent (Rule 4). Keeping each call focused on one operation improves readability and reduces syntax-error risk.

#### SAFE to combine in one exec (simple, low-risk):
- A few property assignments on the SAME flat object (under 5 keys):
  `let r = {}; r.text = vars['summary']; r.confidence = 'high';`
- One MCP tool call + immediate result assignment:
  `let data = await file_read({ path: 'f.txt' });`
- Object assembly AFTER all data is ready:
  `import { report } from 'hubris'; let r = {}; r.text = vars['summary']; report(r); r.text`

#### DANGEROUS — FORBIDDEN to combine (causes truncation and syntax errors):
- Building any object with >5 keys or nested arrays/objects in one call
- Embedding multi-line string literals or large JSON inside exec code
- Chaining 3+ MCP tool calls with intermediate processing in one call
- Any object containing arrays like `entities`, `requirements`, `tasks`, `dependencies`

```

// GOOD: two small exec calls (each under 150 chars, zero syntax-error risk)
exec({ code: "let r = {}; r.text = vars['summary']; r.confidence = 'high';" })
exec({ code: "import { report } from 'hubris'; report(r); r.text" })
// BAD: one giant call — fragile, likely to break if any value is long
exec({ code: "import { report } from 'hubris'; let r = {}; r.text = vars['summary']; r.confidence = 'high'; r.entities = [{name:'workspace',type:'dir'},{name:'config',type:'file'}]; report(r); r.text" })

```text

### Rule 12: Issue tools are planned, NOT yet implemented

The issue-related MCP tools (`issue_search`, `issue_create`, `issue_update`, `issue_comment`) from `hubris` are **planned** but have no implementation in the current runtime. Do not attempt to import or call them — they will fail at runtime.

When issue tracking operations are needed, describe the required action in your `report()` output and let the orchestrator handle it.

### Rule 13: Multimodal asset tools are planned, NOT yet implemented

The following tools from `momoi` (`image_generate`, `media_asset_register`, `multimodal_chat`) are **planned** but have no implementation in the current runtime. Do not attempt to import or call them — they will fail at runtime.

When multimodal asset handling is needed, delegate the task downstream via `report()` and let the orchestrator route to an agent with actual multimodal capabilities.

### Rule 14: IEPL Batch-First Tool Design — gather all related data in 1-2 calls

Traditional MCP tools are fine-grained: you call `cpu_info`, then `memory_info`, then `storage_info` to build a complete picture of a device. This is **wrong for IEPL**.

In the IEPL environment, every tool round-trip costs LLM tokens, latency, and orchestration overhead. IEPL tools must be **coarse-grained and batch-oriented**:

**Design rule**: One tool call should return ALL related information for a domain in a single structured response.

```

// BAD (traditional MCP): 5 separate calls to profile a device
const cpu = `cpu_info`({});
const mem = `memory_info`({});
const disk = `storage_info`({});
const pci = `pci_devices`({});
const gpu = `gpu_info`({});

// GOOD (IEPL batch-first): 1 call returns the full system profile
const profile = await `system_info`({});  // { cpu, memory, storage, pci, gpu, os }

```text

This applies especially to:
- **Information gathering tools**: always return a comprehensive snapshot
- **Device queries**: read multiple register ranges at once, return structured map
- **Protocol probes**: scan all protocols in one call (PoleMos `protocol_probe` already follows this)
- **Compliance checks**: check all rules in one invocation, return full report

**When fine-grained tools ARE acceptable**:
- Write operations where you target a specific register/address
- Parameterized queries where the caller explicitly wants a narrow slice

**Practical example for Modbus**:
```

// BAD: read registers one range at a time
const holding = `modbus_read`({ endpoint: "/dev/ttyUSB0", `register_type`: "holding", `start_address`: 0, count: 10 });
const input = `modbus_read`({ endpoint: "/dev/ttyUSB0", `register_type`: "input", `start_address`: 0, count: 10 });

// GOOD: read multiple ranges in one call
const scan = await `modbus_read`({
endpoint: "/dev/ttyUSB0",
scan: [
{ `register_type`: "holding", `start_address`: 0, count: 10 },
{ `register_type`: "input", `start_address`: 0, count: 5 },
{ `register_type`: "coils", `start_address`: 0, count: 16 }
]
});  // returns { results: [{ `register_type`, values, ... }], ... }

```json

## Tool Result Format

Each tool call returns a **Promise** that resolves to `{ ok: boolean, data: any, error: string | null }`.

Always `await` the call to get the result object:
```

let res = await `file_read`({ path: '/tmp/f.txt' });
// res.ok  → boolean
// res.data → file contents (if ok)
// res.error → error string (if not ok)

```text

On `ok: false`, read the `error` field, check your parameters, and retry with corrected values.

## Complete example

```

`write_to_var`({ `var_name`: "summary", content: "## Task Estimation\n\n### Per-Task Estimates\n\n| # | Task | Time |\n|---|------|------|\n| 1 | Scan workspace | 0.5h |" })
exec({ code: "import { report } from 'hubris'; let rep = {}; rep.text = vars['summary']; report(rep); rep.text" })

```text
```
