+++
title = "Evernight Architecture"
description = """evernight — cross-platform remote control library and daemon: module map, protocol layer, connection model."""
lang = "en"
category = "architecture"
subcategory = "router"
+++

# Evernight Architecture

> **evernight** is a cross-platform remote-control library and daemon. It is the
> mandatory hardware/protocol capability broker for the celestia-island
> ecosystem — no upstream crate talks to physical devices directly.

## At a glance

| Capability | Module | Feature |
|---|---|---|
| Screen capture (X11/DXGI/CoreGraphics) | `screen` | `screen` |
| WebRTC screen streaming | `stream` | `webrtc` |
| SSH remote shell + SFTP | `remote` | `remote-ssh` |
| VNC (RFB) client | `vnc` | `remote-vnc` |
| RDP client | `rdp` | `remote-rdp` |
| Hardware telemetry | `hardware` | `hardware` |
| Industrial protocols | `protocol` | `protocol` / `s7comm` / `opcua` / `ethercat` |
| Serial / Modbus | `serial`, `sensor` | `serial` |
| TCP tunneling + NAT traversal | `tunnel` | `tunnel` / `upnp` |
| Connection catalog (URI) | `connection`, `connection_chain` | core |
| Encrypted credential vault | `vault` | `vault` |
| Container / K8s / libvirt | `container`, `vm_manager` | `container` / `k8s` / `libvirt` / `vm` |
| API server (JSON-RPC) | `api` | `api` |

## Three polymorphic backend traits

Everything above is abstracted behind three transport-agnostic traits:

- `TerminalBackend` — read/write/resize for text terminals (SSH, serial, Docker)
- `ViewportBackend` — render/input for graphical desktop (VNC, RDP, local screen)
- `FileBackend` — list/get/put/rm for file operations (SFTP, shell, local FS)

Adding a transport is a plug-in — consumers don't change.

## Protocol layer

Industrial I/O is brokered through two traits:

- `ProtocolBackend` — connect / read / write / ping
- `ProtocolProbe` — auto-detect the protocol of an unknown endpoint

```
ProtocolRegistry::auto_detect(transport)  →  ProtocolProbeResult
```

Backends: Modbus (aoba), S7comm (rust7 + snap7-client), MC Protocol, EtherCAT
(ethercrab), EtherNet/IP + CIP, OPC UA (opcua crate, client + server), CAN
(SocketCAN).

### S7 self-networking (auto-provision)

Point evernight at a bare IP and it self-networks:

```rust
use evernight::protocol::auto_provision;
let profile = auto_provision("192.168.1.10").await?;
```

The probe → connect → scan-DB → structure-probe pipeline returns an
`S7DeviceProfile` with zero manual symbol entry. See the
[TIA Portal setup guide](../../guides/router/tia-portal-setup.md) for the
one-time PLC preparation.

## Connection model

Connections are URI-typed and catalog-managed:

```
ssh://user@host:22          s7://10.0.0.5?rack=0&slot=1
vnc://host:5900             opcua://10.0.0.5:4840
serial:///dev/ttyUSB0?baud=9600
```

`connection_chain` resolves a target into an ordered hop chain (a generalized
ProxyJump) for tunnelling.

## Feature flags

`full` (default) enables everything. Each capability is independently gateable
for minimal-dependency builds — e.g. `--features s7comm,serial` ships only the
industrial-protocol subset.
