+++
title = "Security Policy"
description = """Do not open public issues for security vulnerabilities."""
lang = "en"
category = "meta"
+++

# Security Policy

## Reporting a Vulnerability

**Do not open public issues for security vulnerabilities.**

Report them privately via
[GitHub Security Advisories](https://github.com/celestia-island/arona/security/advisories/new).
If GitHub Security Advisories are unavailable to you, email the maintainer at
security@celestia.world with a clear description and reproduction steps.

## Scope

In scope:

- Authentication bypass, JWT/OAuth weaknesses, session handling flaws
- API key / credential disclosure or improper storage
- Authorization and RBAC enforcement gaps
- Injection vulnerabilities (SQL, command, SSRF, XSS)
- Insecure deserialization, path traversal, SSRF
- Issues that allow escalation of privilege or cross-tenant access

Out of scope:

- Vulnerabilities in upstream dependencies not exploitable through this project
- Self-hosted deployments with insecure configuration against documented guidance
- Denial-of-service against the public LLM provider endpoints

## Response

| Stage | Target |
| --- | --- |
| Agent acknowledgment | 10 minutes |
| Human acknowledgment | 1 calendar day |
| Initial assessment | 3 calendar days |
| Fix or mitigation | 30 calendar days (severity-dependent) |

Please include: (1) the affected component and version, (2) the attack vector
and impact, (3) reproduction steps, and (4) suggested mitigations.

## Supported Versions

Only the latest release line on the `main` / `dev` branches receives security
fixes.
