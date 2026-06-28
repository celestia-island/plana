+++
title = "Arquitectura de Evernight"
description = """evernight — biblioteca y demonio de control remoto multiplataforma: mapa de módulos, capa de protocolo, modelo de conexión."""
lang = "es"
category = "architecture"
subcategory = "router"
+++

# Arquitectura de Evernight

> **evernight** es una biblioteca y demonio de control remoto multiplataforma.
> Es el bróker obligatorio de capacidades de hardware/protocolo del ecosistema
> celestia-island — ningún crate ascendente habla directamente con dispositivos
> físicos.

## De un vistazo

| Capacidad | Módulo | Feature |
|---|---|---|
| Captura de pantalla (X11/DXGI/CoreGraphics) | `screen` | `screen` |
| Streaming de pantalla WebRTC | `stream` | `webrtc` |
| Shell remoto SSH + SFTP | `remote` | `remote-ssh` |
| Cliente VNC (RFB) | `vnc` | `remote-vnc` |
| Cliente RDP | `rdp` | `remote-rdp` |
| Telemetría de hardware | `hardware` | `hardware` |
| Protocolos industriales | `protocol` | `protocol` / `s7comm` / `opcua` / `ethercat` |
| Serial / Modbus | `serial`, `sensor` | `serial` |
| Túneles TCP + cruce NAT | `tunnel` | `tunnel` / `upnp` |
| Catálogo de conexiones (URI) | `connection`, `connection_chain` | núcleo |
| Cofre de credenciales cifrado | `vault` | `vault` |
| Contenedores / K8s / libvirt | `container`, `vm_manager` | `container` / `k8s` / `libvirt` / `vm` |
| Servidor API (JSON-RPC) | `api` | `api` |

## Tres traits de backend polimórficos

- `TerminalBackend` — read/write/resize para terminales de texto (SSH, serial, Docker)
- `ViewportBackend` — render/input para escritorio gráfico (VNC, RDP, pantalla local)
- `FileBackend` — list/get/put/rm para archivos (SFTP, shell, FS local)

Añadir un transporte es un plug-in — los consumidores no cambian.

## Capa de protocolo

Las E/S industriales se gestionan mediante dos traits:

- `ProtocolBackend` — connect / read / write / ping
- `ProtocolProbe` — detecta automáticamente el protocolo de un endpoint desconocido

```
ProtocolRegistry::auto_detect(transport)  →  ProtocolProbeResult
```

Backends: Modbus (aoba), S7comm (rust7 + snap7-client), MC Protocol, EtherCAT
(ethercrab), EtherNet/IP + CIP, OPC UA (crate opcua, cliente + servidor), CAN
(SocketCAN).

### Autoorganización S7 (auto-provision)

Dé a evernight una IP simple y se autoorganiza:

```rust
use evernight::protocol::auto_provision;
let profile = auto_provision("192.168.1.10").await?;
```

El pipeline probe → connect → scan-DB → structure-probe devuelve un
`S7DeviceProfile` sin entrada manual de símbolos. Vea la
[guía de preparación de TIA Portal](../../guides/router/tia-portal-setup.md)
para la preparación única del PLC.

## Modelo de conexión

Las conexiones son tipadas por URI y gestionadas por catálogo:

```
ssh://user@host:22          s7://10.0.0.5?rack=0&slot=1
vnc://host:5900             opcua://10.0.0.5:4840
serial:///dev/ttyUSB0?baud=9600
```

`connection_chain` resuelve un objetivo en una cadena de saltos ordenada
(ProxyJump generalizado) para túneles.

## Feature flags

`full` (predeterminado) activa todo. Cada capacidad es gateable de forma
independiente para builds de dependencias mínimas — p. ej.
`--features s7comm,serial` entrega solo el subconjunto de protocolos
industriales.
