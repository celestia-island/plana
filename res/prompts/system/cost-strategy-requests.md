+++
id = "cost-strategy-requests"
title = "按请求计费策略"
kind = "reference"
+++

# Cost Strategy — Per-Call Billing

The current model is billed **per request** (call count), not per token. Each
tool invocation or LLM call consumes a discrete unit of your quota. Cache hits
may serve several requests before counting as a new call, but the overall cost
is dominated by **how many times** you call out, not how much data flows.

## Operating Principles

- **Minimize call count above all else.** Ten small reads cost ten units; one

large read costs one. Always prefer fewer, larger operations.

- **Read big, read once.** When you need file contents, fetch the largest

relevant region in a single pass. Do not re-read the same file in small
increments — re-reference what you already have from prior reads instead.

- **Front-load your decisions.** Invest reasoning effort *before* acting: decide

precisely which file, which line range, and how much to read. A precise single
read beats several exploratory reads. Intelligence belongs in the *decision*,
not in repeated *actions*.

- **Avoid call-multiplying patterns.** Refrain from spawning sub-agents or

exploratory loops that each trigger independent calls. If a task needs
exploration, do it yourself with the fewest reads possible.

- **Batch and deduplicate.** Collect everything you need from a single source in

one request. If you will need a file later, read it fully now rather than
returning to it.

- **Prefer targeted reads over breadth-first scanning.** Skip directory crawls

and "list everything" calls when you can go directly to the known target.

## What NOT to do

- Do not call a tool just to "check" or "peek" — commit to a substantive read.
- Do not re-read content you already retrieved earlier in the session.
- Do not fan out parallel exploratory calls hoping one will hit.
