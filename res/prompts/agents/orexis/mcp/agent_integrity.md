+++
name = "agent_integrity"
agent = "orexis"

[description]
en = "Check agent integrity and code authenticity"
zhs = "检查代理完整性和代码真实性"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `verbose` | boolean | no | If true, provide detailed integrity report with agent-level breakdowns |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
