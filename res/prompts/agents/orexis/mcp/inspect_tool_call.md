+++
name = "inspect_tool_call"
agent = "orexis"

[description]
en = "Inspect a tool call for security compliance"
zhs = "检查工具调用的安全合规性"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `value` | string | yes | The tool call value or expression to inspect |
| `context` | string | no | Additional context for the inspection |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
