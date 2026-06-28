# Feature Flag Architecture

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight's monolithic dependency graph forces all consumers to pull in every dependency (webrtc, russh, screenshots, sysinfo) even if they only need SSH or hardware telemetry. This increases compile times and binary sizes for downstream users.

## Decision

Use Cargo feature flags to split the crate into the following features:

| Feature      | Gates                                        |
|--------------|----------------------------------------------|
| `screen`     | Screen capture module + `screenshots` crate  |
| `webrtc`     | WebRTC module + `webrtc` crate (implies `screen`) |
| `remote-ssh` | SSH handler module + `russh` crate           |
| `hardware`   | Hardware telemetry module + `sysinfo` crate  |
| `protocol`   | Protocol/message types (no heavy deps)       |
| `tunnel`     | TCP tunnel module                            |
| `full`       | All features (default)                       |

Each feature gates both the module and its dependencies. The `webrtc` feature implies `screen` since WebRTC sessions require screen capture.

## Consequences

### Positive

- Consumers only compile what they need
- Compile time reduced for partial usage

### Negative

- Feature flag matrix grows; must test each feature combination in CI
