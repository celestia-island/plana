+++
title = "Webhook Platform Setup"
description = """> Description of the current webhook layout and integration scope"""
lang = "en"
category = "guides"
subcategory = "core"
+++

# Webhook Platform Setup

> Description of the current webhook layout and integration scope

## Overview

The repository already contains webhook integrations for code-hosting platforms and chat platforms, but the whole thing is still in a transitional phase and is not a fully unified, mature, settled solution.

The current directory structure contains both:

- Old per-platform directories, such as `plugins/github-webhook/github/`, `gitee/`, `gitlab/`, `telegram/`, `qq/`, `lark/`
- A newer TypeScript implementation: `plugins/github-webhook/ts/`

The TypeScript package currently integrates:

- GitHub
- Gitee
- GitLab
- Feishu / Lark
- QQ
- Discord
- Telegram

## What currently works

- Receiving webhook or bot events
- Forwarding events to Scepter via WebSocket or HTTP helper calls
- Providing a `/health` health-check endpoint in the TypeScript service

## What cannot currently be assumed

- A unified, stable deployment approach across all platforms
- A complete issue-driven skill chain for every platform
- The same level of maturity for every platform integration

## TypeScript package

Location: `plugins/github-webhook/ts/`

Development run:

```bash
cd plugins/github-webhook/ts
npm install
npm run dev
```

Production build:

```bash
cd plugins/github-webhook/ts
npm run build
npm start
```

## Key environment variables

- `PORT`: webhook service port, default `8000`
- `SCEPTER_URL`: HTTP forwarding address, default `http://localhost:8424`
- `SCEPTER_WS_URL`: WebSocket forwarding address, default `ws://localhost:8424/ws`

## Usage recommendations

You can treat the webhook capability as "existing, but with uneven maturity." If you rely on a particular platform, first check the actual implementation of the corresponding router or bot under `plugins/github-webhook/` before describing it as stable for production use.
