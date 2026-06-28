+++
title = "Evernight 架構"
description = """evernight —— 跨平台遠端控制函式庫與常駐程式：模組圖、協定層、連線模型。"""
lang = "zht"
category = "architecture"
subcategory = "router"
+++

# Evernight 架構

> **evernight** 是一個跨平台遠端控制函式庫與常駐程式。它是 celestia-island 生態的
> 強制硬體/協定能力代理 —— 任何上游 crate 都不直接與實體裝置通訊。

## 概覽

| 能力 | 模組 | Feature |
|---|---|---|
| 螢幕擷取（X11/DXGI/CoreGraphics） | `screen` | `screen` |
| WebRTC 螢幕串流 | `stream` | `webrtc` |
| SSH 遠端 shell + SFTP | `remote` | `remote-ssh` |
| VNC（RFB）客戶端 | `vnc` | `remote-vnc` |
| RDP 客戶端 | `rdp` | `remote-rdp` |
| 硬體遙測 | `hardware` | `hardware` |
| 工業協定 | `protocol` | `protocol` / `s7comm` / `opcua` / `ethercat` |
| 序列埠 / Modbus | `serial`, `sensor` | `serial` |
| TCP 隧道 + NAT 穿透 | `tunnel` | `tunnel` / `upnp` |
| 連線目錄（URI） | `connection`, `connection_chain` | 核心 |
| 加密憑證保險庫 | `vault` | `vault` |
| 容器 / K8s / libvirt | `container`, `vm_manager` | `container` / `k8s` / `libvirt` / `vm` |
| API 伺服器（JSON-RPC） | `api` | `api` |

## 三個多態後端 trait

- `TerminalBackend` —— 文字終端的讀/寫/resize（SSH、序列埠、Docker）
- `ViewportBackend` —— 圖形桌面的渲染/輸入（VNC、RDP、本機螢幕）
- `FileBackend` —— 檔案操作的 list/get/put/rm（SFTP、shell、本機 FS）

新增一種傳輸只是插入插件 —— 消費方無需改動。

## 協定層

工業 I/O 透過兩個 trait 代理：

- `ProtocolBackend` —— connect / read / write / ping
- `ProtocolProbe` —— 自動識別未知端點的協定

```
ProtocolRegistry::auto_detect(transport)  →  ProtocolProbeResult
```

後端：Modbus（aoba）、S7comm（rust7 + snap7-client）、MC Protocol、EtherCAT
（ethercrab）、EtherNet/IP + CIP、OPC UA（opcua crate，client + server）、CAN
（SocketCAN）。

### S7 自組網（auto-provision）

給 evernight 一個裸 IP，它就自組網：

```rust
use evernight::protocol::auto_provision;
let profile = auto_provision("192.168.1.10").await?;
```

探測 → 連線 → 掃描 DB → 結構探測的流水線回傳一個 `S7DeviceProfile`，零人工錄
符號。一次性 PLC 準備見
[TIA Portal 前置準備指南](../../guides/router/tia-portal-setup.md)。

## 連線模型

連線是 URI 型別化、目錄化管理的：

```
ssh://user@host:22          s7://10.0.0.5?rack=0&slot=1
vnc://host:5900             opcua://10.0.0.5:4840
serial:///dev/ttyUSB0?baud=9600
```

`connection_chain` 把目標解析成有序跳板鏈（泛化的 ProxyJump）用於隧道。

## Feature flags

`full`（預設）啟用全部。每個能力可獨立門控，用於最小相依建置 —— 例如
`--features s7comm,serial` 只出貨工業協定子集。
