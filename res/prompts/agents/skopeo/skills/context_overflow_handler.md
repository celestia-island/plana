+++
name = "Context Overflow Handler"
agent = "skopeo"

[description]
en = "This skill intelligently compresses and summarizes historical content when conversation context approaches or exceeds limits, ensuring no loss of critical information."
zh-Hans = "此技能在对话上下文接近或超过限制时，智能压缩和总结历史内容，确保关键信息不丢失。"
zh-Hant = "此技能在對話上下文接近或超過限制時，智慧壓縮和總結歷史內容，確保關鍵資訊不遺失。"
ja = "このスキルは会話コンテキストが制限に近づくまたは超過した際、履歴コンテンツをインテリジェントに圧縮・要約し、重要な情報の損失を防ぎます。"
ko = "이 스킬은 대화 컨텍스트가 한계에 근접하거나 초과할 때 이력 콘텐츠를 지능적으로 압축 및 요약하여 중요 정보의 손실을 방지합니다."
fr = "Cette compétence compresse et résume intelligemment le contenu historique lorsque le contexte de conversation approche ou dépasse les limites, garantissant aucune perte d'informations critiques."
es = "Esta habilidad comprime y resume inteligentemente el contenido histórico cuando el contexto de conversación se acerca o excede los límites, asegurando que no se pierda información crítica."
ru = "Этот навык интеллектуально сжимает и обобщает историческое содержимое, когда контекст разговора приближается к лимиту или превышает его, гарантируя сохранность критически важной информации."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[features]
execution_mode = "read"
location = "cosmos"
+++

Intelligently compress and summarize conversation history when context usage approaches token limits, preserving all critical information.

## SoP

1. **Detect overflow risk** — Monitor context token usage. When usage exceeds the configured threshold (default 85% of max), trigger compression automatically.
1. **Identify key nodes** — Scan conversation history for: decisions made, action items, code changes, error resolutions, and user-marked important content. These are preserved verbatim.
1. **Classify content** — Partition messages into three tiers: (a) must-preserve key nodes, (b) recent N rounds (default 10), (c) older compressible content.
1. **Generate summaries** — For compressible content, call `llm_chat()` with the raw messages and instructions to produce a concise summary capturing: topics discussed, conclusions reached, and unresolved items.
1. **Reconstruct context** — Replace compressible messages with the generated summary. Retain key nodes and recent messages unchanged. Verify the reconstructed context is below the target size.
1. **Validate integrity** — Cross-check that all decisions, action items, and critical data points are present in the compressed context. If any are missing, restore relevant original messages.
1. **Report** — Call `report()` with compression metrics. Log the compression event for future analysis.

> Return type and IEPL enforcement: @system/return-type-convention
