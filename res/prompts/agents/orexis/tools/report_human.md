+++
name = "report_human"
agent = "orexis"

[description]
en = "Send a direct reply to the user and TERMINATE the skill chain — no further skills run. Use for conversational responses, opinions, chitchat, or any reply that doesn't need downstream processing."
+++

# report_human

## Description

Generates a human-readable report, typically used at the end of a skill chain to deliver results to the user. This variant includes compliance-aware formatting for Orexis-mediated workflows.

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `summary` | string | yes | Short one-line summary of the result |
| `body` | string \| null | no | Detailed multi-line report body (Markdown) |
| `text` | string \| null | no | Plain text fallback for terminals without Markdown rendering |
| `mode` | string \| null | no | Set to `"reply"` to signal reply-mode termination |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
