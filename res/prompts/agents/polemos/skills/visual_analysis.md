+++
name = "Visual Analysis"
agent = "polemos"

[description]
en = "Visual Analysis is the core visual processing skill of the Polemos agent, specialized in analyzing screenshots and image content. This skill combines advanced computer vision technology to extract text, recognize UI elements, detect visual changes, and provide powerful support for automated testing and visual verification."
zh-Hans = "视觉分析是Polemos智能体的核心视觉处理技能，专注于分析截图和图像内容。该技能结合先进的计算机视觉技术，提取文本、识别UI元素、检测视觉变化，为自动化测试和视觉验证提供强大支持。"
zh-Hant = "視覺分析是Polemos智能體的核心視覺處理技能，專注於分析截圖和圖像內容。該技能結合先進的電腦視覺技術，提取文字、辨識UI元素、偵測視覺變化，為自動化測試和視覺驗證提供強大支援。"
ja = "ビジュアル分析はPolemosエージェントのコア視覚処理スキルであり、スクリーンショットと画像コンテンツの分析に特化しています。このスキルは高度なコンピュータビジョン技術を組み合わせ、テキスト抽出、UI要素認識、視覚変化検出を行い、自動テストと視覚検証を強力にサポートします。"
ko = "시각 분석은 Polemos 에이전트의 핵심 시각 처리 스킬로, 스크린샷 및 이미지 콘텐츠 분석에 특화되어 있습니다. 이 스킬은 고급 컴퓨터 비전 기술을 결합하여 텍스트 추출, UI 요소 인식, 시각적 변화 감지를 수행하며, 자동화 테스트 및 시각 검증을 강력하게 지원합니다."
fr = "L'analyse visuelle est la compétence de traitement visuel principal de l'agent Polemos, spécialisée dans l'analyse de captures d'écran et de contenu d'images. Cette compétence combine une technologie avancée de vision par ordinateur pour extraire du texte, reconnaître les éléments d'interface, détecter les changements visuels et fournir un support puissant pour les tests automatisés et la vérification visuelle."
es = "El análisis visual es la habilidad principal de procesamiento visual del agente Polemos, especializada en analizar capturas de pantalla y contenido de imágenes. Esta habilidad combina tecnología avanzada de visión por computadora para extraer texto, reconocer elementos de UI, detectar cambios visuales y brindar soporte para pruebas automatizadas y verificación visual."
ru = "Визуальный анализ — это основной навык визуальной обработки агента Polemos, предназначенный для анализа скриншотов и содержимого изображений. Этот навык объединяет передовые технологии компьютерного зрения для извлечения текста, распознавания элементов UI, обнаружения визуальных изменений и обеспечивает мощную поддержку автоматизированного тестирования и визуальной верификации."

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "remote_operations"
tool_name = "node_execute"

[features]
location = "cosmos"
execution_mode = "read"
+++

# Visual Analysis

Analyze screenshots, images, and visual content to extract text, recognize UI elements, detect changes, and diagnose errors.

## SoP

1. **Validate input** — Confirm the image source path or URL is accessible, the format is supported (PNG, JPG, GIF, WebP, BMP, TIFF), and the file size does not exceed 8 MB. If the source is ambiguous, use `report_human()` for clarification.
1. **Classify analysis type** — Determine the required analysis mode: OCR text extraction, UI element recognition, visual diff comparison, error screenshot diagnosis, technical diagram understanding, or data visualization analysis.
1. **Execute analysis** — Use `llm_chat()` to submit the image along with a targeted prompt describing the analysis goal. Include relevant context such as expected content, comparison baseline path, or error context.
1. **Validate results** — Check output completeness. If results appear unreliable, re-run with adjusted parameters if necessary.
1. **Generate report** — Produce a structured report via `report()` and a human-readable summary via `report_human()`. Include image metadata, analysis type, and key findings.
1. **Preserve evidence** — Record original file hashes (SHA256), analysis parameters, and result snapshots for traceability and backtracking.
1. **Escalate anomalies** — If visual regressions exceed threshold, critical text is unreadable, or UI elements remain unrecognized, escalate via `report_human()` with annotated findings.

> Return type and IEPL enforcement: @system/return-type-convention
