+++
title = "Evernight 架构"
description = """evernight —— 跨平台远程控制库与守护进程：模块图、协议层、连接模型。"""
lang = "zhs"
category = "architecture"
subcategory = "router"
+++

# Evernight 架构

> **evernight** 是一个跨平台远程控制库与守护进程。它是 celestia-island 生态的
> 强制硬件/协议能力代理 —— 任何上游 crate 都不直接与物理设备通信。

## 概览

| 能力 | 模块 | Feature |
|---|---|---|
| 屏幕捕获（X11/DXGI/CoreGraphics） | `screen` | `screen` |
| WebRTC 屏幕推流 | `stream` | `webrtc` |
| SSH 远程 shell + SFTP | `remote` | `remote-ssh` |
| VNC（RFB）客户端 | `vnc` | `remote-vnc` |
| RDP 客户端 | `rdp` | `remote-rdp` |
| 硬件遥测 | `hardware` | `hardware` |
| 工业协议 | `protocol` | `protocol` / `s7comm` / `opcua` / `ethercat` |
| 串口 / Modbus | `serial`, `sensor` | `serial` |
| TCP 隧道 + NAT 穿透 | `tunnel` | `tunnel` / `upnp` |
| 连接目录（URI） | `connection`, `connection_chain` | 核心 |
| 加密凭据保险库 | `vault` | `vault` |
| 容器 / K8s / libvirt | `container`, `vm_manager` | `container` / `k8s` / `libvirt` / `vm` |
| API 服务端（JSON-RPC） | `api` | `api` |

## 三个多态后端 trait

以上一切都被三个与传输无关的 trait 抽象：

- `TerminalBackend` —— 文本终端的读/写/resize（SSH、串口、Docker）
- `ViewportBackend` —— 图形桌面的渲染/输入（VNC、RDP、本地屏幕）
- `FileBackend` —— 文件操作的 list/get/put/rm（SFTP、shell、本地 FS）

新增一种传输只是插入插件 —— 消费方无需改动。

## 协议层

工业 I/O 通过两个 trait 代理：

- `ProtocolBackend` —— connect / read / write / ping
- `ProtocolProbe` —— 自动识别未知端点的协议

```
ProtocolRegistry::auto_detect(transport)  →  ProtocolProbeResult
```

后端：Modbus（aoba）、S7comm（rust7 + snap7-client）、MC Protocol、EtherCAT
（ethercrab）、EtherNet/IP + CIP、OPC UA（opcua crate，client + server）、CAN
（SocketCAN）。

### S7 自组网（auto-provision）

给 evernight 一个裸 IP，它就自组网：

```rust
use evernight::protocol::auto_provision;
let profile = auto_provision("192.168.1.10").await?;
```

探测 → 连接 → 扫描 DB → 结构探测的流水线返回一个 `S7DeviceProfile`，零人工录
符号。一次性 PLC 准备见
[TIA Portal 前置准备指南](../../guides/router/tia-portal-setup.md)。

## 连接模型

连接是 URI 类型化、目录化管理的：

```
ssh://user@host:22          s7://10.0.0.5?rack=0&slot=1
vnc://host:5900             opcua://10.0.0.5:4840
serial:///dev/ttyUSB0?baud=9600
```

`connection_chain` 把目标解析成有序跳板链（泛化的 ProxyJump）用于隧道。

## Feature flags

`full`（默认）启用全部。每个能力可独立门控，用于最小依赖构建 —— 例如
`--features s7comm,serial` 只出货工业协议子集。
