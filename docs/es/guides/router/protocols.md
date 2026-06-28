# Integración de Protocolos Industriales — Evernight

Evernight es el **broker obligatorio de capacidades de hardware** para el
ecosistema celestia-island. Ningún crate de nivel superior importa
`aoba` / `rust7` / etc. directamente — toda la E/S física se enruta a través de
los módulos de protocolo de evernight. Esta guía cubre cómo conectarse, hacer
sondeo (polling), descubrir y generar alarmas en cada protocolo industrial
soportado.

## Vista general

| Protocolo | Transporte | Estado | Puerto / Bus | Cobertura |
|-----------|------------|--------|--------------|-----------|
| **Modbus RTU** | Serie (RS-485) | ✅ Producción | `/dev/ttyUSB*` | ~70 % del mercado de PLC de China |
| **Modbus TCP** | TCP | ✅ Producción | 502 | SCADA de planta |
| **S7comm** (Siemens) | TCP | ✅ Producción | 102 | S7-1200/1500/300/400 |
| **MC Protocol** (Mitsubishi) | TCP | 🚧 Planificado | 5000 | MELSEC FX/Q/L/iQ-R |
| **OPC UA** | TCP | ⏳ Solo sonda | 4840 | "Traductor universal" |
| **EtherCAT** (Beckhoff) | Ethernet | 🚧 Planificado | — | Servo / movimiento |
| **EtherNet/IP** (Rockwell) | TCP/UDP | 🚧 Planificado | 44818/2222 | Allen-Bradley |
| **CAN 2.0B** | Serie | 🚧 Planificado | `/dev/ttyUSBCAN` | Pilas de combustible |

> **Cómo leer la columna de estado**: ✅ = verificado contra un simulador en CI;
> 🚧 = diseño completo, implementación en curso; ⏳ = solo sonda/conectividad,
> o planificado.

## Arquitectura

```
                          ┌──────────────────────────────────┐
   Su aplicación ─────────►│         crate evernight          │
   (CLI / librería /       │                                  │
    sensor-poll /          │  ┌─────────────────────────────┐ │
    servidor API)          │  │   ProtocolBackend trait      │ │
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
                              │   Hardware físico     │
                              │  PLC / sensor / válv  │
                              └───────────────────────┘
```

Cada protocolo implementa los mismos dos traits, por lo que añadir uno nuevo es
un **plug-in** — sin cambios para los consumidores de nivel superior (sondeador
de sensores, descubrimiento, CLI):

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

## Modbus (RTU sobre serie / TCP)

Modbus es el caballo de batalla de la comunicación industrial. Evernight
envuelve el crate `aoba` detrás de
`evernight::serial::modbus::ModbusMaster` y
`evernight::protocol::ModbusBackend`.

### Feature flags

```toml
[dependencies]
evernight = { version = "0.1", features = ["serial", "protocol"] }
```

- `serial` — `ModbusMaster`, lectura/escritura de registros, detección
  automática de baud-rate
- `protocol` — `ModbusBackend` (impl del trait) + `ModbusProbe` (auto-detección
  TCP)

### Inicio rápido — leer registros desde código

```rust
use evernight::serial::modbus::{ModbusMaster, RegisterMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let master = ModbusMaster::builder(0x13)        // estación 19
        .with_port("/dev/ttyUSB1")
        .with_timeout(5000)
        .open()?;

    let result = master.read_registers(RegisterMode::Holding, 0x10, 3)?;
    println!("Registers: {:?}", result.values);     // [pressure, pressure2, pressure3]
    Ok(())
}
```

### Detección automática de baud rate

Si no conoce el baud rate del dispositivo, recorra las velocidades comunes:

```rust
use evernight::serial::probe_modbus_rtu_baud;

let detected = probe_modbus_rtu_baud("/dev/ttyUSB1", 0x13)?;
println!("Device responds at {} baud", detected.baud);
```

### Modbus TCP — sonda + auto-detección

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

### CLI — sondear un host en busca de Modbus

```bash
# Sondear puertos industriales comunes (502=Modbus, 102=S7comm, …)
evernight probe 192.168.1.20 --ports 502,102,4840
```

### Pruebas sin hardware

El CI de evernight usa un puerto serie virtual (`socat`) con un esclavo Modbus
de aoba. Ejecute las pruebas de integración:

```bash
cargo test --features full --test serial_integration
```

Estas abren un par de TTY virtual, ejecutan un bucle real de esclavo Modbus y
verifican lectura/escritura/escaneo-de-baud — sin necesidad de hardware físico.

---

## S7comm (Siemens S7-1200/1500/300/400)

S7comm es el protocolo nativo de Siemens sobre ISO-on-TCP (puerto 102).
Evernight envuelve el crate `rust7` (Rust puro, sin FFI) para el acceso a datos
y `snap7-client` para la descarga/flasheado de bloques.

### Feature flags

```toml
[dependencies]
evernight = { version = "0.1", features = ["s7comm", "manifest"] }
```

### Leer un bloque de datos (DB)

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

### Descubrir DBs desconocidos + sondear estructura

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

### Descarga de bloque + flasheado (programación de PLC)

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

### Manifiesto S7 — configuración declarativa de sondeo

En lugar de codificar los layouts de registros de forma rígida, describa toda
la estación en TOML:

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

Sondéelo con un solo comando (las estaciones S7 son sondeadas por
`S7SensorPoller` en paralelo, las reglas de alarma se etiquetan automáticamente
con `protocol = "s7comm"`):

```bash
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### Tipos de datos de campo S7

| Tipo | Tamaño | Formato de offset | Decodificación |
|------|--------|-------------------|----------------|
| `BOOL` | 1 bit | `8.0` (byte 8, bit 0) | test de bit |
| `BYTE` | 1 byte | `8` | `u8` |
| `WORD` | 2 bytes | `8` | `u16::from_be_bytes` |
| `INT` | 2 bytes | `8` | `i16::from_be_bytes` |
| `DWORD` | 4 bytes | `8` | `u32::from_be_bytes` |
| `DINT` | 4 bytes | `8` | `i32::from_be_bytes` |
| `REAL` | 4 bytes | `0` | `f32::from_be_bytes` |
| `STRING` | var | `20` | ASCII con prefijo de longitud |

> La parte fraccionaria de `offset` codifica el **índice de bit** para los
> campos BOOL. P. ej. `10.3` = byte 10, bit 3.

### Pruebas sin hardware

El CI ejecuta un simulador `snap7-server` (emulador de S7-1500 puro en Rust) y
verifica conectar → leer DB → escanear DBs → sondear estructura → ops de
bloque:

```bash
cargo test --features full --test s7comm_integration
```

12 pruebas pasan contra el PLC simulado — es un intercambio de protocolo real
de extremo a extremo, no un mock.

> **⚠️ Requisito previo de TIA Portal**: los PLC S7-1200/1500 requieren tener
> **"Permit access with PUT/GET"** habilitado en la configuración de hardware, y
> los DBs deben usar **acceso a bloque no optimizado** (clic derecho en el DB →
> Properties → desmarcar "Optimized block access"). Los DBs optimizados
> devuelven un error en lecturas a nivel de byte.

---

## OPC UA (sonda de conectividad)

Evernight puede detectar endpoints OPC UA (puerto TCP 4840) pero aún no envía
un cliente completo (las características de cliente del crate `opcua` de Rust
están incompletas). Use la sonda para descubrir endpoints, luego un cliente
OPC UA dedicado hasta que el backend esté disponible.

```bash
evernight probe 192.168.1.50 --ports 4840
```

---

## Descubrimiento — identificación autónoma de protocolo

Apunte evernight a un rango de red desconocido o a un puerto serie e
identificará el protocolo automáticamente:

```
Endpoint TCP              Bus serie
      │                         │
      ▼                         ▼
  Cadena ProtocolProbe      barrido de baud + escaneo de estación
  (ordenado por prioridad)  (probe_modbus_rtu_baud)
      │                         │
      ▼                         ▼
  ProtocolProbeResult       { port, baud, protocol, stations[] }
  { protocol, confidence,
    banner }
```

Las sondas se prueban en orden de prioridad (Modbus=40 antes que S7comm=50); la
primera coincidencia por encima del umbral de confianza (por defecto 0.5)
gana.

---

## Enrutamiento de alarmas (Modbus + S7comm)

Las lecturas de sensores fluyen a través de una tubería de alarmas compartida.
Cada protocolo obtiene su propio espacio de nombres de topic para que los
consumidores de nivel inferior (entelecheia, shittim-chest) puedan filtrar por
origen:

| Protocolo | Topic de disparo | Id de origen |
|-----------|------------------|--------------|
| Modbus | `modbus.{station}.{field}.{level}` | `evernight.modbus.{station}` |
| S7comm | `s7comm.{station}.{field}.{level}` | `evernight.s7comm.{station}` |

Los niveles de alarma siguen ISA-18.2: `ll` / `l` / `h` / `hh` / `roc`, con
una banda muerta de histéresis y un conteo de debounce para evitar el
parpadeo (chattering).

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

## Referencia de comandos CLI

| Comando | Descripción |
|---------|-------------|
| `evernight probe <host> [--ports 502,102,...]` | Sondear un host en busca de protocolos industriales |
| `evernight sensor-poll [--manifest X.toml]` | Sondar registros de sensores, emitir alarmas a entelecheia |
| `evernight file cp <local> <user@host:path>` | Subir un archivo por SSH |
| `evernight file get <user@host:path> <local>` | Descargar un archivo por SSH |
| `evernight file ls <user@host:path>` | Listar un directorio remoto |
| `evernight proxy <port> --host <jump>` | Proxy SOCKS5 local vía SSH dynamic forward |
| `evernight exec --host X --command "..."` | Ejecutar un comando SSH de un solo uso |
| `evernight hw` | Mostrar telemetría del hardware local |
| `evernight api-serve --transport ws` | Iniciar el servidor de API JSON-RPC |

---

## Hoja de ruta

- **MC Protocol** (Mitsubishi) — códec de trama binaria implementado a mano,
  todavía no existe ningún crate en Rust. Añade ~7 % de cobertura del mercado
  de PLC.
- **EtherCAT** (Beckhoff / Inovance) — vía el crate `ethercrab`.
- **EtherNet/IP + CIP** (Rockwell) — direccionamiento de
  clase/instancia/atributo.
- **Cliente/servidor OPC UA** — pendiente de la madurez del crate `opcua`.
- **CAN 2.0B** — puente USB-CAN para pilas de combustible con análisis.
