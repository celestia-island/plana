+++
title = "Интеграция промышленных протоколов — Evernight"
description = """Руководство по интеграции промышленных протоколов evernight (Modbus, S7comm, OPC UA)."""
lang = "ru"
category = "guides"
subcategory = "router"
+++

# Интеграция промышленных протоколов — Evernight

Evernight — **обязательный брокер аппаратных возможностей** для экосистемы
celestia-island. Ни один вышестоящий crate не импортирует `aoba` / `rust7` / и т. п.
напрямую — весь физический ввод-вывод проходит через протокольные модули evernight.
Это руководство описывает, как подключаться, опрашивать, обнаруживать и формировать
тревоги по каждому поддерживаемому промышленному протоколу.

## Краткий обзор

| Протокол | Транспорт | Статус | Порт / Шина | Охват |
|----------|-----------|--------|------------|-------|
| **Modbus RTU** | Последовательный (RS-485) | ✅ Промышленная эксплуатация | `/dev/ttyUSB*` | ~70 % рынка ПЛК Китая |
| **Modbus TCP** | TCP | ✅ Промышленная эксплуатация | 502 | SCADA предприятия |
| **S7comm** (Siemens) | TCP | ✅ Промышленная эксплуатация | 102 | S7-1200/1500/300/400 |
| **MC Protocol** (Mitsubishi) | TCP | 🚧 Запланирован | 5000 | MELSEC FX/Q/L/iQ-R |
| **OPC UA** | TCP | ⏳ Только проба | 4840 | «Универсальный транслятор» |
| **EtherCAT** (Beckhoff) | Ethernet | 🚧 Запланирован | — | Сервоприводы / движение |
| **EtherNet/IP** (Rockwell) | TCP/UDP | 🚧 Запланирован | 44818/2222 | Allen-Bradley |
| **CAN 2.0B** | Последовательный | 🚧 Запланирован | `/dev/ttyUSBCAN` | Топливные элементы |

> **Как читать столбец статуса**: ✅ = проверено на симуляторе в CI; 🚧 =
> проектирование завершено, идёт реализация; ⏳ = только проба / проверка связи,
> либо запланировано.

## Архитектура

```
                          ┌──────────────────────────────────┐
   Ваше приложение ───────►│         crate evernight           │
   (CLI / библиотека /     │                                   │
    sensor-poll /          │  ┌─────────────────────────────┐ │
    API-сервер)            │  │   трейт ProtocolBackend      │ │
                          │  │   трейт ProtocolProbe         │ │
                          │  │   ProtocolRegistry            │ │
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
                               │   Физическое         │
                               │   оборудование       │
                               │  ПЛК / датчик / клапан │
                               └───────────────────────┘
```

Каждый протокол реализует одни и те же два трейта, поэтому добавление нового —
это **подключаемый модуль**: изменения в вышестоящих потребителях (опросчик
датчиков, обнаружение, CLI) не требуются:

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

## Modbus (RTU через последовательный порт / TCP)

Modbus — рабочая лошадка промышленной связи. Evernight оборачивает crate `aoba`
за `evernight::serial::modbus::ModbusMaster` и
`evernight::protocol::ModbusBackend`.

### Флаги функций

```toml
[dependencies]
evernight = { version = "0.1", features = ["serial", "protocol"] }
```

- `serial` — `ModbusMaster`, чтение/запись регистров, автоопределение скорости
  передачи (baud rate)
- `protocol` — `ModbusBackend` (реализация трейта) + `ModbusProbe`
  (автоопределение TCP)

### Быстрый старт — чтение регистров из кода

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

### Автоопределение скорости передачи

Если скорость передачи устройства неизвестна, выполните перебор стандартных
значений:

```rust
use evernight::serial::probe_modbus_rtu_baud;

let detected = probe_modbus_rtu_baud("/dev/ttyUSB1", 0x13)?;
println!("Device responds at {} baud", detected.baud);
```

### Modbus TCP — проба + автоопределение

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

### CLI — проверка хоста на наличие Modbus

```bash
# Probe common industrial ports (502=Modbus, 102=S7comm, …)
evernight probe 192.168.1.20 --ports 502,102,4840
```

### Тестирование без оборудования

В CI evernight используется виртуальный последовательный порт (`socat`) с
Modbus-ведомым (slave) на базе aoba. Запустите интеграционные тесты:

```bash
cargo test --features full --test serial_integration
```

Они открывают виртуальную пару TTY, запускают реальный цикл Modbus-ведомого и
проверяют чтение/запись/сканирование скорости — физическое оборудование не
требуется.

---

## S7comm (Siemens S7-1200/1500/300/400)

S7comm — собственный протокол Siemens поверх ISO-on-TCP (порт 102). Evernight
оборачивает crate `rust7` (чистый Rust, без FFI) для доступа к данным и
`snap7-client` для загрузки/прошивки блоков.

### Флаги функций

```toml
[dependencies]
evernight = { version = "0.1", features = ["s7comm", "manifest"] }
```

### Чтение блока данных (DB)

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

### Обнаружение неизвестных DB + анализ структуры

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

### Загрузка и прошивка блоков (программирование ПЛК)

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

### Манифест S7 — декларативная конфигурация опроса

Вместо жёсткого кодирования раскладки регистров опишите всю станцию в TOML:

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

Опрос одной командой (станции S7 опрашиваются параллельным `S7SensorPoller`,
правила тревог автоматически помечаются `protocol = "s7comm"`):

```bash
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### Типы данных полей S7

| Тип | Размер | Формат смещения | Декодирование |
|------|--------|-----------------|---------------|
| `BOOL` | 1 бит | `8.0` (байт 8, бит 0) | проверка бита |
| `BYTE` | 1 байт | `8` | `u8` |
| `WORD` | 2 байта | `8` | `u16::from_be_bytes` |
| `INT` | 2 байта | `8` | `i16::from_be_bytes` |
| `DWORD` | 4 байта | `8` | `u32::from_be_bytes` |
| `DINT` | 4 байта | `8` | `i32::from_be_bytes` |
| `REAL` | 4 байта | `0` | `f32::from_be_bytes` |
| `STRING` | перем. | `20` | ASCII с префиксом длины |

> Дробная часть `offset` задаёт **индекс бита** для полей BOOL. Например,
> `10.3` = байт 10, бит 3.

### Тестирование без оборудования

В CI запускается симулятор `snap7-server` (эмулятор S7-1500 на чистом Rust) и
проверяется цепочка подключение → чтение DB → сканирование DB → анализ структуры
→ операции с блоками:

```bash
cargo test --features full --test s7comm_integration
```

12 тестов проходят на симулированном ПЛК — это реальный сквозной обмен по
протоколу, а не заглушка (mock).

> **⚠️ Требование TIA Portal**: для ПЛК S7-1200/1500 необходимо включить
> **«Permit access with PUT/GET»** в аппаратной конфигурации, а DB должны
> использовать **неоптимизированный доступ к блокам** (ПКМ по DB → Properties →
> снимите флажок «Optimized block access»). Оптимизированные DB возвращают
> ошибку при побайтовом чтении.

---

## OPC UA (проба связи)

Evernight умеет обнаруживать конечные точки OPC UA (TCP-порт 4840), но пока не
поставляет полноценный клиент (клиентские возможности Rust-crate `opcua` не
завершены). Используйте пробу для обнаружения конечных точек, а затем —
специализированный клиент OPC UA, пока бэкенд не будет готов.

```bash
evernight probe 192.168.1.50 --ports 4840
```

---

## Обнаружение — автономная идентификация протоколов

Укажите evernight на неизвестный диапазон сети или последовательный порт, и он
определит протокол автоматически:

```
TCP-конечная точка          Последовательная шина
     │                         │
     ▼                         ▼
 Цепочка ProtocolProbe      перебор скоростей + скан станций
 (сортировка по приоритету) (probe_modbus_rtu_baud)
     │                         │
     ▼                         ▼
 ProtocolProbeResult       { port, baud, protocol, stations[] }
 { protocol, confidence,
   banner }
```

Пробы выполняются в порядке приоритета (Modbus=40 перед S7comm=50); побеждает
первое совпадение выше порога уверенности (по умолчанию 0.5).

---

## Маршрутизация тревог (Modbus + S7comm)

Показания датчиков проходят через общий конвейер тревог. Каждый протокол
получает собственное пространство имён тем, чтобы нижестоящие потребители
(entelecheia, shittim-chest) могли фильтровать по источнику:

| Протокол | Тема срабатывания | Идентификатор источника |
|----------|-------------------|-------------------------|
| Modbus | `modbus.{station}.{field}.{level}` | `evernight.modbus.{station}` |
| S7comm | `s7comm.{station}.{field}.{level}` | `evernight.s7comm.{station}` |

Уровни тревог соответствуют ISA-18.2: `ll` / `l` / `h` / `hh` / `roc`, с зоной
гистерезиса и счётом дебаунса для предотвращения дребезга.

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

## Справочник команд CLI

| Команда | Описание |
|---------|----------|
| `evernight probe <host> [--ports 502,102,...]` | Проверка хоста на наличие промышленных протоколов |
| `evernight sensor-poll [--manifest X.toml]` | Опрос регистров датчиков, отправка тревог в entelecheia |
| `evernight file cp <local> <user@host:path>` | Загрузка файла по SSH |
| `evernight file get <user@host:path> <local>` | Скачивание файла по SSH |
| `evernight file ls <user@host:path>` | Просмотр содержимого удалённого каталога |
| `evernight proxy <port> --host <jump>` | Локальный SOCKS5-прокси через динамическую переадресацию SSH |
| `evernight exec --host X --command "..."` | Выполнение разовой команды по SSH |
| `evernight hw` | Показ телеметрии локального оборудования |
| `evernight api-serve --transport ws` | Запуск API-сервера JSON-RPC |

---

## Дорожная карта

- **MC Protocol** (Mitsubishi) — вручную реализованный кодек бинарных кадров,
  соответствующего Rust-crate пока не существует. Добавляет ~7 % охвата рынка ПЛК.
- **EtherCAT** (Beckhoff / Inovance) — через crate `ethercrab`.
- **EtherNet/IP + CIP** (Rockwell) — адресация класс/экземпляр/атрибут.
- **Клиент/сервер OPC UA** — в ожидании зрелости crate `opcua`.
- **CAN 2.0B** — разбор моста USB-CAN для топливных элементов.
