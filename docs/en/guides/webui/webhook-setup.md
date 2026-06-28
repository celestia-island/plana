+++
title = "Webhook Configuration Guide"
description = """> Audience: Administrators integrating external services with shittim-chest."""
lang = "en"
category = "guides"
subcategory = "webui"
+++

# Webhook Configuration Guide

> **Audience**: Administrators integrating external services with shittim-chest.
> **Last updated**: 2026-05-25

## Overview

Webhooks allow external services (GitHub, GitLab, Gitee) to send real-time events to shittim-chest. Events are validated, parsed, and forwarded to scepter which dispatches them to the appropriate agent.

```text
External Service → shittim_chest → scepter → Agent
```

`shittim_chest` also supports custom webhook endpoints for services not natively supported.

## GitHub Webhook Setup

### Step 1: Configure Environment

Set the webhook secret in your `.env`:

```bash
WEBHOOK_GITHUB_SECRET=your-hmac-secret-here
WEBHOOK_PUBLIC_URL=https://your-domain.com
```

Generate a strong secret:

```bash
openssl rand -hex 32
```

### Step 2: Create the Webhook in GitHub

1. Navigate to your repository → **Settings** → **Webhooks** → **Add webhook**
1. Set **Payload URL** to `https://your-domain.com/api/webhook/github`
1. Set **Content type** to `application/json`
1. Set **Secret** to the same value as `WEBHOOK_GITHUB_SECRET`
1. Select events: `push`, `pull_request`, `issues`, `issue_comment`
1. Ensure **Active** is checked
1. Click **Add webhook**

### Step 3: Verify

GitHub will send a `ping` event immediately. Check the **Recent Deliveries** tab to confirm a `200` response.

## GitLab Webhook Setup

### Step 1: Configure Environment

```bash
WEBHOOK_GITLAB_SECRET=your-gitlab-secret-token
```

### Step 2: Create the Webhook in GitLab

1. Navigate to your project → **Settings** → **Webhooks**
1. Set **URL** to `https://your-domain.com/api/webhook/gitlab`
1. Set **Secret token** to the same value as `WEBHOOK_GITLAB_SECRET`
1. Select triggers: `Push events`, `Merge request events`, `Issue events`
1. Ensure **Enable SSL verification** is checked (for HTTPS)
1. Click **Add webhook**

### Step 3: Verify

Use the **Test** button in GitLab to send a test event. Confirm the delivery succeeds.

## Gitee Webhook Setup

Gitee (码云) webhooks are also supported.

### Step 1: Configure Environment

Gitee uses the same `WEBHOOK_GITLAB_SECRET` for HMAC validation (with token as fallback). Alternatively, set `WEBHOOK_GITEE_PASSWORD` if using password-based auth.

### Step 2: Create the Webhook in Gitee

1. Navigate to your repository → **Management** → **Webhooks**
1. Set **URL** to `https://your-domain.com/api/webhook/gitee`
1. Set **Password/Signing Key** to the same secret
1. Select events: `Push`, `Pull Request`, `Issues`
1. Click **Add**

## Custom Webhooks

`shittim_chest` supports a generic custom webhook endpoint at `/api/webhook/custom/{name}`. To add a custom webhook source:

1. Set `WEBHOOK_PUBLIC_URL` in `.env`
1. Configure your external service to POST to `https://your-domain.com/api/webhook/custom/{name}`
1. Events are forwarded to scepter with the webhook name as the event source

For integrating new webhook providers at the code level:

1. Add a handler in `packages/core/src/webhook.rs`
1. Implement HMAC or token validation for the new provider
1. Parse the custom event format and forward to scepter via Unix socket

## IP Whitelist

`shittim_chest` supports IP whitelisting for webhook sources to reject requests from unknown origins:

```bash
# .env
WEBHOOK_IP_WHITELIST=140.82.112.0/20,192.30.252.0/22  # GitHub IPs
```

Configure CIDR ranges for each webhook provider. Requests from IPs outside the whitelist are rejected.

## Event Types

Supported events and their mapping to scepter triggers:

| Source | Event | scepter `event_type` |
| --- | --- | --- |
| GitHub | `push` | `github.push` |
| GitHub | `pull_request` | `github.pull_request` |
| GitHub | `issues` | `github.issues` |
| GitHub | `issue_comment` | `github.issue_comment` |
| GitLab | `push` | `gitlab.push` |
| GitLab | `merge_request` | `gitlab.merge_request` |
| GitLab | `issues` | `gitlab.issues` |
| Gitee | `push` | `gitee.push` |
| Gitee | `pull_request` | `gitee.pull_request` |
| Gitee | `issues` | `gitee.issues` |

## Delivery Log

`shittim_chest` maintains a delivery log of webhook events. Duplicate deliveries are detected using a LRU cache (up to 10,000 delivery IDs). Access delivery logs via:

- **REST API**: `GET /api/webhook/deliveries`
- Admin panel: **Webhooks** → **Delivery Log**

## Security

All webhooks must pass signature verification:

- **GitHub**: Uses `X-Hub-Signature-256` header. Validated against `WEBHOOK_GITHUB_SECRET`.
- **GitLab**: Uses `X-Gitlab-Token` header. Validated against `WEBHOOK_GITLAB_SECRET`.
- **Gitee**: Uses HMAC-SHA256 signature with token fallback.

Requests without valid signatures are rejected with `401 Unauthorized`. Never expose webhook secrets in client-side code or logs.

## Testing

Use the admin panel to test webhook integration:

1. Log in to admin panel (default `:3000`)
1. Navigate to **Webhooks** in the sidebar
1. View delivery logs and configuration
1. Test endpoints via the external service's test functionality

You can also test manually with curl:

```bash
curl -X POST https://your-domain.com/api/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=<computed-hmac>" \
  -d '{"action":"push","ref":"refs/heads/main"}'
```

## Troubleshooting

### 401 Unauthorized

**Cause**: HMAC signature mismatch or IP not in whitelist.
**Fix**: Ensure the secret in `.env` matches the secret configured in the source platform. Check for trailing whitespace or encoding issues. Verify IP whitelist configuration.

### 502 Bad Gateway

**Cause**: scepter is not reachable.
**Fix**: Verify `ENTELECHEIA_SCEPTER_URL` and `ENTELECHEIA_TUI_SOCK` in `.env`. Ensure the scepter instance is running and the Unix socket path is accessible.

### Events not reaching agents

**Cause**: Event type not mapped or agent not configured to handle it.
**Fix**: Check backend logs for the parsed `event_type`. Verify the target agent has a handler registered for that event. Check the delivery log via API or admin panel.

### Duplicate deliveries

**Cause**: The external service is retrying due to timeout. `shittim_chest` automatically detects duplicates via LRU cache.
**Fix**: If valid retries are being blocked, increase the delivery ID cache size. Ensure `shittim_chest` responds within the service's timeout window (GitHub: 10 seconds).
