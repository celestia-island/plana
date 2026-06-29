+++
title = "AI Agent Identification & Commit Co-author Strategy"
description = """Design note: AI agent identification and commit co-author strategy."""
lang = "en"
category = "design"
subcategory = "router"
+++

# AI Agent Identification & Commit Co-author Strategy

## Overview

`evernight` participates in the celestia-island co-author strategy in two ways:

1. **As a commit host**: when an AI agent orchestrates a commit through evernight
   (agent on host A → evernight SSH/exec → host B → `git commit`), the host-side
   `commit-msg` hook (installed by `noa`) fires locally and stamps the commit with
   provenance metadata.
2. **As a transit provider**: when evernight relays model traffic, it can appear in
   the author email as the serving platform, making the transport hop auditable.

This document specifies evernight's role. The authoritative mechanism is defined in
`noa`'s design document; this covers the evernight-specific integration.

## Provider Identity Model

The author email uses the `celestia.world` trust namespace:

```
Display Name <provider-or-platform-id@celestia.world>
```

When evernight relays a model, the provider id reflects the relay:

```
GLM 5 <evernight.celestia.world@celestia.world>   # GLM 5 relayed via evernight
```

First-party providers keep their own domain (`anthropic.com`, `deepseek.com`,
`zhipuai.cn`, ...); third-party relays keep theirs (`opencode.ai`, `jdcloud.com`,
`openrouter.ai`, ...). This makes the chain "which model, through whom" visible on
every commit.

## Co-author Trailer

- Trailer key: `Co-authored-by` (git-recognised).
- One trailer per distinct model, in usage order.
- A chain run fully under YOLO cruise control additionally gets:
  `Co-authored-by: Entelecheia <demiurge@celestia.world>`.

## Embedded Token Usage

Appended after the co-author trailers (blank-line separated):

```
Co-authored-by: Claude Opus 4.8 (↑ 12.5k ↓ 8.3k ●45.2k) <anthropic.com@celestia.world>
Co-authored-by: Deepseek V4 Pro (↑ 5.1k ↓ 3.2k) <deepseek.com@celestia.world>
```

- `Upload` = input tokens; `Download` = output tokens.
- `Cache` appears only when cached-input tokens were reported and are > 0.
- Counts in thousands (`k`), one decimal place, trailing-zero trimmed.

## evernight Integration Points

### Host-side hook

Commits made through `evernight`'s `Command.Exec` JSON-RPC (used by entelecheia's
surgery pipeline and the `KaLos:auto_fix` loop) invoke the system `git`, so the
`.git/hooks/commit-msg` hook installed by `noa hook install` applies unchanged. No
evernight code change is required for commits made on a host where the hook is
installed.

### Transit provider identity

When evernight proxies LLM traffic (e.g. routing a model call to a remote host's
local inference), the co-author resolver can be told the relay endpoint so the
provider id becomes `evernight.celestia.world`. This is configured through the same
`aporia.toml` provider list that `noa co-author resolve` reads.

## Full Commit Message Example

```
perf(screen): cache X11 connection to avoid per-frame reconnect

X11CaptureBackend previously called x11rb::connect on every capture_frame.
Cache the connection in a Mutex<Option<..>>, reusing it across frames.

Co-authored-by: Entelecheia <demiurge@celestia.world>
Co-authored-by: Deepseek V4 Pro (↑ 18.2k ↓ 2.1k) <deepseek.com@celestia.world>
```

## Security Considerations

- Co-author trailers are self-reported provenance, not cryptographic proof.
- The resolver degrades safely: a missing `noa` or parse error yields an empty block
  and the commit proceeds untouched.
- Provider identifiers come from the local `aporia.toml`, reflecting the configured
  providers.

## Provider Identifier Reference (initial registry)

| Provider id | Brand | Endpoint hint |
| --- | --- | --- |
| `zhipuai.cn` | GLM | `open.bigmodel.cn` |
| `deepseek.com` | Deepseek | `api.deepseek.com` |
| `anthropic.com` | Claude | `api.anthropic.com` |
| `openai.com` | GPT / OpenAI | `api.openai.com` |
| `evernight.celestia.world` | (relay) | evernight proxy |
| `opencode.ai` | (relay) | `opencode.ai` |
| `jdcloud.com` | (relay) | `jdcloud.com` |
