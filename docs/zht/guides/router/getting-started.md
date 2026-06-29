+++
title = "快速上手 — Evernight"
description = """evernight 快速上手 —— 建置、執行與首批指令。"""
lang = "zht"
category = "guides"
subcategory = "router"
+++

# 快速上手 — Evernight

Evernight（長夜月）是一套以 Rust 撰寫的跨平臺遠端控制函式庫與常駐服務。它將螢幕擷取、WebRTC 推流、SSH 遠端 shell、遠端終端機、檔案傳輸、硬體遙測、工業協定支援（Modbus、S7comm、OPC-UA 探測）與 NAT 穿透整合到一個可重複使用的 crate 與獨立的 CLI 二進位中。

## 前置條件

- Rust 1.85 或更高版本（2024 edition）
- 對應平臺的 C 編譯器（Windows 使用 MSVC，Linux/macOS 使用 GCC/Clang）
- 硬體遙測：`nvidia-smi`（NVIDIA GPU），Linux 上需要 `libudev`
- 工業協定：序列埠（`/dev/ttyUSB*`）或 PLC 的網路存取權限

## 編譯

```bash
git clone https://github.com/celestia-island/evernight.git
cd evernight
cargo build --release
```

主二進位位於 `target/release/evernight`。

## 快速上手

CLI 使用子命令。執行 `evernight --help` 查看全部命令。

### SSH — 執行遠端命令

```bash
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"
```

### SSH — 檔案操作

```bash
# 上傳本機檔案到遠端主機
evernight file cp ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# 下載遠端檔案
evernight file get root@192.168.1.100:/var/log/syslog ./syslog

# 列出遠端目錄
evernight file ls root@192.168.1.100:/etc/
```

### SSH — SOCKS5 代理

```bash
# 透過 SSH 跳板機啟動本機 SOCKS5 代理（連接埠 1080）
evernight proxy 1080 --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519
```

### 硬體遙測

```bash
evernight hw
```

### 網路協定探測

```bash
# 探測主機的常見工業連接埠
evernight probe 192.168.1.20 --ports 502,102,4840,22
```

### 工業感測器輪詢

```bash
# 從硬體 manifest 輪詢感測器，向 entelecheia 傳送警示
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### NAT 型態偵測

```bash
evernight nat
```

### API 服務（基於 WebSocket 的 JSON-RPC）

```bash
evernight api-serve --transport ws --host 0.0.0.0 --port 50000
```

## Feature 開關

Evernight 採用 feature 開關，只編譯您需要的部分：

```toml
[dependencies]
# 最小：SSH + 硬體遙測
evernight = { version = "0.1", features = ["remote-ssh", "hardware"] }

# 工業：Modbus + S7comm + manifest 支援
evernight = { version = "0.1", features = ["serial", "s7comm", "manifest"] }

# 全部（預設）
evernight = { version = "0.1", features = ["full"] }
```

| Feature | 啟用內容 |
|---------|---------|
| `remote-ssh` | SSH 執行、檔案傳輸、終端機、連接埠轉送、SOCKS5 代理 |
| `remote-vnc` | VNC（RFB）用戶端 |
| `remote-rdp` | RDP 傳輸層骨架（TPKT/COTP/MCS） |
| `serial` | 序列埠 + Modbus RTU（透過 aoba） |
| `s7comm` | S7comm 用戶端 + 區塊下載/燒錄（透過 rust7 + snap7-client） |
| `protocol` | 協定探測 + ProtocolBackend trait 抽象 |
| `sensor` | 感測器輪詢迴圈、警示評估、時序儲存 |
| `manifest` | 硬體 manifest TOML/JSON schema + 執行時期轉換器 |
| `container` | Docker / Podman 容器管理 |
| `hardware` | CPU/GPU/記憶體/儲存遙測 |
| `screen` | 螢幕擷取 + JPEG/VP9 編碼 |
| `webrtc` | WebRTC 螢幕推流 |
| `tunnel` | TCP 連接埠轉送 + NAT 穿透 |
| `api` | JSON-RPC 2.0 API 服務（ws/wss/ipc） |

## 核心功能

- **螢幕擷取** — 列舉顯示器，擷取原始 RGBA 幀
- **WebRTC 推流** — DataChannel 上的 JPEG 或 VP9 影片軌；ICE/STUN 支援
- **SSH 遠端 shell** — 透過 `russh` 執行命令、傳輸檔案、開啟終端機
- **檔案傳輸** — 透過 SSH 上傳/下載，附帶進度回呼
- **硬體遙測** — CPU、GPU、記憶體、儲存、PCI 裝置
- **工業協定** — Modbus RTU/TCP、S7comm（西門子）、OPC-UA 探測
- **感測器輪詢** — 宣告式 manifest 驅動的輪詢迴圈，附帶 ISA-18.2 警示路由
- **TCP 隧道** — 本機/遠端連接埠轉送 + SOCKS5 動態轉送
- **NAT 探索** — 基於 STUN 的 NAT 型態偵測
- **API 服務** — 基於 WebSocket/IPC 的 JSON-RPC 2.0，供 Web 前端使用

## 後續步驟

- 閱讀 **[工業協定接入指南](./protocols.md)** 瞭解 Modbus/S7comm 用法
- 執行 `evernight <命令> --help` 查看各命令選項
- 執行 `cargo doc --open` 查看完整 API 參考
- 執行整合測試驗證環境（無需硬體）：
```bash
  cargo test --features full --test s7comm_integration    # S7comm 對 snap7-server
  cargo test --features full --test serial_integration    # Modbus 對虛擬序列埠
```
