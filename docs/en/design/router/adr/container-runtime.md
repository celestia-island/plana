+++
title = "Container Runtime Client (Docker/Podman)"
description = """Architecture decision record — Container Runtime Client (Docker/Podman)."""
lang = "en"
category = "design"
subcategory = "router"
+++

# Container Runtime Client (Docker/Podman)

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Managing containers (Docker, Podman) is a key use case for a universal connection manager. Operations include listing containers, exec shell, viewing logs, and port forwarding.

## Decision

Use the Docker Engine API via Unix socket (or named pipe on Windows). Podman's Docker-compatible API is supported via the same code path. Define typed container models (`ContainerInfo`, `ContainerState`, `ContainerPort`). Future: implement `TerminalBackend` over Docker exec attach for interactive container shells.

## Consequences

### Positive

- Docker API is well-documented and stable; Podman compatibility is free.

### Negative

- Requires Docker/Podman daemon to be running.
- API version differences between Docker versions.
