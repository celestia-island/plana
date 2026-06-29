+++
title = "Getting Started — Evernight"
description = """Getting started with evernight — build, run, and first commands."""
lang = "en"
category = "guides"
subcategory = "router"
+++

# Getting Started — Evernight

Evernight (长夜月) is a cross-platform remote control library and daemon written in Rust. It bundles screen capture, WebRTC streaming, SSH remote shell, remote terminal access, file transfer, hardware telemetry, industrial protocol support (Modbus, S7comm, OPC-UA probing), and NAT traversal into a single reusable crate and standalone CLI binary.

## Prerequisites

- Rust 1.85 or later (2024 edition)
- A C compiler for your platform (MSVC on Windows, GCC/Clang on Linux/macOS)
- For hardware telemetry: `nvidia-smi` (NVIDIA GPU), `libudev` on Linux
- For industrial protocols: a serial port (`/dev/ttyUSB*`) or network access to a PLC

## Build

```bash
git clone https://github.com/celestia-island/evernight.git
cd evernight
cargo build --release
```

The main binary lands at `target/release/evernight`.

## Quick Start

The CLI uses subcommands. Run `evernight --help` to see them all.

### SSH — run a remote command

```bash
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"
```

### SSH — file operations

```bash
# Upload a local file to a remote host
evernight file cp ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# Download a remote file
evernight file get root@192.168.1.100:/var/log/syslog ./syslog

# List a remote directory
evernight file ls root@192.168.1.100:/etc/
```

### SSH — SOCKS5 proxy

```bash
# Start a local SOCKS5 proxy (port 1080) tunneled through an SSH jump host
evernight proxy 1080 --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519
```

### Hardware telemetry

```bash
evernight hw
```

### Network protocol probing

```bash
# Probe common industrial ports on a host
evernight probe 192.168.1.20 --ports 502,102,4840,22
```

### Industrial sensor polling

```bash
# Poll sensors from a hardware manifest and emit alarms to entelecheia
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### NAT type detection

```bash
evernight nat
```

### API server (JSON-RPC over WebSocket)

```bash
evernight api-serve --transport ws --host 0.0.0.0 --port 50000
```

## Feature Flags

Evernight is feature-gated so you only compile what you need:

```toml
[dependencies]
# Minimal: SSH + hardware telemetry
evernight = { version = "0.1", features = ["remote-ssh", "hardware"] }

# Industrial: Modbus + S7comm + manifest support
evernight = { version = "0.1", features = ["serial", "s7comm", "manifest"] }

# Everything (default)
evernight = { version = "0.1", features = ["full"] }
```

| Feature | Enables |
|---------|---------|
| `remote-ssh` | SSH exec, file transfer, terminal, port-forward, SOCKS5 proxy |
| `remote-vnc` | VNC (RFB) client |
| `remote-rdp` | RDP transport skeleton (TPKT/COTP/MCS) |
| `serial` | Serial port + Modbus RTU (via aoba) |
| `s7comm` | S7comm client + block download/flash (via rust7 + snap7-client) |
| `protocol` | Protocol probing + ProtocolBackend trait abstraction |
| `sensor` | Sensor poll loop, alarm evaluation, time-series storage |
| `manifest` | Hardware manifest TOML/JSON schema + runtime converters |
| `container` | Docker / Podman container management |
| `hardware` | CPU/GPU/memory/storage telemetry |
| `screen` | Screen capture + JPEG/VP9 encoding |
| `webrtc` | WebRTC screen streaming |
| `tunnel` | TCP port forwarding + NAT traversal |
| `api` | JSON-RPC 2.0 API server (ws/wss/ipc) |

## Key Features

- **Screen capture** — Enumerate displays, capture raw RGBA frames
- **WebRTC streaming** — JPEG over DataChannel or VP9 video track; ICE/STUN support
- **SSH remote shell** — Execute commands, transfer files, open terminals via `russh`
- **File transfer** — Upload/download with progress callbacks over SSH
- **Hardware telemetry** — CPU, GPU, memory, storage, PCI devices
- **Industrial protocols** — Modbus RTU/TCP, S7comm (Siemens), OPC-UA probing
- **Sensor polling** — Declarative manifest-driven poll loop with ISA-18.2 alarm routing
- **TCP tunneling** — Local/remote port forwarding + SOCKS5 dynamic forward
- **NAT discovery** — STUN-based NAT type detection
- **API server** — JSON-RPC 2.0 over WebSocket / IPC for web frontends

## Next Steps

- Read the **[Industrial Protocol Integration Guide](./protocols.md)** for Modbus/S7comm usage
- See `evernight <command> --help` for per-command options
- Check `cargo doc --open` for the full API reference
- Run the integration tests to verify your setup (no hardware needed):
```bash
  cargo test --features full --test s7comm_integration    # S7comm vs snap7-server
  cargo test --features full --test serial_integration    # Modbus vs virtual serial
```
