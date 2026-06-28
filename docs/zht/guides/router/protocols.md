# 工業協定接入指南 — Evernight

Evernight 是 celestia-island 生態的**強制硬體能力代理（hardware capability broker）**。任何上游 crate 都不得直接匯入 `aoba` / `rust7` 等函式庫——所有物理 I/O 必須經過 evernight 的協定模組。本指南涵蓋如何連接、輪詢、發現並對每個支援的工業協定進行警報。

## 一覽

| 協定 | 傳輸層 | 狀態 | 連接埠 / 匯流排 | 涵蓋範圍 |
|------|--------|------|------------|----------|
| **Modbus RTU** | 序列埠（RS-485） | ✅ 生產就緒 | `/dev/ttyUSB*` | 約占中國 PLC 市場 70% |
| **Modbus TCP** | TCP | ✅ 生產就緒 | 502 | 廠級 SCADA |
| **S7comm**（西門子） | TCP | ✅ 生產就緒 | 102 | S7-1200/1500/300/400 |
| **MC Protocol**（三菱） | TCP | 🚧 規劃中 | 5000 | MELSEC FX/Q/L/iQ-R |
| **OPC UA** | TCP | ⏳ 僅探測 | 4840 | "萬能翻譯器" |
| **EtherCAT**（倍福） | 乙太網 | ⏳ 規劃中 | — | 伺服 / 運動 |
| **EtherNet/IP**（羅克韋爾） | TCP/UDP | ⏳ 規劃中 | 44818/2222 | Allen-Bradley |
| **CAN 2.0B** | 序列埠 | ⏳ 規劃中 | `/dev/ttyUSBCAN` | 燃料電池 |

> **狀態欄說明**：✅ = 已在 CI 中通過模擬器驗證；🚧 = 設計完成，實作進行中；⏳ = 僅探測/連通性檢查，或規劃中。

## 架構

```
                         ┌──────────────────────────────────┐
   你的應用 ────────────►│         evernight crate           │
   (CLI / 庫 /            │                                   │
    sensor-poll /         │  ┌─────────────────────────────┐ │
    API 服務)             │  │   ProtocolBackend trait      │ │
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
                         │   │(Modbus) │  │ snap7-cli │      │
                         │   └─────────┘  └───────────┘      │
                         └──────────────────────────────────┘
                                          │
                              ┌───────────┴───────────┐
                              │      物理硬體         │
                              │  PLC / 感測器 / 閥門  │
                              └───────────────────────┘
```

每個協定實作相同的兩個 trait，因此新增協定是**外掛式**的——無需修改上游消費者（感測器輪詢器、發現工具、CLI）：

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

## Modbus（序列埠 RTU / TCP）

Modbus 是工業通訊的主力。Evernight 將 `aoba` crate 封裝在 `evernight::serial::modbus::ModbusMaster` 和 `evernight::protocol::ModbusBackend` 之後。

### Feature 開關

```toml
[dependencies]
evernight = { version = "0.1", features = ["serial", "protocol"] }
```

- `serial` — `ModbusMaster`、暫存器讀寫、鮑率自動偵測
- `protocol` — `ModbusBackend`（trait 實作）+ `ModbusProbe`（TCP 自動偵測）

### 快速上手 — 程式碼讀取暫存器

```rust
use evernight::serial::modbus::{ModbusMaster, RegisterMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let master = ModbusMaster::builder(0x13)        // 站号 19
        .with_port("/dev/ttyUSB1")
        .with_timeout(5000)
        .open()?;

    let result = master.read_registers(RegisterMode::Holding, 0x10, 3)?;
    println!("寄存器值: {:?}", result.values);       // [压力1, 压力2, 压力3]
    Ok(())
}
```

### 自動偵測鮑率

如果不知道設備的鮑率，可以掃描常見速率：

```rust
use evernight::serial::probe_modbus_rtu_baud;

let detected = probe_modbus_rtu_baud("/dev/ttyUSB1", 0x13)?;
println!("设备在 {} 波特率下响应", detected.baud);
```

### Modbus TCP — 探測 + 自動識別

```rust
use evernight::protocol::{ProtocolRegistry, ModbusProbe, TransportInfo};
use std::sync::Arc;

let mut registry = ProtocolRegistry::new();
registry.register_probe(Arc::new(ModbusProbe));

let transport = TransportInfo::Tcp { host: "192.168.1.20".into(), port: 502 };
if let Some(result) = registry.auto_detect(&transport, 0.5).await {
    println!("检测到 {}（置信度 {:.0}%）", result.protocol, result.confidence * 100.0);
}
```

### CLI — 探測主機的 Modbus

```bash
# 探测常见工业端口（502=Modbus, 102=S7comm, …）
evernight probe 192.168.1.20 --ports 502,102,4840
```

### 無硬體測試

Evernight 的 CI 使用虛擬序列埠（`socat`）配合 aoba 的 Modbus 從站。執行整合測試：

```bash
cargo test --features full --test serial_integration
```

這些測試開啟一對虛擬 TTY，執行真實的 Modbus 從站迴圈，驗證讀寫/鮑率掃描——無需物理硬體。

---

## S7comm（西門子 S7-1200/1500/300/400）

S7comm 是西門子的原生協定，基於 ISO-on-TCP（連接埠 102）。Evernight 用 `rust7` crate（純 Rust，無 FFI）做資料存取，用 `snap7-client` 做區塊下載/燒錄。

### Feature 開關

```toml
[dependencies]
evernight = { version = "0.1", features = ["s7comm", "manifest"] }
```

### 讀取資料區塊（DB）

```rust
use evernight::protocol::s7comm::{S7CommClient, S7ConnectParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = S7CommClient::new(S7ConnectParams {
        host: "192.168.1.10".into(),
        port: 102,
        rack: 0,
        slot: 1,            // S7-1500: slot 1；S7-300: slot 2
    });
    client.connect().await?;

    // 从 DB1 偏移 0 读取 4 字节（一个 REAL 温度值）
    let bytes = client.read_db(1, 0, 4).await?;
    let temp = f32::from_be_bytes(bytes.try_into().unwrap());
    println!("DB1.DBD0 = {:.1} °C", temp);

    client.disconnect().await;
    Ok(())
}
```

### 發現未知 DB + 探測結構

```rust
use evernight::protocol::s7comm::{scan_db_numbers, probe_db_structure};

// 这个 PLC 上有哪些 DB 编号？
let dbs = scan_db_numbers(&client, 1, 100).await?;
for db in &dbs {
    println!("DB{}: {}", db.db_number, db.status);   // ok / optimized / not_found
}

// 读取 DB1 前 512 字节用于类型推断
let report = probe_db_structure(&client, 1, 512).await?;
println!("DB1 原始字节: {} 字节, 可读={}", report.raw_bytes.len(), report.readable);
```

### 區塊下載 + 燒錄（PLC 程式設計）

```rust
use evernight::protocol::s7comm_blocks::{S7BlockOps, flash_block};

let ops = S7BlockOps::new(&client);
let binary_blocks = ops.full_upload_block(1).await?;   // 读取已有块
// … 修改 MC7 二进制 …
ops.download_block(block_type, block_num, &new_binary).await?;

// 一步完成：停止 → 下载 → 重启：
let result = flash_block(&client, block_type, block_num, &new_binary).await?;
println!("烧录完成，PLC 已重启: {:?}", result);
```

### S7 manifest — 宣告式輪詢設定

不用硬編碼暫存器佈局，而是用 TOML 描述整個站點：

```toml
# corridor.toml
format_version = "1"

[facility]
id = "hydrogen-plant"
name = "制氢产线"

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

# 要轮询的 S7 数据块
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

一條命令即可輪詢（S7 站點由平行的 `S7SensorPoller` 輪詢，警報規則自動標記 `protocol = "s7comm"`）：

```bash
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### S7 欄位資料型別

| 型別 | 大小 | 偏移格式 | 解碼 |
|------|------|----------|------|
| `BOOL` | 1 位元 | `8.0`（位元組 8，位元 0） | 位元測試 |
| `BYTE` | 1 位元組 | `8` | `u8` |
| `WORD` | 2 位元組 | `8` | `u16::from_be_bytes` |
| `INT` | 2 位元組 | `8` | `i16::from_be_bytes` |
| `DWORD` | 4 位元組 | `8` | `u32::from_be_bytes` |
| `DINT` | 4 位元組 | `8` | `i32::from_be_bytes` |
| `REAL` | 4 位元組 | `0` | `f32::from_be_bytes` |
| `STRING` | 變長 | `20` | 長度前綴 ASCII |

> `offset` 的小數部分編碼 BOOL 欄位的**位元索引**。例如 `10.3` = 位元組 10，位元 3。

### 無硬體測試

CI 執行 `snap7-server` 模擬器（純 Rust S7-1500 模擬器），驗證 連接 → 讀 DB → 掃描 DB → 探測結構 → 區塊操作：

```bash
cargo test --features full --test s7comm_integration
```

12 個測試通過——這是真實的端對端協定互動，不是 mock。

> **⚠️ TIA Portal 前置條件**：S7-1200/1500 PLC 需要在硬體組態中啟用
> **"允許 PUT/GET 存取"**，且 DB 必須使用**非最佳化區塊存取**（右鍵 DB → 屬性 →
> 取消勾選"最佳化的區塊存取"）。最佳化 DB 在位元組層級讀取時會回傳錯誤。

---

## OPC UA（連通性探測）

Evernight 可以偵測 OPC UA 端點（TCP 連接埠 4840），但尚未提供完整用戶端（Rust `opcua` crate 的用戶端功能尚不完善）。用探測發現端點，後端上線前使用專用 OPC UA 用戶端。

```bash
evernight probe 192.168.1.50 --ports 4840
```

---

## 發現 — 自主協定識別

將 evernight 指向未知的網路範圍或序列埠，它會自動識別協定：

```
TCP 端點                    序列匯流排
     │                         │
     ▼                         ▼
 ProtocolProbe 鏈          鮑率掃描 + 站號掃描
 （按優先順序排序）           (probe_modbus_rtu_baud)
     │                         │
     ▼                         ▼
 ProtocolProbeResult       { port, baud, protocol, stations[] }
 { protocol, confidence,
   banner }
```

探測按優先順序執行（Modbus=40 先於 S7comm=50）；信心度閾值（預設 0.5）以上的首個匹配勝出。

---

## 警報路由（Modbus + S7comm）

感測器讀數流經共享的警報管道。每個協定有自己的 topic 命名空間，下游消費者（entelecheia、shittim-chest）可以按來源過濾：

| 協定 | 觸發 topic | 來源 ID |
|------|-----------|---------|
| Modbus | `modbus.{站號}.{欄位}.{級別}` | `evernight.modbus.{站號}` |
| S7comm | `s7comm.{站號}.{欄位}.{級別}` | `evernight.s7comm.{站號}` |

警報級別遵循 ISA-18.2：`ll` / `l` / `h` / `hh` / `roc`，帶遲滯死區和去抖計數以防止抖動。

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

## CLI 命令參考

| 命令 | 說明 |
|------|------|
| `evernight probe <host> [--ports 502,102,...]` | 探測主機的工業協定 |
| `evernight sensor-poll [--manifest X.toml]` | 輪詢感測器暫存器，向 entelecheia 傳送警報 |
| `evernight file cp <本地> <user@host:路徑>` | 透過 SSH 上傳檔案 |
| `evernight file get <user@host:路徑> <本地>` | 透過 SSH 下載檔案 |
| `evernight file ls <user@host:路徑>` | 列出遠端目錄 |
| `evernight proxy <連接埠> --host <跳板機>` | 透過 SSH 動態轉送的本地 SOCKS5 代理 |
| `evernight exec --host X --command "..."` | 執行一次性 SSH 命令 |
| `evernight hw` | 顯示本地硬體遙測 |
| `evernight api-serve --transport ws` | 啟動 JSON-RPC API 服務 |

---

## 路線圖

- **MC Protocol**（三菱）— 手寫二進位訊框編解碼器，目前無 Rust crate。增加約 7% PLC 市場覆蓋。
- **EtherCAT**（倍福 / 匯川）— 透過 `ethercrab` crate。
- **EtherNet/IP + CIP**（羅克韋爾）— 類別/實例/屬性定址。
- **OPC UA 用戶端/伺服端** — 取決於 `opcua` crate 成熟度。
- **CAN 2.0B** — 燃料電池 USB-CAN 橋接解析。
