+++
name = "llm_chat"
agent = "aporia"


[description]
en = "Interact with a Large Language Model (LLM) to generate text responses"
zh-Hans = "与大语言模型（LLM）交互以生成文本响应"
zh-Hant = "與大型語言模型（LLM）互動以產生文字回應"
ja = "大規模言語モデル（LLM）と対話してテキスト応答を生成する"
ko = "대규모 언어 모델(LLM)과 상호작용하여 텍스트 응답 생성"
fr = "Interagir avec un Grand Modèle de Langage (LLM) pour générer des réponses textuelles"
es = "Interactuar con un Modelo de Lenguaje Grande (LLM) para generar respuestas de texto"
ru = "Взаимодействовать с большой языковой моделью (LLM) для генерации текстовых ответов"
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
