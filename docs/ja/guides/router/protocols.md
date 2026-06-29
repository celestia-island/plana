+++
title = "産業プロトコル接続ガイド — Evernight"
description = """evernight の工業プロトコル統合ガイド（Modbus、S7comm、OPC UA）。"""
lang = "ja"
category = "guides"
subcategory = "router"
+++

# 産業プロトコル接続ガイド — Evernight

Evernight は celestia-island エコシステムの**強制ハードウェア能力ブローカー（hardware capability broker）**です。上流 crate はすべて `aoba` / `rust7` などのライブラリを直接インポートしてはならず、物理 I/O はすべて evernight のプロトコルモジュールを経由する必要があります。本ガイドでは、対応する各産業プロトコルの接続、ポーリング、発見、アラームの方法を説明します。

## 一覧

| プロトコル | トランスポート層 | 状態 | ポート / バス | カバレッジ |
|------|--------|------|------------|----------|
| **Modbus RTU** | シリアル（RS-485） | ✅ 本番対応 | `/dev/ttyUSB*` | 中国 PLC 市場の約 70% を占有 |
| **Modbus TCP** | TCP | ✅ 本番対応 | 502 | 工場レベル SCADA |
| **S7comm**（シーメンス） | TCP | ✅ 本番対応 | 102 | S7-1200/1500/300/400 |
| **MC Protocol**（三菱） | TCP | 🚧 計画中 | 5000 | MELSEC FX/Q/L/iQ-R |
| **OPC UA** | TCP | ⏳ プローブのみ | 4840 | "万能トランスレータ" |
| **EtherCAT**（ベッコフ） | イーサネット | ⏳ 計画中 | — | サーボ / モーション |
| **EtherNet/IP**（ロックウェル） | TCP/UDP | ⏳ 計画中 | 44818/2222 | Allen-Bradley |
| **CAN 2.0B** | シリアル | ⏳ 計画中 | `/dev/ttyUSBCAN` | 燃料電池 |

> **状態列の説明**：✅ = CI 上でシミュレータにより検証済み；🚧 = 設計完了、実装進行中；⏳ = プローブ／接続性チェックのみ、または計画中。

## アーキテクチャ

```
                         ┌──────────────────────────────────┐
   あなたのアプリ ────────►│         evernight crate           │
   (CLI / ライブラリ /      │                                   │
    sensor-poll /          │  ┌─────────────────────────────┐ │
    API サービス)          │  │   ProtocolBackend trait      │ │
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
                              │     物理ハードウェア   │
                              │  PLC / センサ / バルブ │
                              └───────────────────────┘
```

各プロトコルは同じ 2 つの trait を実装するため、新しいプロトコルの追加は**プラグイン式**です。上流のコンシューマー（センサポーラ、発見ツール、CLI）を変更する必要はありません：

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

## Modbus（シリアル RTU / TCP）

Modbus は産業通信の主力です。Evernight は `aoba` crate を `evernight::serial::modbus::ModbusMaster` と `evernight::protocol::ModbusBackend` の背後にカプセル化しています。

### Feature スイッチ

```toml
[dependencies]
evernight = { version = "0.1", features = ["serial", "protocol"] }
```

- `serial` — `ModbusMaster`、レジスタの読み書き、ボーレート自動検出
- `protocol` — `ModbusBackend`（trait 実装）+ `ModbusProbe`（TCP 自動検出）

### クイックスタート — コードでレジスタを読み取る

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

### ボーレートの自動検出

デバイスのボーレートが不明な場合は、一般的なレートをスキャンできます：

```rust
use evernight::serial::probe_modbus_rtu_baud;

let detected = probe_modbus_rtu_baud("/dev/ttyUSB1", 0x13)?;
println!("设备在 {} 波特率下响应", detected.baud);
```

### Modbus TCP — プローブ + 自動識別

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

### CLI — ホストの Modbus をプローブ

```bash
# 探测常见工业端口（502=Modbus, 102=S7comm, …）
evernight probe 192.168.1.20 --ports 502,102,4840
```

### ハードウェアなしのテスト

Evernight の CI は仮想シリアルポート（`socat`）と aoba の Modbus スレーブを組み合わせて使用します。統合テストを実行します：

```bash
cargo test --features full --test serial_integration
```

これらのテストは仮想 TTY のペアを開き、実際の Modbus スレーブループを実行して、読み書き／ボーレートスキャンを検証します。物理ハードウェアは不要です。

---

## S7comm（シーメンス S7-1200/1500/300/400）

S7comm はシーメンスのネイティブプロトコルで、ISO-on-TCP（ポート 102）に基づいています。Evernight は `rust7` crate（純 Rust、FFI なし）でデータアクセスを行い、`snap7-client` でブロックのダウンロード／フラッシュを行います。

### Feature スイッチ

```toml
[dependencies]
evernight = { version = "0.1", features = ["s7comm", "manifest"] }
```

### データブロック（DB）の読み取り

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

### 未知 DB の発見 + 構造プローブ

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

### ブロックのダウンロード + フラッシュ（PLC プログラミング）

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

### S7 manifest — 宣言型ポーリング設定

レジスタレイアウトをハードコードする代わりに、TOML でサイト全体を記述します：

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

1 つのコマンドでポーリングできます（S7 ステーションは並列の `S7SensorPoller` によってポーリングされ、アラームルールには自動的に `protocol = "s7comm"` がタグ付けされます）：

```bash
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### S7 フィールドのデータ型

| 型 | サイズ | オフセット形式 | デコード |
|------|------|----------|------|
| `BOOL` | 1 ビット | `8.0`（バイト 8，ビット 0） | ビットテスト |
| `BYTE` | 1 バイト | `8` | `u8` |
| `WORD` | 2 バイト | `8` | `u16::from_be_bytes` |
| `INT` | 2 バイト | `8` | `i16::from_be_bytes` |
| `DWORD` | 4 バイト | `8` | `u32::from_be_bytes` |
| `DINT` | 4 バイト | `8` | `i32::from_be_bytes` |
| `REAL` | 4 バイト | `0` | `f32::from_be_bytes` |
| `STRING` | 可変長 | `20` | 長さプレフィックス ASCII |

> `offset` の小数部分は BOOL フィールドの**ビットインデックス**をエンコードします。例えば `10.3` = バイト 10，ビット 3。

### ハードウェアなしのテスト

CI は `snap7-server` シミュレータ（純 Rust S7-1500 エミュレータ）を実行し、接続 → DB 読み取り → DB スキャン → 構造プローブ → ブロック操作を検証します：

```bash
cargo test --features full --test s7comm_integration
```

12 個のテストがパスします。これは実際のエンドツーエンドのプロトコル相互作用であり、mock ではありません。

> **⚠️ TIA Portal の前提条件**：S7-1200/1500 PLC ではハードウェア構成で
> **"PUT/GET アクセスを許可"** を有効化する必要があり、DB は**非最適化ブロックアクセス**を
> 使用しなければなりません（DB を右クリック → プロパティ → "最適化されたブロックアクセス" の
> チェックを外す）。最適化 DB はバイトレベルの読み取りでエラーを返します。

---

## OPC UA（接続性プローブ）

Evernight は OPC UA エンドポイント（TCP ポート 4840）を検出できますが、完全なクライアントはまだ提供していません（Rust の `opcua` crate のクライアント機能はまだ十分ではありません）。プローブでエンドポイントを発見し、バックエンドが利用可能になるまでは専用の OPC UA クライアントを使用します。

```bash
evernight probe 192.168.1.50 --ports 4840
```

---

## 発見 — 自律的なプロトコル識別

evernight を未知のネットワーク範囲やシリアルポートに向けると、プロトコルを自動的に識別します：

```
TCP エンドポイント          シリアルバス
     │                         │
     ▼                         ▼
 ProtocolProbe チェーン     ボーレートスキャン + ステーション番号スキャン
 （優先度順）               (probe_modbus_rtu_baud)
     │                         │
     ▼                         ▼
 ProtocolProbeResult       { port, baud, protocol, stations[] }
 { protocol, confidence,
   banner }
```

プローブは優先度順に実行され（Modbus=40 が S7comm=50 より先）、信頼度しきい値（デフォルト 0.5）を上回る最初の一致が勝者となります。

---

## アラームルーティング（Modbus + S7comm）

センサの読み取り値は共有のアラームパイプラインを流れます。各プロトコルには独自の topic 名前空間があり、下流のコンシューマー（entelecheia、shittim-chest）はソースごとにフィルタリングできます：

| プロトコル | トリガ topic | ソース ID |
|------|-----------|---------|
| Modbus | `modbus.{ステーション番号}.{フィールド}.{レベル}` | `evernight.modbus.{ステーション番号}` |
| S7comm | `s7comm.{ステーション番号}.{フィールド}.{レベル}` | `evernight.s7comm.{ステーション番号}` |

アラームレベルは ISA-18.2 に従います：`ll` / `l` / `h` / `hh` / `roc`。チャタリングを防ぐため、ヒステリシスデッドバンドとデバウンスカウントを備えています。

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

## CLI コマンドリファレンス

| コマンド | 説明 |
|------|------|
| `evernight probe <host> [--ports 502,102,...]` | ホストの産業プロトコルをプローブ |
| `evernight sensor-poll [--manifest X.toml]` | センサーレジスタをポーリングし、entelecheia にアラームを送信 |
| `evernight file cp <ローカル> <user@host:パス>` | SSH でファイルをアップロード |
| `evernight file get <user@host:パス> <ローカル>` | SSH でファイルをダウンロード |
| `evernight file ls <user@host:パス>` | リモートディレクトリを一覧表示 |
| `evernight proxy <ポート> --host <踏み台>` | SSH 動的転送によるローカル SOCKS5 プロキシ |
| `evernight exec --host X --command "..."` | 一回限りの SSH コマンドを実行 |
| `evernight hw` | ローカルハードウェアテレメトリを表示 |
| `evernight api-serve --transport ws` | JSON-RPC API サービスを起動 |

---

## ロードマップ

- **MC Protocol**（三菱）— 手書きのバイナリフレームコーデック。現状 Rust crate はなし。PLC 市場の約 7% のカバレッジを追加。
- **EtherCAT**（ベッコフ / 汇川）— `ethercrab` crate 経由。
- **EtherNet/IP + CIP**（ロックウェル）— クラス／インスタンス／属性アドレッシング。
- **OPC UA クライアント／サーバー** — `opcua` crate の成熟度次第。
- **CAN 2.0B** — 燃料電池の USB-CAN ブリッジ解析。
