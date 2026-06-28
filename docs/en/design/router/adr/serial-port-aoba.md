# Serial Port Communication via aoba

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight needs serial port support for embedded device management and industrial protocol (Modbus RTU) probing. The sibling crate `aoba` already provides cross-platform serial port enumeration, VID/PID/serial extraction, and Modbus RTU/TCP master functionality.

## Decision

Delegate all serial port operations to aoba. Evernight's `serial` module defines types (`SerialConfig`, `SerialPortInfo`) and implements `TerminalBackend` over a serial transport, calling aoba for the actual port I/O. Protocol autodetection (Modbus RTU baud/parity sweep) also delegates to aoba.

## Consequences

### Positive

- Reuses proven code; aoba handles cross-platform edge cases.

### Negative

- Adds a dependency on aoba; serial feature requires aoba to be available.
