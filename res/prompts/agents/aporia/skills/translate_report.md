+++
name = "translate_report"
agent = "aporia"
config = ["target_language"]

[description]
en = "Translate report content to the user's preferred language before delivering to human"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
execution_mode = "read"
location = "cosmos"
+++

Translate report content into the user's preferred language using a lightweight LLM call before delivering to the human via `report_human()`.

## SoP

1. Read the `target_language` from the injected config and the report `content` (and optional `summary`) from the dispatch pipeline.
1. Detect the source language of the content; if it already matches the target language, return the content unchanged.
1. Call `llm_chat()` with a translation prompt instructing the model to translate faithfully without adding, removing, or summarizing information.
1. Preserve all markdown formatting, code blocks, headings, lists, and structural elements exactly as in the source.
1. Validate the translation using `llm_chat()` with a verification prompt to confirm the output language matches the target and no content was lost.
1. If validation fails (wrong language or missing sections), retry the translation once with stricter prompt constraints.
1. Return the translated content and summary for delivery via `report_human()`.

> Return type and IEPL enforcement: @system/return-type-convention
