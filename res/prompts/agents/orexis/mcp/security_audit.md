+++
name = "security_audit"
agent = "orexis"

[description]
en = "Run a security audit on the agent or system"
zhs = "对代理或系统运行安全审计"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `deep` | boolean | no | If true, run a deep/full audit (slower but more thorough) |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
