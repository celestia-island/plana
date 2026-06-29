+++
title = "工业协议接入指南 — Evernight"
description = """evernight 工业协议集成指南（Modbus、S7comm、OPC UA）。"""
lang = "zhs"
category = "guides"
subcategory = "router"
+++

# 工业协议接入指南 — Evernight

Evernight 是 celestia-island 生态的**强制硬件能力代理（hardware capability broker）**。任何上游 crate 都不得直接导入 `aoba` / `rust7` 等库——所有物理 I/O 必须经过 evernight 的协议模块。本指南覆盖如何连接、轮询、发现并对每个支持的工业协议进行告警。

## 一览

| 协议 | 传输层 | 状态 | 端口 / 总线 | 覆盖范围 |
|------|--------|------|------------|----------|
| **Modbus RTU** | 串口（RS-485） | ✅ 生产就绪 | `/dev/ttyUSB*` | 约占中国 PLC 市场 70% |
| **Modbus TCP** | TCP | ✅ 生产就绪 | 502 | 厂级 SCADA |
| **S7comm**（西门子） | TCP | ✅ 生产就绪 | 102 | S7-1200/1500/300/400 |
| **MC Protocol**（三菱） | TCP | 🚧 规划中 | 5000 | MELSEC FX/Q/L/iQ-R |
| **OPC UA** | TCP | ⏳ 仅探测 | 4840 | "万能翻译器" |
| **EtherCAT**（倍福） | 以太网 | ⏳ 规划中 | — | 伺服 / 运动 |
| **EtherNet/IP**（罗克韦尔） | TCP/UDP | ⏳ 规划中 | 44818/2222 | Allen-Bradley |
| **CAN 2.0B** | 串口 | ⏳ 规划中 | `/dev/ttyUSBCAN` | 燃料电池 |

> **状态列说明**：✅ = 已在 CI 中通过模拟器验证；🚧 = 设计完成，实现进行中；⏳ = 仅探测/连通性检查，或规划中。

## 架构

```
                         ┌──────────────────────────────────┐
   你的应用 ────────────►│         evernight crate           │
   (CLI / 库 /            │                                   │
    sensor-poll /         │  ┌─────────────────────────────┐ │
    API 服务)             │  │   ProtocolBackend trait      │ │
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
                              │      物理硬件         │
                              │  PLC / 传感器 / 阀门  │
                              └───────────────────────┘
```

每个协议实现相同的两个 trait，因此新增协议是**插件式**的——无需修改上游消费者（传感器轮询器、发现工具、CLI）：

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

## Modbus（串口 RTU / TCP）

Modbus 是工业通信的主力。Evernight 将 `aoba` crate 封装在 `evernight::serial::modbus::ModbusMaster` 和 `evernight::protocol::ModbusBackend` 之后。

### Feature 开关

```toml
[dependencies]
evernight = { version = "0.1", features = ["serial", "protocol"] }
```

- `serial` — `ModbusMaster`、寄存器读写、波特率自动检测
- `protocol` — `ModbusBackend`（trait 实现）+ `ModbusProbe`（TCP 自动检测）

### 快速上手 — 代码读取寄存器

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

### 自动检测波特率

如果不知道设备的波特率，可以扫描常见速率：

```rust
use evernight::serial::probe_modbus_rtu_baud;

let detected = probe_modbus_rtu_baud("/dev/ttyUSB1", 0x13)?;
println!("设备在 {} 波特率下响应", detected.baud);
```

### Modbus TCP — 探测 + 自动识别

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

### CLI — 探测主机的 Modbus

```bash
# 探测常见工业端口（502=Modbus, 102=S7comm, …）
evernight probe 192.168.1.20 --ports 502,102,4840
```

### 无硬件测试

Evernight 的 CI 使用虚拟串口（`socat`）配合 aoba 的 Modbus 从站。运行集成测试：

```bash
cargo test --features full --test serial_integration
```

这些测试打开一对虚拟 TTY，运行真实的 Modbus 从站循环，验证读写/波特率扫描——无需物理硬件。

---

## S7comm（西门子 S7-1200/1500/300/400）

S7comm 是西门子的原生协议，基于 ISO-on-TCP（端口 102）。Evernight 用 `rust7` crate（纯 Rust，无 FFI）做数据访问，用 `snap7-client` 做块下载/烧录。

### Feature 开关

```toml
[dependencies]
evernight = { version = "0.1", features = ["s7comm", "manifest"] }
```

### 读取数据块（DB）

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

### 发现未知 DB + 探测结构

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

### 块下载 + 烧录（PLC 编程）

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

### S7 manifest — 声明式轮询配置

不用硬编码寄存器布局，而是用 TOML 描述整个站点：

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

一条命令即可轮询（S7 站点由并行的 `S7SensorPoller` 轮询，告警规则自动标记 `protocol = "s7comm"`）：

```bash
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### S7 字段数据类型

| 类型 | 大小 | 偏移格式 | 解码 |
|------|------|----------|------|
| `BOOL` | 1 位 | `8.0`（字节 8，位 0） | 位测试 |
| `BYTE` | 1 字节 | `8` | `u8` |
| `WORD` | 2 字节 | `8` | `u16::from_be_bytes` |
| `INT` | 2 字节 | `8` | `i16::from_be_bytes` |
| `DWORD` | 4 字节 | `8` | `u32::from_be_bytes` |
| `DINT` | 4 字节 | `8` | `i32::from_be_bytes` |
| `REAL` | 4 字节 | `0` | `f32::from_be_bytes` |
| `STRING` | 变长 | `20` | 长度前缀 ASCII |

> `offset` 的小数部分编码 BOOL 字段的**位索引**。例如 `10.3` = 字节 10，位 3。

### 无硬件测试

CI 运行 `snap7-server` 模拟器（纯 Rust S7-1500 仿真器），验证 连接 → 读 DB → 扫描 DB → 探测结构 → 块操作：

```bash
cargo test --features full --test s7comm_integration
```

12 个测试通过——这是真实的端到端协议交互，不是 mock。

> **⚠️ TIA Portal 前置条件**：S7-1200/1500 PLC 需要在硬件组态中启用
> **"允许 PUT/GET 访问"**，且 DB 必须使用**非优化块访问**（右键 DB → 属性 →
> 取消勾选"优化的块访问"）。优化 DB 在字节级读取时会返回错误。

---

## OPC UA（连通性探测）

Evernight 可以检测 OPC UA 端点（TCP 端口 4840），但尚未提供完整客户端（Rust `opcua` crate 的客户端功能尚不完善）。用探测发现端点，后端上线前使用专用 OPC UA 客户端。

```bash
evernight probe 192.168.1.50 --ports 4840
```

---

## 发现 — 自主协议识别

将 evernight 指向未知的网络范围或串口，它会自动识别协议：

```
TCP 端点                    串行总线
     │                         │
     ▼                         ▼
 ProtocolProbe 链          波特率扫描 + 站号扫描
 （按优先级排序）           (probe_modbus_rtu_baud)
     │                         │
     ▼                         ▼
 ProtocolProbeResult       { port, baud, protocol, stations[] }
 { protocol, confidence,
   banner }
```

探测按优先级顺序执行（Modbus=40 先于 S7comm=50）；置信度阈值（默认 0.5）以上的首个匹配胜出。

---

## 告警路由（Modbus + S7comm）

传感器读数流经共享的告警管道。每个协议有自己的 topic 命名空间，下游消费者（entelecheia、shittim-chest）可以按来源过滤：

| 协议 | 触发 topic | 来源 ID |
|------|-----------|---------|
| Modbus | `modbus.{站号}.{字段}.{级别}` | `evernight.modbus.{站号}` |
| S7comm | `s7comm.{站号}.{字段}.{级别}` | `evernight.s7comm.{站号}` |

告警级别遵循 ISA-18.2：`ll` / `l` / `h` / `hh` / `roc`，带回环死区和防抖计数以防止抖动。

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

## CLI 命令参考

| 命令 | 说明 |
|------|------|
| `evernight probe <host> [--ports 502,102,...]` | 探测主机的工业协议 |
| `evernight sensor-poll [--manifest X.toml]` | 轮询传感器寄存器，向 entelecheia 发送告警 |
| `evernight file cp <本地> <user@host:路径>` | 通过 SSH 上传文件 |
| `evernight file get <user@host:路径> <本地>` | 通过 SSH 下载文件 |
| `evernight file ls <user@host:路径>` | 列出远程目录 |
| `evernight proxy <端口> --host <跳板机>` | 通过 SSH 动态转发的本地 SOCKS5 代理 |
| `evernight exec --host X --command "..."` | 执行一次性 SSH 命令 |
| `evernight hw` | 显示本地硬件遥测 |
| `evernight api-serve --transport ws` | 启动 JSON-RPC API 服务 |

---

## 路线图

- **MC Protocol**（三菱）— 手写二进制帧编解码器，目前无 Rust crate。增加约 7% PLC 市场覆盖。
- **EtherCAT**（倍福 / 汇川）— 通过 `ethercrab` crate。
- **EtherNet/IP + CIP**（罗克韦尔）— 类/实例/属性寻址。
- **OPC UA 客户端/服务端** — 取决于 `opcua` crate 成熟度。
- **CAN 2.0B** — 燃料电池 USB-CAN 桥接解析。
