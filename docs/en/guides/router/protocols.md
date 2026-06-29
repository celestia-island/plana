+++
title = "Industrial Protocol Integration — Evernight"
description = """Industrial protocol integration guide for evernight (Modbus, S7comm, OPC UA)."""
lang = "en"
category = "guides"
subcategory = "router"
+++

# Industrial Protocol Integration — Evernight

Evernight is the **mandatory hardware capability broker** for the celestia-island
ecosystem. No upstream crate imports `aoba` / `rust7` / etc. directly — all
physical I/O routes through evernight's protocol modules. This guide covers how
to connect, poll, discover, and alarm on each supported industrial protocol.

## At a Glance

| Protocol | Transport | Status | Port / Bus | Coverage |
|----------|-----------|--------|------------|----------|
| **Modbus RTU** | Serial (RS-485) | ✅ Production | `/dev/ttyUSB*` | ~70 % of China's PLC market |
| **Modbus TCP** | TCP | ✅ Production | 502 | Plant SCADA |
| **S7comm** (Siemens) | TCP | ✅ Production | 102 | S7-1200/1500/300/400 |
| **MC Protocol** (Mitsubishi) | TCP | 🚧 Planned | 5000 | MELSEC FX/Q/L/iQ-R |
| **OPC UA** | TCP | ⏳ Probe-only | 4840 | "Universal translator" |
| **EtherCAT** (Beckhoff) | Ethernet | ⏳ Planned | — | Servo / motion |
| **EtherNet/IP** (Rockwell) | TCP/UDP | ⏳ Planned | 44818/2222 | Allen-Bradley |
| **CAN 2.0B** | Serial | ⏳ Planned | `/dev/ttyUSBCAN` | Fuel cells |

> **How to read the status column**: ✅ = verified against a simulator in CI;
> 🚧 = design complete, implementation in progress; ⏳ = probe/connectivity
> only, or planned.

## Architecture

```
                         ┌──────────────────────────────────┐
   Your application ────►│         evernight crate           │
   (CLI / library /      │                                   │
    sensor-poll /        │  ┌─────────────────────────────┐ │
    API server)          │  │   ProtocolBackend trait      │ │
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
                              │   Physical hardware   │
                              │  PLC / sensor / valve │
                              └───────────────────────┘
```

Every protocol implements the same two traits, so adding a new one is a
**plug-in** — no changes to upstream consumers (sensor poller, discovery, CLI):

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

## Modbus (RTU over serial / TCP)

Modbus is the workhorse of industrial communication. Evernight wraps the `aoba`
crate behind `evernight::serial::modbus::ModbusMaster` and
`evernight::protocol::ModbusBackend`.

### Feature flags

```toml
[dependencies]
evernight = { version = "0.1", features = ["serial", "protocol"] }
```

- `serial` — `ModbusMaster`, register read/write, baud-rate auto-detection
- `protocol` — `ModbusBackend` (trait impl) + `ModbusProbe` (TCP auto-detect)

### Quick start — read registers from code

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

### Auto-detect baud rate

If you don't know the device's baud rate, sweep common rates:

```rust
use evernight::serial::probe_modbus_rtu_baud;

let detected = probe_modbus_rtu_baud("/dev/ttyUSB1", 0x13)?;
println!("Device responds at {} baud", detected.baud);
```

### Modbus TCP — probe + auto-detect

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

### CLI — probe a host for Modbus

```bash
# Probe common industrial ports (502=Modbus, 102=S7comm, …)
evernight probe 192.168.1.20 --ports 502,102,4840
```

### Testing without hardware

Evernight's CI uses a virtual serial port (`socat`) with an aoba Modbus slave.
Run the integration tests:

```bash
cargo test --features full --test serial_integration
```

These open a virtual TTY pair, run a real Modbus slave loop, and verify
read/write/baud-scan — no physical hardware required.

---

## S7comm (Siemens S7-1200/1500/300/400)

S7comm is Siemens' native protocol over ISO-on-TCP (port 102). Evernight wraps
the `rust7` crate (pure Rust, no FFI) for data access and `snap7-client` for
block download/flash.

### Feature flags

```toml
[dependencies]
evernight = { version = "0.1", features = ["s7comm", "manifest"] }
```

### Read a data block (DB)

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

### Discover unknown DBs + probe structure

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

### Block download + flash (PLC programming)

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

### S7 manifest — declarative polling config

Instead of hardcoding register layouts, describe the whole station in TOML:

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

Poll it with one command (S7 stations are polled by the parallel
`S7SensorPoller`, alarm rules auto-tagged with `protocol = "s7comm"`):

```bash
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### S7 field data types

| Type | Size | Offset format | Decode |
|------|------|---------------|--------|
| `BOOL` | 1 bit | `8.0` (byte 8, bit 0) | bit test |
| `BYTE` | 1 byte | `8` | `u8` |
| `WORD` | 2 bytes | `8` | `u16::from_be_bytes` |
| `INT` | 2 bytes | `8` | `i16::from_be_bytes` |
| `DWORD` | 4 bytes | `8` | `u32::from_be_bytes` |
| `DINT` | 4 bytes | `8` | `i32::from_be_bytes` |
| `REAL` | 4 bytes | `0` | `f32::from_be_bytes` |
| `STRING` | var | `20` | length-prefixed ASCII |

> The fractional part of `offset` encodes the **bit index** for BOOL fields.
> E.g. `10.3` = byte 10, bit 3.

### Testing without hardware

CI runs a `snap7-server` simulator (pure Rust S7-1500 emulator) and verifies
connect → read DB → scan DBs → probe structure → block ops:

```bash
cargo test --features full --test s7comm_integration
```

12 tests pass against the simulated PLC — this is a real end-to-end protocol
exchange, not a mock.

> **⚠️ TIA Portal prerequisite**: S7-1200/1500 PLCs require **"Permit access
> with PUT/GET"** enabled in the hardware configuration, and DBs must use
> **non-optimized block access** (right-click DB → Properties → uncheck
> "Optimized block access"). Optimized DBs return an error on byte-level reads.

---

## OPC UA (connectivity probe)

Evernight can detect OPC UA endpoints (TCP port 4840) but does not yet ship a
full client (the Rust `opcua` crate's client features are incomplete). Use the
probe to discover endpoints, then a dedicated OPC UA client until the backend
ships.

```bash
evernight probe 192.168.1.50 --ports 4840
```

---

## Discovery — autonomous protocol identification

Point evernight at an unknown network range or serial port and it identifies
the protocol automatically:

```
TCP endpoint                Serial bus
     │                         │
     ▼                         ▼
 ProtocolProbe chain       baud sweep + station scan
 (priority-sorted)         (probe_modbus_rtu_baud)
     │                         │
     ▼                         ▼
 ProtocolProbeResult       { port, baud, protocol, stations[] }
 { protocol, confidence,
   banner }
```

Probes are tried in priority order (Modbus=40 before S7comm=50); the first
match above the confidence threshold (default 0.5) wins.

---

## Alarm routing (Modbus + S7comm)

Sensor readings flow through a shared alarm pipeline. Each protocol gets its
own topic namespace so downstream consumers (entelecheia, shittim-chest) can
filter by source:

| Protocol | Trigger topic | Source id |
|----------|---------------|-----------|
| Modbus | `modbus.{station}.{field}.{level}` | `evernight.modbus.{station}` |
| S7comm | `s7comm.{station}.{field}.{level}` | `evernight.s7comm.{station}` |

Alarm levels follow ISA-18.2: `ll` / `l` / `h` / `hh` / `roc`, with hysteresis
deadband and debounce counting to prevent chattering.

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

## CLI command reference

| Command | Description |
|---------|-------------|
| `evernight probe <host> [--ports 502,102,...]` | Probe a host for industrial protocols |
| `evernight sensor-poll [--manifest X.toml]` | Poll sensor registers, emit alarms to entelecheia |
| `evernight file cp <local> <user@host:path>` | Upload a file over SSH |
| `evernight file get <user@host:path> <local>` | Download a file over SSH |
| `evernight file ls <user@host:path>` | List a remote directory |
| `evernight proxy <port> --host <jump>` | Local SOCKS5 proxy via SSH dynamic forward |
| `evernight exec --host X --command "..."` | Run a one-shot SSH command |
| `evernight hw` | Show local hardware telemetry |
| `evernight api-serve --transport ws` | Start the JSON-RPC API server |

---

## Roadmap

- **MC Protocol** (Mitsubishi) — hand-implemented binary frame codec, no Rust
  crate exists yet. Adds ~7 % PLC market coverage.
- **EtherCAT** (Beckhoff / Inovance) — via the `ethercrab` crate.
- **EtherNet/IP + CIP** (Rockwell) — class/instance/attribute addressing.
- **OPC UA client/server** — pending `opcua` crate maturity.
- **CAN 2.0B** — fuel-cell USB-CAN bridge parsing.
