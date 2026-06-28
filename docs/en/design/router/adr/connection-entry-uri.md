# Connection Entry URI Scheme

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

The connection catalog needs a uniform way to represent different protocol connections (SSH, VNC, RDP, serial, Docker). A URI scheme provides human-readable, serializable descriptors.

## Decision

Use protocol-specific URI schemes:

- `ssh://user@host:port`
- `vnc://host:5900`
- `rdp://host:3389`
- `serial:///dev/ttyUSB0?baud=9600`
- `docker:///var/run/docker.sock?container=name`

`ConnectionEntry` parses URIs into typed structs with scheme, host, port, username, path, and query params. The catalog is a tree of `ConnectionCategory` nodes holding entries.

## Consequences

### Positive

- Familiar URI format; easily serializable; supports copy-paste sharing of connections.

### Negative

- Some protocols don't map cleanly to URIs (e.g., Kubernetes context).
- Query string params are unstructured.
