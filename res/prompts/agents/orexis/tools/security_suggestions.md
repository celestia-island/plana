+++
name = "security_suggestions"
agent = "orexis"

[description]
en = "Get security improvement suggestions"
zh-Hans = "获取安全改进建议"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `scope` | string | no | Scope of security suggestions (e.g., "config", "sandbox", "all") |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
