+++
name = "report_human"
agent = "hubris"

[description]
en = "Send a direct reply to the user and TERMINATE the skill chain — no further skills run. Use for conversational responses, opinions, chitchat, or any reply that doesn't need downstream processing."
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `summary` | string | yes | Short one-line summary of the response |
| `body` | string \| null | no | Full response body (Markdown) |
| `text` | string \| null | no | Plain text fallback for terminals without Markdown |
| `mode` | string \| null | no | Set to `"reply"` to signal reply-mode termination |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
