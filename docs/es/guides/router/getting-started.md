+++
title = "Primeros pasos — Evernight"
description = """Introducción a evernight — compilación, ejecución y primeros comandos."""
lang = "es"
category = "guides"
subcategory = "router"
+++

# Primeros pasos — Evernight

Evernight (长夜月) es una biblioteca y demonio de control remoto multiplataforma escrito en Rust. Integra captura de pantalla, streaming por WebRTC, shell remoto SSH, acceso a terminal remota, transferencia de archivos, telemetría de hardware, soporte de protocolos industriales (sondeo de Modbus, S7comm y OPC-UA) y cruce de NAT en un único crate reutilizable y binario CLI autónomo.

## Requisitos previos

- Rust 1.85 o superior (edición 2024)
- Un compilador de C para tu plataforma (MSVC en Windows, GCC/Clang en Linux/macOS)
- Para telemetría de hardware: `nvidia-smi` (GPU NVIDIA), `libudev` en Linux
- Para protocolos industriales: un puerto serie (`/dev/ttyUSB*`) o acceso de red a un PLC

## Compilación

```bash
git clone https://github.com/celestia-island/evernight.git
cd evernight
cargo build --release
```

El binario principal se encuentra en `target/release/evernight`.

## Inicio rápido

La CLI usa subcomandos. Ejecuta `evernight --help` para verlos todos.

### SSH — ejecutar un comando remoto

```bash
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"
```

### SSH — operaciones con archivos

```bash
# Subir un archivo local a un host remoto
evernight file cp ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# Descargar un archivo remoto
evernight file get root@192.168.1.100:/var/log/syslog ./syslog

# Listar un directorio remoto
evernight file ls root@192.168.1.100:/etc/
```

### SSH — proxy SOCKS5

```bash
# Iniciar un proxy SOCKS5 local (puerto 1080) tunelizado a través de un host de salto SSH
evernight proxy 1080 --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519
```

### Telemetría de hardware

```bash
evernight hw
```

### Sondeo de protocolos de red

```bash
# Sondear puertos industriales comunes en un host
evernight probe 192.168.1.20 --ports 502,102,4840,22
```

### Sondeo de sensores industriales

```bash
# Consultar sensores a partir de un manifiesto de hardware y emitir alarmas a entelecheia
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### Detección del tipo de NAT

```bash
evernight nat
```

### Servidor API (JSON-RPC sobre WebSocket)

```bash
evernight api-serve --transport ws --host 0.0.0.0 --port 50000
```

## Marcas de características

Evernight usa puertas de características (feature flags) para que solo compiles lo que necesitas:

```toml
[dependencies]
# Mínimo: SSH + telemetría de hardware
evernight = { version = "0.1", features = ["remote-ssh", "hardware"] }

# Industrial: Modbus + S7comm + soporte de manifiesto
evernight = { version = "0.1", features = ["serial", "s7comm", "manifest"] }

# Todo (por defecto)
evernight = { version = "0.1", features = ["full"] }
```

| Característica | Habilita |
|---------|---------|
| `remote-ssh` | exec SSH, transferencia de archivos, terminal, reenvío de puertos, proxy SOCKS5 |
| `remote-vnc` | cliente VNC (RFB) |
| `remote-rdp` | esqueleto de transporte RDP (TPKT/COTP/MCS) |
| `serial` | puerto serie + Modbus RTU (vía aoba) |
| `s7comm` | cliente S7comm + descarga/flash de bloques (vía rust7 + snap7-client) |
| `protocol` | sondeo de protocolos + abstracción del trait ProtocolBackend |
| `sensor` | bucle de consulta de sensores, evaluación de alarmas, almacenamiento de series temporales |
| `manifest` | esquema TOML/JSON de manifiesto de hardware + conversores en tiempo de ejecución |
| `container` | gestión de contenedores Docker / Podman |
| `hardware` | telemetría de CPU/GPU/memoria/almacenamiento |
| `screen` | captura de pantalla + codificación JPEG/VP9 |
| `webrtc` | streaming de pantalla por WebRTC |
| `tunnel` | reenvío de puertos TCP + cruce de NAT |
| `api` | servidor API JSON-RPC 2.0 (ws/wss/ipc) |

## Características principales

- **Captura de pantalla** — Enumerar pantallas, capturar fotogramas RGBA en bruto
- **Streaming WebRTC** — JPEG sobre DataChannel o pista de vídeo VP9; soporte ICE/STUN
- **Shell remoto SSH** — Ejecutar comandos, transferir archivos, abrir terminales vía `russh`
- **Transferencia de archivos** — Subida/bajada con callbacks de progreso sobre SSH
- **Telemetría de hardware** — CPU, GPU, memoria, almacenamiento, dispositivos PCI
- **Protocolos industriales** — Modbus RTU/TCP, S7comm (Siemens), sondeo OPC-UA
- **Consulta de sensores** — Bucle de consulta declarativo basado en manifiesto con enrutado de alarmas ISA-18.2
- **Túneles TCP** — Reenvío de puertos local/remoto + reenvío dinámico SOCKS5
- **Descubrimiento de NAT** — Detección del tipo de NAT basada en STUN
- **Servidor API** — JSON-RPC 2.0 sobre WebSocket / IPC para frontales web

## Siguientes pasos

- Lee la **[Guía de integración de protocolos industriales](./protocols.md)** para el uso de Modbus/S7comm
- Consulta `evernight <comando> --help` para las opciones de cada comando
- Revisa `cargo doc --open` para la referencia completa de la API
- Ejecuta las pruebas de integración para verificar tu configuración (no se necesita hardware):
```bash
  cargo test --features full --test s7comm_integration    # S7comm vs snap7-server
  cargo test --features full --test serial_integration    # Modbus vs puerto serie virtual
```
