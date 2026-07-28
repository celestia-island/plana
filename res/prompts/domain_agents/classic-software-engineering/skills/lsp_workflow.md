+++
name = "lsp_workflow"
agent = "classic-software-engineering"
execution_mode = "read"

[description]
en = "Orchestrate LSP-based diagnostics, symbol navigation, and refactoring via NeiKos sidecar processes running language servers."
zh-Hans = "LSP工作流：通过NeiKos sidecar编排语言服务器的诊断、符号导航和重构"
zh-Hant = "LSP工作流：透過NeiKos sidecar編排語言伺服器的診斷、符號導航和重構"
ja = "LSPワークフロー：NeiKosサイドカーで言語サーバーの診断、シンボルナビゲーション、リファクタリングをオーケストレーション"
ko = "LSP 워크플로우: NeiKos 사이드카를 통해 언어 서버의 진단, 심볼 내비게이션 및 리팩토링 오케스트레이션"
fr = "Workflow LSP : orchestration des diagnostics, navigation de symboles et refactoring via les processus sidecar NeiKos"
es = "Flujo de trabajo LSP : orquestación de diagnósticos, navegación de símbolos y refactoring a través de procesos sidecar NeiKos"
ru = "Рабочий процесс LSP : оркестрация диагностики, навигации по символам и рефакторинга через сайдкар-процессы NeiKos"

[[related_tools]]
name = "lsp_diagnose"
agent = "classic_software_engineering"
description = "Run language server diagnostics on source files"

[[related_tools]]
name = "lsp_symbols"
agent = "classic_software_engineering"
description = "Query document symbols from language server"

[[related_tools]]
name = "lsp_refactor"
agent = "classic_software_engineering"
description = "Execute refactoring operations via language server"

[[related_tools]]
name = "sidecar_spawn"
agent = "neikos"
description = "Spawn a sidecar process (LSP server) via NeiKos"

[[related_tools]]
name = "sidecar_send"
agent = "neikos"
description = "Send JSON-RPC request to a running sidecar process"

[[related_tools]]
name = "sidecar_kill"
agent = "neikos"
description = "Terminate a running sidecar process"

[[related_tools]]
name = "file_read"
agent = "kalos"
description = "Read source file content for LSP analysis"

[[related_tools]]
name = "report"
agent = "hubris"
description = "Emit LSP analysis findings"
+++

# lsp_workflow

Orchestrate LSP-based code analysis workflows using NeiKos sidecar processes. Coordinates language server processes (rust-analyzer, typescript-language-server, pyright, gopls) running as NeiKos sidecar child processes via the `sidecar_spawn`/`sidecar_send`/`sidecar_kill` protocol.

## Preconditions

- Target file path and language are known
- NeiKos container is running and sidecar protocol is available
- Toolchain profile for the target language exists in `image/` directory

## SOP

### Step 1: Language Detection and Toolchain Resolution

- Detect language from file extension: `.rs` → rust, `.ts`/`.js` → typescript, `.py` → python, `.go` → go
- Resolve toolchain profile from `image/<lang>-lsp.yaml`
- **Gate**: If language is unsupported → error `"No LSP toolchain for language: <lang>"`

### Step 2: Spawn Language Server

```bash
$ sidecar_spawn({
  name: "lsp-<session-id>",
  language: <lang>,
  framing: "content_length"
})
```

- Auto-resolves the correct LSP binary from toolchain profile
- **Gate**: If spawn fails → error `"Failed to spawn LSP server: <reason>"`

### Step 3: Initialize LSP

```bash
$ sidecar_send({
  name: "lsp-<session-id>",
  method: "initialize",
  params: {
    processId: null,
    rootUri: "file:///workspace",
    capabilities: {}
  },
  timeout_secs: 30
})
```

```bash
$ sidecar_send({
  name: "lsp-<session-id>",
  method: "initialized",
  params: {}
})
```

- **Gate**: If initialize times out → kill sidecar, retry once, then error

### Step 4: Open Document

```bash
$ file_read(path=<target_file>)
```

```bash
$ sidecar_send({
  name: "lsp-<session-id>",
  method: "textDocument/didOpen",
  params: {
    textDocument: {
      uri: "file:///workspace/<file_path>",
      languageId: <lang>,
      version: 1,
      text: <file_content>
    }
  }
})
```

### Step 5: Request Analysis

Based on the requested operation:

**For diagnostics:**

```bash
$ sidecar_send({
  name: "lsp-<session-id>",
  method: "textDocument/diagnostic",
  params: { textDocument: { uri: "file:///workspace/<file_path>" } },
  timeout_secs: 60
})
```

**For symbols:**

```bash
$ sidecar_send({
  name: "lsp-<session-id>",
  method: "textDocument/documentSymbol",
  params: { textDocument: { uri: "file:///workspace/<file_path>" } },
  timeout_secs: 30
})
```

**For refactoring:**

```bash
$ sidecar_send({
  name: "lsp-<session-id>",
  method: "textDocument/codeAction",
  params: {
    textDocument: { uri: "file:///workspace/<file_path>" },
    range: { start: { line: <start>, character: 0 }, end: { line: <end>, character: 0 } },
    context: { diagnostics: [], only: ["refactor"] }
  },
  timeout_secs: 30
})
```

### Step 6: Parse Response

- Parse LSP JSON-RPC response into structured result types (`LspDiagnoseResult`, `LspSymbolsResult`, `LspRefactorResult`)
- **Gate**: If response contains errors → log error, return partial results with error annotation

### Step 7: Shutdown and Cleanup

```bash
$ sidecar_send({ name: "lsp-<session-id>", method: "shutdown", params: {} })
$ sidecar_kill({ name: "lsp-<session-id>" })
```

- Always clean up the sidecar process, even on error

### Step 8: Report Findings

```bash
$ report({
  file_path: <target_file>,
  language: <lang>,
  analysis_type: <diagnostics|symbols|refactor>,
  results: <parsed_results>
})
```

## Supported Language Servers

| Language | Language Server | Toolchain Profile |
| --- | --- | --- |
| Rust | `rust-analyzer` | `rust-lsp` |
| TypeScript/JS | `typescript-language-server` | `nodejs-lsp` |
| Python | `pyright-langserver` | `python-lsp` |
| Go | `gopls` | `go-lsp` |

## Postconditions

- LSP analysis results returned in structured format
- Sidecar process cleaned up (no orphan processes)
- Results compatible with downstream skills (`code_review`, `code_health_check`, etc.)

## Example IEPL TypeScript

```typescript
const lang = "rust";
const fileName = "src/main.rs";
const fileContent = await file_read({ path: fileName });
const sidecarName = `lsp-${Date.now()}`;

await sidecar_spawn({ name: sidecarName, language: lang, framing: "content_length" });
await sidecar_send({ name: sidecarName, method: "initialize", params: { processId: null, rootUri: "file:///workspace", capabilities: {} }, timeout_secs: 30 });
await sidecar_send({ name: sidecarName, method: "initialized", params: {} });
await sidecar_send({ name: sidecarName, method: "textDocument/didOpen", params: { textDocument: { uri: `file:///workspace/${fileName}`, languageId: lang, version: 1, text: fileContent } } });

const diagResult = await sidecar_send({ name: sidecarName, method: "textDocument/diagnostic", params: { textDocument: { uri: `file:///workspace/${fileName}` } }, timeout_secs: 60 });

await sidecar_send({ name: sidecarName, method: "shutdown", params: {} });
await sidecar_kill({ name: sidecarName });

const diagnostics = diagResult.data?.response?.result?.items || [];
await report({ file_path: fileName, diagnostics });
```

@system/return-type-convention
