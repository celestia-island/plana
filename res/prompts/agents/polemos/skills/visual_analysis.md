+++
name = "Visual Analysis"
agent = "polemos"

[description]
en = "Visual Analysis is the core visual processing skill of the Polemos agent, specialized in analyzing screenshots and image content. This skill combines advanced computer vision technology to extract text, recognize UI elements, detect visual changes, and provide powerful support for automated testing and visual verification."

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
