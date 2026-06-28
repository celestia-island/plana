+++
title = "Fundamentals"
description = """> Concept explanations based on the current reality of the code"""
lang = "en"
category = "guides"
subcategory = "core"
+++

# Fundamentals

> Concept explanations based on the current reality of the code

## Overview

Entelecheia is a multi-agent platform that uses a small model-visible tool surface, a shared runtime, and multiple client entry points. Because the repository contains both the current implementation, experimental capabilities, and design documents, this guide only explains the core concepts that are already active in the current code.

## Core concepts

### Agent

An Agent is a runtime role with prompts, skills, and MCP tools.

- Layer1 is the current platform's core capability.
- The active built-in Layer2 in the current workspace is Web Automation.
- Layer3 (design phase) is planned to load from the `.amphoreus/` directory — not yet implemented.

### Exec-Only tool surface

The model does not directly see all MCP tools. The currently primary model-visible tools are:

- `exec`
- `write_to_var`
- `write_to_var_json`

Within the runtime, code in `exec` can invoke tool functions via ES module imports (e.g. `import { tool } from 'agent'`).

### MCP tools

MCP tools are internal structured capability interfaces.

- Some are truly implemented.
- Some are partially implemented.
- Some are still stubs or parameter-validation skeletons.

Therefore, you should not assume by default that every tool appearing in the documentation is already stable and deliverable.

### Skill

A Skill is a prompt-defined workflow that references related tools and sometimes other skills.

- Some skills can already drive real workflows.
- Some skills are closer to SOP documents than complete automation chains.

### Tiers

| Tier | Current meaning |
| --- | --- |
| Layer1 | Core agents compiled and enabled in the workspace |
| Layer2 | Web Automation, the active built-in domain agent, plus some archived designs |
| Layer3 | User-defined agents (planned, not yet implemented) |

## Clients

### TUI

The most complete and most mature user entry point currently is the TUI.

### WebUI

The Web UI (arona chat) and the management panel (plana) have been migrated to the sister repository [shittim-chest](https://github.com/celestia-island/shittim-chest) and removed from this codebase; the preferred interface for this repository is the TUI.

### CLI

The CLI exists, but some commands are still placeholder output.

### Tauri clients

Desktop and mobile code already lives in the sibling repository [shittim-chest](https://github.com/celestia-island/shittim-chest), but is best regarded as an early integration. IDE integrations (VS Code, IntelliJ) are likewise located in shittim-chest.

## Conservative statement of the security model

- JWT and API key authentication capabilities exist.
- RBAC mappings for known HTTP, WebSocket, and MCP paths exist.
- Encrypted provider key storage capability exists.
- Container hardening and audit integrity are still incomplete.

Unless you have verified the specific code path, do not treat mutual TLS, complete capability tokens, or full-chain strict policy enforcement as current fact.
