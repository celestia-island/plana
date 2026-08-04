+++
name = "translate_report"
agent = "aporia"
config = ["target_language"]

[description]
en = "Translate report content to the user's preferred language before delivering to human"
zh-Hans = "在报告送达人类之前，将报告内容翻译为用户偏好的语言"
zh-Hant = "在報告送達人類之前，將報告內容翻譯為使用者偏好的語言"
ja = "人間へのレポート配信前に、レポート内容をユーザーの優先言語に翻訳する"
ko = "인간에게 보고서를 전달하기 전에 보고서 내용을 사용자 선호 언어로 번역합니다"
fr = "Traduire le contenu du rapport dans la langue préférée de l'utilisateur avant de le livrer"
es = "Traducir el contenido del informe al idioma preferido del usuario antes de entregarlo"
ru = "Перевести содержание отчёта на предпочитаемый язык пользователя перед отправкой"

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
