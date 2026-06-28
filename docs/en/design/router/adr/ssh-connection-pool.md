# SSH Connection Pool

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Original code opened a new SSH connection for every operation (list files, execute command). This is expensive (TCP + key exchange + auth per call) and doesn't scale for interactive use with multiple concurrent operations.

## Decision

Implement `SshConnectionPool` keyed by `(host, port, username)`. Connections are lazily established on first request and reused across operations. `PooledSshClient` wraps a shared `Arc<Mutex<SshSession>>`. Periodic health checks remove dead connections.

## Consequences

### Positive

- Dramatic latency reduction for repeated operations.
- Enables concurrent shell + file + terminal sessions over one connection.

### Negative

- Pool must handle connection expiry and reconnection.
- Long-lived connections may be killed by firewalls.
