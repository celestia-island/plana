+++
name = "set_security_policy"
agent = "orexis"

[description]
en = "Set the security policy for the agent"
zhs = "设置代理的安全策略"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `action` | string | yes | Policy action to set (e.g., "block", "allow", "warn") |
| `target` | string | yes | Policy target (tool name, agent name, or pattern) |
| `reason` | string | no | Reason for setting this policy |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
