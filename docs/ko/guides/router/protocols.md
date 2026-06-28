# 산업 프로토콜 연동 가이드 — Evernight

Evernight은 celestia-island 생태계의 **필수 하드웨어 능력 브로커(hardware capability broker)**입니다. 모든 상류 crate은 `aoba` / `rust7` 등의 라이브러리를 직접 임포트해서는 안 됩니다——모든 물리 I/O는 evernight의 프로토콜 모듈을 거쳐야 합니다. 이 가이드는 지원되는 각 산업 프로토콜에 대해 연결, 폴링, 발견 및 알람 수행 방법을 다룹니다.

## 개요

| 프로토콜 | 전송 계층 | 상태 | 포트 / 버스 | 지원 범위 |
|------|--------|------|------------|----------|
| **Modbus RTU** | 시리얼(RS-485) | ✅ 프로덕션 준비 완료 | `/dev/ttyUSB*` | 중국 PLC 시장의 약 70% |
| **Modbus TCP** | TCP | ✅ 프로덕션 준비 완료 | 502 | 공장급 SCADA |
| **S7comm**(지멘스) | TCP | ✅ 프로덕션 준비 완료 | 102 | S7-1200/1500/300/400 |
| **MC Protocol**(미쓰비시) | TCP | 🚧 계획 중 | 5000 | MELSEC FX/Q/L/iQ-R |
| **OPC UA** | TCP | ⏳ 탐지 전용 | 4840 | "만능 번역기" |
| **EtherCAT**(벡호프) | 이더넷 | ⏳ 계획 중 | — | 서보 / 모션 |
| **EtherNet/IP**(록웰) | TCP/UDP | ⏳ 계획 중 | 44818/2222 | Allen-Bradley |
| **CAN 2.0B** | 시리얼 | ⏳ 계획 중 | `/dev/ttyUSBCAN` | 연료 전지 |

> **상태 열 설명**: ✅ = CI에서 시뮬레이터로 검증 완료; 🚧 = 설계 완료, 구현 진행 중; ⏳ = 탐지/연결 확인 전용 또는 계획 중.

## 아키텍처

```
                         ┌──────────────────────────────────┐
   당신의 애플리케이션 ─────►│         evernight crate           │
   (CLI / 라이브러리 /      │                                   │
    sensor-poll /          │  ┌─────────────────────────────┐ │
    API 서버)               │  │   ProtocolBackend trait      │ │
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
                              │      물리 하드웨어        │
                              │  PLC / 센서 / 밸브        │
                              └───────────────────────┘
```

각 프로토콜은 동일한 두 가지 trait을 구현하므로, 새 프로토콜 추가는 **플러그인 방식**입니다——상류 소비자(센서 폴러, 발견 도구, CLI)를 수정할 필요가 없습니다:

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

## Modbus(시리얼 RTU / TCP)

Modbus는 산업 통신의 핵심입니다. Evernight은 `aoba` crate을 `evernight::serial::modbus::ModbusMaster`와 `evernight::protocol::ModbusBackend` 뒤에 캡슐화합니다.

### Feature 스위치

```toml
[dependencies]
evernight = { version = "0.1", features = ["serial", "protocol"] }
```

- `serial` — `ModbusMaster`, 레지스터 읽기/쓰기, 보레이트 자동 감지
- `protocol` — `ModbusBackend`(trait 구현) + `ModbusProbe`(TCP 자동 감지)

### 빠른 시작 — 코드로 레지스터 읽기

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

### 보레이트 자동 감지

기기의 보레이트를 모를 경우, 일반적인 속도를 스캔할 수 있습니다:

```rust
use evernight::serial::probe_modbus_rtu_baud;

let detected = probe_modbus_rtu_baud("/dev/ttyUSB1", 0x13)?;
println!("设备在 {} 波特率下响应", detected.baud);
```

### Modbus TCP — 탐지 + 자동 식별

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

### CLI — 호스트의 Modbus 탐지

```bash
# 探测常见工业端口（502=Modbus, 102=S7comm, …）
evernight probe 192.168.1.20 --ports 502,102,4840
```

### 하드웨어 없이 테스트

Evernight의 CI는 가상 시리얼 포트(`socat`)를 aoba의 Modbus 슬레이브와 함께 사용합니다. 통합 테스트를 실행하세요:

```bash
cargo test --features full --test serial_integration
```

이 테스트는 가상 TTY 쌍을 열고, 실제 Modbus 슬레이브 루프를 실행하여 읽기/쓰기 및 보레이트 스캔을 검증합니다——물리 하드웨어가 필요 없습니다.

---

## S7comm(지멘스 S7-1200/1500/300/400)

S7comm은 지멘스의 네이티브 프로토콜로, ISO-on-TCP(포트 102) 기반입니다. Evernight은 `rust7` crate(순수 Rust, FFI 없음)으로 데이터 접근을, `snap7-client`로 블록 다운로드/플래시를 수행합니다.

### Feature 스위치

```toml
[dependencies]
evernight = { version = "0.1", features = ["s7comm", "manifest"] }
```

### 데이터 블록(DB) 읽기

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

### 알 수 없는 DB 발견 + 구조 탐지

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

### 블록 다운로드 + 플래시(PLC 프로그래밍)

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

### S7 manifest — 선언적 폴링 설정

레지스터 레이아웃을 하드코딩하는 대신, TOML로 전체 사이트를 기술합니다:

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

한 번의 명령으로 폴링할 수 있습니다(S7 스테이션은 병렬 `S7SensorPoller`가 폴링하며, 알람 규칙은 자동으로 `protocol = "s7comm"`으로 표시됩니다):

```bash
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### S7 필드 데이터 타입

| 타입 | 크기 | 오프셋 형식 | 디코딩 |
|------|------|----------|------|
| `BOOL` | 1 비트 | `8.0`(바이트 8, 비트 0) | 비트 테스트 |
| `BYTE` | 1 바이트 | `8` | `u8` |
| `WORD` | 2 바이트 | `8` | `u16::from_be_bytes` |
| `INT` | 2 바이트 | `8` | `i16::from_be_bytes` |
| `DWORD` | 4 바이트 | `8` | `u32::from_be_bytes` |
| `DINT` | 4 바이트 | `8` | `i32::from_be_bytes` |
| `REAL` | 4 바이트 | `0` | `f32::from_be_bytes` |
| `STRING` | 가변 길이 | `20` | 길이 접두 ASCII |

> `offset`의 소수 부분은 BOOL 필드의 **비트 인덱스**를 인코딩합니다. 예를 들어 `10.3` = 바이트 10, 비트 3.

### 하드웨어 없이 테스트

CI는 `snap7-server` 시뮬레이터(순수 Rust S7-1500 에뮬레이터)를 실행하여 연결 → DB 읽기 → DB 스캔 → 구조 탐지 → 블록 작업을 검증합니다:

```bash
cargo test --features full --test s7comm_integration
```

12개의 테스트가 통과합니다——이는 실제 엔드투엔드 프로토콜 상호작용이며, mock이 아닙니다.

> **⚠️ TIA Portal 사전 조건**: S7-1200/1500 PLC는 하드웨어 구성에서
> **"PUT/GET 접근 허용"**을 활성화해야 하며, DB는 **최적화되지 않은 블록 접근**을 사용해야 합니다(DB 우클릭 → 속성 →
> "최적화된 블록 접근" 체크 해제). 최적화된 DB는 바이트 단위 읽기 시 오류를 반환합니다.

---

## OPC UA(연결 탐지)

Evernight은 OPC UA 엔드포인트(TCP 포트 4840)를 탐지할 수 있지만, 아직 완전한 클라이언트는 제공하지 않습니다(Rust `opcua` crate의 클라이언트 기능이 아직 충분히 성숙하지 않음). 탐지로 엔드포인트를 발견하고, 백엔드가 출시되기 전까지는 전용 OPC UA 클라이언트를 사용하세요.

```bash
evernight probe 192.168.1.50 --ports 4840
```

---

## 발견 — 자율 프로토콜 식별

evernight을 알 수 없는 네트워크 범위나 시리얼 포트로 향하게 하면, 자동으로 프로토콜을 식별합니다:

```
TCP 엔드포인트                시리얼 버스
     │                         │
     ▼                         ▼
 ProtocolProbe 체인        보레이트 스캔 + 스테이션 번호 스캔
 (우선순위순)               (probe_modbus_rtu_baud)
     │                         │
     ▼                         ▼
 ProtocolProbeResult       { port, baud, protocol, stations[] }
 { protocol, confidence,
   banner }
```

탐지는 우선순위순으로 실행됩니다(Modbus=40이 S7comm=50보다 먼저); 신뢰도 임계값(기본 0.5) 이상의 첫 번째 일치가 승리합니다.

---

## 알람 라우팅(Modbus + S7comm)

센서 판독값은 공유 알람 파이프라인을 통해 흐릅니다. 각 프로토콜은 자체 topic 네임스페이스를 가지며, 하류 소비자(entelecheia, shittim-chest)는 출처별로 필터링할 수 있습니다:

| 프로토콜 | 트리거 topic | 출처 ID |
|------|-----------|---------|
| Modbus | `modbus.{스테이션 번호}.{필드}.{레벨}` | `evernight.modbus.{스테이션 번호}` |
| S7comm | `s7comm.{스테이션 번호}.{필드}.{레벨}` | `evernight.s7comm.{스테이션 번호}` |

알람 레벨은 ISA-18.2를 따릅니다: `ll` / `l` / `h` / `hh` / `roc`, 히스테레시스 데드밴드와 디바운스 카운트로 떨림을 방지합니다.

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

## CLI 명령 참조

| 명령 | 설명 |
|------|------|
| `evernight probe <host> [--ports 502,102,...]` | 호스트의 산업 프로토콜 탐지 |
| `evernight sensor-poll [--manifest X.toml]` | 센서 레지스터 폴링, entelecheia로 알람 전송 |
| `evernight file cp <로컬> <user@host:경로>` | SSH로 파일 업로드 |
| `evernight file get <user@host:경로> <로컬>` | SSH로 파일 다운로드 |
| `evernight file ls <user@host:경로>` | 원격 디렉터리 나열 |
| `evernight proxy <포트> --host <점프 호스트>` | SSH 동적 전달을 통한 로컬 SOCKS5 프록시 |
| `evernight exec --host X --command "..."` | 일회성 SSH 명령 실행 |
| `evernight hw` | 로컬 하드웨어 원격 측정 표시 |
| `evernight api-serve --transport ws` | JSON-RPC API 서버 시작 |

---

## 로드맵

- **MC Protocol**(미쓰비시) — 수작업 이진 프레임 코덱, 현재 Rust crate 없음. PLC 시장 커버리지 약 7% 증가.
- **EtherCAT**(벡호프 / 인보넌스) — `ethercrab` crate 경유.
- **EtherNet/IP + CIP**(록웰) — 클래스/인스턴스/속성 어드레싱.
- **OPC UA 클라이언트/서버** — `opcua` crate의 성숙도에 따라.
- **CAN 2.0B** — 연료 전지 USB-CAN 브리지 파싱.
