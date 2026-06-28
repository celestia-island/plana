# Intégration des protocoles industriels — Evernight

Evernight est le **courtier obligatoire de capacités matérielles** pour
l'écosystème celestia-island. Aucun crate amont n'importe `aoba` / `rust7` /
etc. directement — toutes les E/S physiques transitent par les modules de
protocole d'evernight. Ce guide décrit comment se connecter, scruter, découvrir
et gérer les alarmes sur chaque protocole industriel pris en charge.

## En un coup d'œil

| Protocole | Transport | Statut | Port / Bus | Couverture |
|----------|-----------|--------|------------|----------|
| **Modbus RTU** | Série (RS-485) | ✅ Production | `/dev/ttyUSB*` | ~70 % du marché PLC chinois |
| **Modbus TCP** | TCP | ✅ Production | 502 | SCADA d'usine |
| **S7comm** (Siemens) | TCP | ✅ Production | 102 | S7-1200/1500/300/400 |
| **MC Protocol** (Mitsubishi) | TCP | 🚧 Planifié | 5000 | MELSEC FX/Q/L/iQ-R |
| **OPC UA** | TCP | ⏳ Sondage uniquement | 4840 | « Traducteur universel » |
| **EtherCAT** (Beckhoff) | Ethernet | ⏳ Planifié | — | Servo / motion |
| **EtherNet/IP** (Rockwell) | TCP/UDP | ⏳ Planifié | 44818/2222 | Allen-Bradley |
| **CAN 2.0B** | Série | ⏳ Planifié | `/dev/ttyUSBCAN` | Piles à combustible |

> **Comment lire la colonne Statut** : ✅ = vérifié face à un simulateur dans le
> CI ; 🚧 = conception terminée, mise en œuvre en cours ; ⏳ = sondage /
> connectivité uniquement, ou planifié.

## Architecture

```
                          ┌──────────────────────────────────┐
   Votre application ────►│         evernight crate           │
   (CLI / bibliothèque /  │                                   │
    sensor-poll /         │  ┌─────────────────────────────┐ │
    serveur API)          │  │   ProtocolBackend trait      │ │
                          │  │   ProtocolProbe trait         │ │
                          │  │   ProtocolRegistry           │ │
                          │  │   ┌─────────┐ ┌─────────┐    │ │
                          │  │   │ Modbus  │ │ S7comm  │ …  │ │
                          │  │   │ Backend │ │ Backend │    │ │
                          │  │   └────┬────┘ └────┬────┘    │ │
                          │  └───────┼────────────┼─────────┘ │
                          │          │            │           │
                          │   ┌──────▼──┐  ┌──────▼────┐      │
                          │   │  aoba   │  │  rust7 +  │      │
                          │   │ (Modbus)│  │ snap7-cli │      │
                          │   └─────────┘  └───────────┘      │
                          └──────────────────────────────────┘
                                           │
                               ┌───────────┴───────────┐
                               │   Matériel physique   │
                               │  PLC / capteur / vanne│
                               └───────────────────────┘
```

Chaque protocole implémente les deux mêmes traits, donc en ajouter un nouveau
est un **plug-in** — aucun changement pour les consommateurs amont
(scruteur de capteurs, découverte, CLI) :

```rust
// src/protocol/backend.rs
pub trait ProtocolBackend: Send + Sync {
    fn protocol_name(&self) -> &'static str;
    fn connect(&self, transport: &TransportInfo) -> Result<()>;
    fn read(&self, addr: &DataAddress) -> Result<ProtocolReadResult>;
    fn write(&self, addr: &DataAddress, data: &[u8]) -> Result<ProtocolWriteResult>;
    fn ping(&self) -> Result<bool>;
}

pub trait ProtocolProbe: Send + Sync {
    fn protocol_name(&self) -> &'static str;
    fn probe(&self, transport: &TransportInfo) -> Result<Option<ProtocolProbeResult>>;
    fn confidence(&self) -> f32;
}
```

---

## Modbus (RTU sur série / TCP)

Modbus est le cheval de trait de la communication industrielle. Evernight
encapsule le crate `aoba` derrière `evernight::serial::modbus::ModbusMaster`
et `evernight::protocol::ModbusBackend`.

### Fonctionnalités (feature flags)

```toml
[dependencies]
evernight = { version = "0.1", features = ["serial", "protocol"] }
```

- `serial` — `ModbusMaster`, lecture/écriture de registres, auto-détection de la vitesse (baud)
- `protocol` — `ModbusBackend` (impl. du trait) + `ModbusProbe` (auto-détection TCP)

### Démarrage rapide — lire des registres depuis le code

```rust
use evernight::serial::modbus::{ModbusMaster, RegisterMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let master = ModbusMaster::builder(0x13)        // station 19
        .with_port("/dev/ttyUSB1")
        .with_timeout(5000)
        .open()?;

    let result = master.read_registers(RegisterMode::Holding, 0x10, 3)?;
    println!("Registers: {:?}", result.values);     // [pressure, pressure2, pressure3]
    Ok(())
}
```

### Auto-détection de la vitesse (baud rate)

Si vous ne connaissez pas la vitesse (baud rate) du périphérique, balayez les vitesses courantes :

```rust
use evernight::serial::probe_modbus_rtu_baud;

let detected = probe_modbus_rtu_baud("/dev/ttyUSB1", 0x13)?;
println!("Device responds at {} baud", detected.baud);
```

### Modbus TCP — sondage + auto-détection

```rust
use evernight::protocol::{ProtocolRegistry, ModbusProbe, TransportInfo};
use std::sync::Arc;

let mut registry = ProtocolRegistry::new();
registry.register_probe(Arc::new(ModbusProbe));

let transport = TransportInfo::Tcp { host: "192.168.1.20".into(), port: 502 };
if let Some(result) = registry.auto_detect(&transport, 0.5).await {
    println!("Detected {} (confidence {:.0}%)", result.protocol, result.confidence * 100.0);
}
```

### CLI — sonder un hôte pour Modbus

```bash
# Sondage des ports industriels courants (502=Modbus, 102=S7comm, …)
evernight probe 192.168.1.20 --ports 502,102,4840
```

### Tester sans matériel

Le CI d'Evernight utilise un port série virtuel (`socat`) avec un esclave
Modbus aoba. Lancez les tests d'intégration :

```bash
cargo test --features full --test serial_integration
```

Ils ouvrent une paire TTY virtuelle, exécutent une véritable boucle d'esclave
Modbus, et vérifient lecture/écriture/balayage de baud — aucun matériel physique
requis.

---

## S7comm (Siemens S7-1200/1500/300/400)

S7comm est le protocole natif de Siemens sur ISO-on-TCP (port 102). Evernight
encapsule le crate `rust7` (Rust pur, pas de FFI) pour l'accès aux données et
`snap7-client` pour le téléchargement de blocs / le flashage.

### Fonctionnalités (feature flags)

```toml
[dependencies]
evernight = { version = "0.1", features = ["s7comm", "manifest"] }
```

### Lire un bloc de données (DB)

```rust
use evernight::protocol::s7comm::{S7CommClient, S7ConnectParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = S7CommClient::new(S7ConnectParams {
        host: "192.168.1.10".into(),
        port: 102,
        rack: 0,
        slot: 1,            // S7-1500: slot 1; S7-300: slot 2
    });
    client.connect().await?;

    // Read 4 bytes from DB1 offset 0 (a REAL temperature)
    let bytes = client.read_db(1, 0, 4).await?;
    let temp = f32::from_be_bytes(bytes.try_into().unwrap());
    println!("DB1.DBD0 = {:.1} °C", temp);

    client.disconnect().await;
    Ok(())
}
```

### Découvrir des DB inconnus + sonder la structure

```rust
use evernight::protocol::s7comm::{scan_db_numbers, probe_db_structure};

// Which DB numbers exist on this PLC?
let dbs = scan_db_numbers(&client, 1, 100).await?;
for db in &dbs {
    println!("DB{}: {}", db.db_number, db.status);   // ok / optimized / not_found
}

// Read the first 512 bytes of DB1 for type inference
let report = probe_db_structure(&client, 1, 512).await?;
println!("DB1 raw bytes: {} bytes, readable={}", report.raw_bytes.len(), report.readable);
```

### Téléchargement de blocs + flashage (programmation du PLC)

```rust
use evernight::protocol::s7comm_blocks::{S7BlockOps, flash_block};

let ops = S7BlockOps::new(&client);
let binary_blocks = ops.full_upload_block(1).await?;   // read existing block
// … modify MC7 binary …
ops.download_block(block_type, block_num, &new_binary).await?;

// Stop → download → restart in one call:
let result = flash_block(&client, block_type, block_num, &new_binary).await?;
println!("Flashed, PLC restarted: {:?}", result);
```

### Manifeste S7 — configuration de scrutation déclarative

Plutôt que de coder en dur l'agencement des registres, décrivez toute la
station en TOML :

```toml
# corridor.toml
format_version = "1"

[facility]
id = "hydrogen-plant"
name = "Hydrogen Production Line"

[[connections]]
id = "conn-s7"
kind = "s7comm"
host = "192.168.1.10"
port = 102
rack = 0
slot = 1

[[stations]]
id = 1
connection_ref = "conn-s7"
poll_interval_ms = 1000
device_class = "equipment"
vendor = "siemens"
model = "S7-1500"

# S7 data blocks to poll
[[stations.s7_data_blocks]]
db_number = 1
start_offset = 0
length = 16

[[stations.s7_data_blocks.fields]]
offset = 0.0
name = "temperature"
data_type = "REAL"
scale = { kind = "linear", factor = 1.0, offset = 0.0, unit = "Celsius" }

[[stations.s7_data_blocks.fields.alarm]]
h = 60.0
hh = 80.0

[[stations.s7_data_blocks.fields]]
offset = 4.0
name = "pressure"
data_type = "REAL"
scale = { kind = "linear", factor = 1.0, offset = 0.0, unit = "MPa" }

[[alarm_rules]]
id = "1.temperature.hh"
station_ref = 1
register_name = "temperature"
level = "HighHigh"
threshold = 80.0
unit = "°C"
```

Sonnez-le avec une seule commande (les stations S7 sont scrutées par le
`S7SensorPoller` parallèle, les règles d'alarmme sont auto-étiquetées avec
`protocol = "s7comm"`) :

```bash
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### Types de données de champ S7

| Type | Taille | Format d'offset | Décodage |
|------|--------|-----------------|----------|
| `BOOL` | 1 bit | `8.0` (octet 8, bit 0) | test de bit |
| `BYTE` | 1 octet | `8` | `u8` |
| `WORD` | 2 octets | `8` | `u16::from_be_bytes` |
| `INT` | 2 octets | `8` | `i16::from_be_bytes` |
| `DWORD` | 4 octets | `8` | `u32::from_be_bytes` |
| `DINT` | 4 octets | `8` | `i32::from_be_bytes` |
| `REAL` | 4 octets | `0` | `f32::from_be_bytes` |
| `STRING` | var | `20` | ASCII à préfixe de longueur |

> La partie fractionnaire de `offset` encode l'**index de bit** pour les champs
> BOOL. Par ex. `10.3` = octet 10, bit 3.

### Tester sans matériel

Le CI exécute un simulateur `snap7-server` (émulateur S7-1500 pur Rust) et
vérifie connexion → lecture DB → scan des DB → sondage de structure →
opérations sur les blocs :

```bash
cargo test --features full --test s7comm_integration
```

12 tests réussissent face au PLC simulé — il s'agit d'un véritable échange de
protocole de bout en bout, pas d'un simulacre.

> **⚠️ Prérequis TIA Portal** : les PLC S7-1200/1500 nécessitent l'activation
> de **« Permit access with PUT/GET »** dans la configuration matérielle, et les
> DB doivent utiliser l'**accès aux blocs non optimisé** (clic droit sur le DB →
> Propriétés → décocher « Optimized block access »). Les DB optimisés renvoient
> une erreur lors des lectures au niveau des octets.

---

## OPC UA (sondage de connectivité)

Evernight peut détecter les points de terminaison OPC UA (port TCP 4840) mais
ne fournit pas encore de client complet (les fonctionnalités client du crate
Rust `opcua` sont incomplètes). Utilisez le sondage pour découvrir les points
de terminaison, puis un client OPC UA dédié jusqu'à la livraison du backend.

```bash
evernight probe 192.168.1.50 --ports 4840
```

---

## Découverte — identification autonome du protocole

Pointez evernight vers une plage réseau inconnue ou un port série et il
identifie le protocole automatiquement :

```
Point de terminaison TCP       Bus série
      │                         │
      ▼                         ▼
  Chaîne ProtocolProbe       balayage baud + scan des stations
  (trié par priorité)        (probe_modbus_rtu_baud)
      │                         │
      ▼                         ▼
  ProtocolProbeResult       { port, baud, protocol, stations[] }
  { protocol, confidence,
    banner }
```

Les sondes sont essayées par ordre de priorité (Modbus=40 avant S7comm=50) ; la
première correspondance au-dessus du seuil de confiance (par défaut 0.5)
l'emporte.

---

## Routage des alarmes (Modbus + S7comm)

Les lectures de capteurs transitent par un pipeline d'alarmme partagé. Chaque
protocole obtient son propre espace de noms de sujet (topic) afin que les
consommateurs en aval (entelecheia, shittim-chest) puissent filtrer par
source :

| Protocole | Sujet de déclenchement | Identifiant source |
|----------|---------------|-----------|
| Modbus | `modbus.{station}.{field}.{level}` | `evernight.modbus.{station}` |
| S7comm | `s7comm.{station}.{field}.{level}` | `evernight.s7comm.{station}` |

Les niveaux d'alarmme suivent la norme ISA-18.2 : `ll` / `l` / `h` / `hh` /
`roc`, avec une bande morte d'hystérésis et un comptage d'anti-rebond pour
éviter le clignotement.

```rust
use evernight::sensor::{AlarmConfig, AlarmRule, AlarmLevel};

let alarm = AlarmConfig::new()
    .with_rule(
        AlarmRule::new("1.temp.hh", 1, "temperature", AlarmLevel::HH, 80.0)
            .with_protocol("s7comm")    // → topic "s7comm.1.temperature.hh"
            .with_hysteresis(2.0),
    );
```

---

## Référence des commandes CLI

| Commande | Description |
|---------|-------------|
| `evernight probe <host> [--ports 502,102,...]` | Sonder un hôte pour des protocoles industriels |
| `evernight sensor-poll [--manifest X.toml]` | Scruter des registres de capteurs, émettre des alarmes vers entelecheia |
| `evernight file cp <local> <user@host:path>` | Téléverser un fichier via SSH |
| `evernight file get <user@host:path> <local>` | Télécharger un fichier via SSH |
| `evernight file ls <user@host:path>` | Lister un répertoire distant |
| `evernight proxy <port> --host <jump>` | Proxy SOCKS5 local via redirection dynamique SSH |
| `evernight exec --host X --command "..."` | Exécuter une commande SSH unique |
| `evernight hw` | Afficher la télémétrie matérielle locale |
| `evernight api-serve --transport ws` | Démarrer le serveur d'API JSON-RPC |

---

## Feuille de route

- **MC Protocol** (Mitsubishi) — codec de trame binaire implémenté à la main,
  aucun crate Rust n'existe encore. Ajoute ~7 % de couverture du marché PLC.
- **EtherCAT** (Beckhoff / Inovance) — via le crate `ethercrab`.
- **EtherNet/IP + CIP** (Rockwell) — adressage classe/instance/attribut.
- **Client/serveur OPC UA** — en attente de la maturité du crate `opcua`.
- **CAN 2.0B** — pont USB-CAN d'analyse des piles à combustible.
