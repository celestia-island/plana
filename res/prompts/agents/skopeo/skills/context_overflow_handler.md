+++
name = "Context Overflow Handler"
agent = "skopeo"

[description]
en = "This skill intelligently compresses and summarizes historical content when conversation context approaches or exceeds limits, ensuring no loss of critical information."

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
