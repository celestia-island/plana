+++
name = "block_tool"
agent = "orexis"

[description]
en = "Block a tool from being used"
zh-Hans = "阻止某个工具被使用"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `agent` | string | yes | Agent name pattern to block (* for all agents) |
| `tool` | string | yes | Tool name pattern to block (* for all tools) |
| `reason` | string | no | Reason for blocking the tool |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
