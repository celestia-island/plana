+++
name = "llm_chat"
agent = "aporia"


[description]
en = "Interact with a Large Language Model (LLM) to generate text responses"
[[related_tools]]
name = "rag_db_read"
description = "Provide knowledge context to the LLM"


[[related_tools]]
name = "rag_db_write"
description = "Store LLM-generated content in the knowledge base"
+++

# llm_chat

Send a prompt to the LLM and receive a text response.

## Parameters

| Parameter | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `prompt` | string | yes | — | The user prompt to send |
| `model` | string | no | `"default"` | Model selector hint |
| `system_prompt` | string | no | `null` | Optional system prompt |

## Returns

```json
{ "ok": true, "model": "gpt-4o-mini", "tokens": "↑120 ↓340", "response": "..." }
```

## Notes

- Uses `ModelTier::Basic` for cost efficiency.
- Suitable for summarization, translation, classification, and analysis sub-tasks within skill execution.
