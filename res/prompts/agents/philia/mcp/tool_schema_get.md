+++
name = "tool_schema_get"
agent = "philia"

[description]
en = "Get the .d.ts type declaration for a specific MCP tool"
zhs = "获取指定MCP工具的.d.ts类型声明"
zht = "取得指定MCP工具的.d.ts型別宣告"
ja = "特定のMCPツールの.d.ts型宣言を取得する"
ko = "특정 MCP 도구의 .d.ts 타입 선언 가져오기"
fr = "Obtenir la déclaration de type .d.ts pour un outil MCP spécifique"
es = "Obtener la declaración de tipo .d.ts para una herramienta MCP específica"
ru = "Получить объявление типа .d.ts для конкретного инструмента MCP"
+++

# tool_schema_get

## Description

Retrieves the TypeScript `.d.ts` type declaration for a specified MCP tool. This provides full parameter and return type information, including optional fields, nested types, and JSDoc comments. Useful for developers integrating with the tool or for agents that need precise type information.

## Parameters

- **`agent_type`** (string, required): The agent that owns the tool (e.g., `"aporia"`, `"philia"`, `"orexis"`).
- **`tool_name`** (string, required): The exact tool name (e.g., `"llm_chat"`, `"memory_store"`).

## Returns

### On Success

```text
Tool schema retrieved

Agent: <agent_type>
Tool: <tool_name>

Type declaration:
  <.d.ts content with full type definitions>
```

### On Failure

```text
Tool schema retrieval failed

Error: Tool '<tool_name>' not found for agent '<agent_type>'
```

## Examples

### Example 1: Get llm_chat schema

Invocation:

```text
tool_schema_get
  agent_type: "aporia"
  tool_name: "llm_chat"
```

Return:

```text
Tool schema retrieved

Agent: aporia
Tool: llm_chat

Type declaration:
  interface LlmChatParams {
    prompt: string;
    model?: string;
    system_prompt?: string;
  }

  interface LlmChatResult {
    ok: boolean;
    model: string;
    tokens: string;
    response: string;
    error?: string;
  }
```

### Example 2: Unknown tool

Invocation:

```text
tool_schema_get
  agent_type: "aporia"
  tool_name: "nonexistent_tool"
```

Return:

```text
Tool schema retrieval failed

Error: Tool 'nonexistent_tool' not found for agent 'aporia'
```

## Important Notes

- **Exact names required**: Both `agent_type` and `tool_name` must match exactly (case-sensitive).
- **Discovery**: Use `agent_registry_get` to list available tools for an agent if unsure of the exact tool name.
- **TypeScript format**: The returned declaration follows `.d.ts` conventions with interfaces for parameters and results.
- **Versioning**: The schema reflects the currently deployed version of the tool. Schema changes may occur across agent updates.
