+++
title = "Pour commencer — Evernight"
description = """Démarrage avec evernight — compilation, exécution et premières commandes."""
lang = "fr"
category = "guides"
subcategory = "router"
+++

# Pour commencer — Evernight

Evernight (长夜月) est une bibliothèque et un démon de contrôle à distance multiplateforme écrits en Rust. Il regroupe la capture d'écran, le streaming WebRTC, le shell distant SSH, l'accès à un terminal distant, le transfert de fichiers, la télémétrie matérielle, la prise en charge des protocoles industriels (sondage Modbus, S7comm, OPC-UA) et la traversée NAT au sein d'un unique crate réutilisable et d'un binaire CLI autonome.

## Prérequis

- Rust 1.85 ou version ultérieure (édition 2024)
- Un compilateur C pour votre plateforme (MSVC sous Windows, GCC/Clang sous Linux/macOS)
- Pour la télémétrie matérielle : `nvidia-smi` (GPU NVIDIA), `libudev` sous Linux
- Pour les protocoles industriels : un port série (`/dev/ttyUSB*`) ou un accès réseau à un API

## Compilation

```bash
git clone https://github.com/celestia-island/evernight.git
cd evernight
cargo build --release
```

Le binaire principal se trouve dans `target/release/evernight`.

## Démarrage rapide

L'interface en ligne de commande utilise des sous-commandes. Exécutez `evernight --help` pour toutes les afficher.

### SSH — exécuter une commande distante

```bash
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"
```

### SSH — opérations sur fichiers

```bash
# Upload a local file to a remote host
evernight file cp ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# Download a remote file
evernight file get root@192.168.1.100:/var/log/syslog ./syslog

# List a remote directory
evernight file ls root@192.168.1.100:/etc/
```

### SSH — proxy SOCKS5

```bash
# Start a local SOCKS5 proxy (port 1080) tunneled through an SSH jump host
evernight proxy 1080 --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519
```

### Télémétrie matérielle

```bash
evernight hw
```

### Sondage de protocoles réseau

```bash
# Probe common industrial ports on a host
evernight probe 192.168.1.20 --ports 502,102,4840,22
```

### Interrogation de capteurs industriels

```bash
# Poll sensors from a hardware manifest and emit alarms to entelecheia
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### Détection du type de NAT

```bash
evernight nat
```

### Serveur API (JSON-RPC sur WebSocket)

```bash
evernight api-serve --transport ws --host 0.0.0.0 --port 50000
```

## Feature flags

Evernight est conditionné par des feature flags afin que vous ne compiliez que ce dont vous avez besoin :

```toml
[dependencies]
# Minimal: SSH + hardware telemetry
evernight = { version = "0.1", features = ["remote-ssh", "hardware"] }

# Industrial: Modbus + S7comm + manifest support
evernight = { version = "0.1", features = ["serial", "s7comm", "manifest"] }

# Everything (default)
evernight = { version = "0.1", features = ["full"] }
```

| Feature | Active |
|---------|---------|
| `remote-ssh` | exec SSH, transfert de fichiers, terminal, transfert de port, proxy SOCKS5 |
| `remote-vnc` | client VNC (RFB) |
| `remote-rdp` | squelette de transport RDP (TPKT/COTP/MCS) |
| `serial` | port série + Modbus RTU (via aoba) |
| `s7comm` | client S7comm + téléchargement de bloc/flash (via rust7 + snap7-client) |
| `protocol` | sondage de protocoles + abstraction du trait ProtocolBackend |
| `sensor` | boucle d'interrogation des capteurs, évaluation d'alarmes, stockage de séries temporelles |
| `manifest` | schéma de manifeste matériel TOML/JSON + convertisseurs à l'exécution |
| `container` | gestion de conteneurs Docker / Podman |
| `hardware` | télémétrie CPU/GPU/mémoire/stockage |
| `screen` | capture d'écran + encodage JPEG/VP9 |
| `webrtc` | streaming d'écran WebRTC |
| `tunnel` | transfert de port TCP + traversée NAT |
| `api` | serveur API JSON-RPC 2.0 (ws/wss/ipc) |

## Fonctionnalités principales

- **Capture d'écran** — Énumérer les écrans, capturer des images RGBA brutes
- **Streaming WebRTC** — JPEG sur DataChannel ou piste vidéo VP9 ; prise en charge ICE/STUN
- **Shell distant SSH** — Exécuter des commandes, transférer des fichiers, ouvrir des terminaux via `russh`
- **Transfert de fichiers** — Envoi/réception avec rappels de progression sur SSH
- **Télémétrie matérielle** — CPU, GPU, mémoire, stockage, périphériques PCI
- **Protocoles industriels** — Modbus RTU/TCP, S7comm (Siemens), sondage OPC-UA
- **Interrogation des capteurs** — Boucle d'interrogation déclarative pilotée par manifeste avec routage d'alarmes ISA-18.2
- **Tunneling TCP** — Transfert de port local/distant + transfert dynamique SOCKS5
- **Découverte NAT** — Détection du type de NAT basée sur STUN
- **Serveur API** — JSON-RPC 2.0 sur WebSocket / IPC pour les frontends web

## Prochaines étapes

- Lisez le **[Guide d'intégration des protocoles industriels](./protocols.md)** pour l'utilisation de Modbus/S7comm
- Consultez `evernight <command> --help` pour les options propres à chaque commande
- Vérifiez `cargo doc --open` pour la référence complète de l'API
- Exécutez les tests d'intégration pour vérifier votre configuration (aucun matériel requis) :
```bash
  cargo test --features full --test s7comm_integration    # S7comm vs snap7-server
  cargo test --features full --test serial_integration    # Modbus vs virtual serial
```
