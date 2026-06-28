+++
title = "Architecture d'Evernight"
description = """evernight — bibliothèque et démon de contrôle à distance multiplateforme : carte des modules, couche protocole, modèle de connexion."""
lang = "fr"
category = "architecture"
subcategory = "router"
+++

# Architecture d'Evernight

> **evernight** est une bibliothèque et un démon de contrôle à distance
> multiplateforme. C'est le courtier obligatoire de capacités matérielles/
> protocole de l'écosystème celestia-island — aucun crate amont ne parle
> directement aux appareils physiques.

## En un coup d'œil

| Capacité | Module | Feature |
|---|---|---|
| Capture d'écran (X11/DXGI/CoreGraphics) | `screen` | `screen` |
| Streaming d'écran WebRTC | `stream` | `webrtc` |
| Shell distant SSH + SFTP | `remote` | `remote-ssh` |
| Client VNC (RFB) | `vnc` | `remote-vnc` |
| Client RDP | `rdp` | `remote-rdp` |
| Télémétrie matérielle | `hardware` | `hardware` |
| Protocoles industriels | `protocol` | `protocol` / `s7comm` / `opcua` / `ethercat` |
| Série / Modbus | `serial`, `sensor` | `serial` |
| Tunnels TCP + traversée NAT | `tunnel` | `tunnel` / `upnp` |
| Catalogue de connexions (URI) | `connection`, `connection_chain` | noyau |
| Coffre d'identifiants chiffré | `vault` | `vault` |
| Conteneurs / K8s / libvirt | `container`, `vm_manager` | `container` / `k8s` / `libvirt` / `vm` |
| Serveur API (JSON-RPC) | `api` | `api` |

## Trois traits backend polymorphes

- `TerminalBackend` — read/write/resize pour terminaux texte (SSH, série, Docker)
- `ViewportBackend` — rendu/entrée pour bureau graphique (VNC, RDP, écran local)
- `FileBackend` — list/get/put/rm pour fichiers (SFTP, shell, FS local)

Ajouter un transport est un plug-in — les consommateurs ne changent pas.

## Couche protocole

Les E/S industrielles sont courtées via deux traits :

- `ProtocolBackend` — connect / read / write / ping
- `ProtocolProbe` — détecte automatiquement le protocole d'un endpoint inconnu

```
ProtocolRegistry::auto_detect(transport)  →  ProtocolProbeResult
```

Backends : Modbus (aoba), S7comm (rust7 + snap7-client), MC Protocol, EtherCAT
(ethercrab), EtherNet/IP + CIP, OPC UA (crate opcua, client + serveur), CAN
(SocketCAN).

### Auto-organisation S7 (auto-provision)

Donnez à evernight une simple IP, il s'auto-organise :

```rust
use evernight::protocol::auto_provision;
let profile = auto_provision("192.168.1.10").await?;
```

Le pipeline probe → connect → scan-DB → structure-probe renvoie un
`S7DeviceProfile` sans saisie manuelle de symboles. Voir le
[guide de préparation TIA Portal](../../guides/router/tia-portal-setup.md)
pour la préparation unique du PLC.

## Modèle de connexion

Les connexions sont typées par URI et gérées par catalogue :

```
ssh://user@host:22          s7://10.0.0.5?rack=0&slot=1
vnc://host:5900             opcua://10.0.0.5:4840
serial:///dev/ttyUSB0?baud=9600
```

`connection_chain` résout une cible en une chaîne de sauts ordonnée (ProxyJump
généralisé) pour le tunnellisation.

## Feature flags

`full` (par défaut) active tout. Chaque capacité est gateable indépendamment
pour des builds à dépendances minimales — ex. `--features s7comm,serial` ne
livre que le sous-ensemble protocoles industriels.
