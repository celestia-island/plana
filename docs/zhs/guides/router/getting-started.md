+++
title = "快速上手 — Evernight"
description = """evernight 快速上手 —— 构建、运行与首批命令。"""
lang = "zhs"
category = "guides"
subcategory = "router"
+++

# 快速上手 — Evernight

Evernight（长夜月）是一个用 Rust 编写的跨平台远程控制库和守护进程。它将屏幕捕获、WebRTC 推流、SSH 远程 shell、远程终端、文件传输、硬件遥测、工业协议支持（Modbus、S7comm、OPC-UA 探测）和 NAT 穿透整合到一个可复用的 crate 和独立的 CLI 二进制中。

## 前置条件

- Rust 1.85 或更高版本（2024 edition）
- 平台对应的 C 编译器（Windows 用 MSVC，Linux/macOS 用 GCC/Clang）
- 硬件遥测：`nvidia-smi`（NVIDIA GPU），Linux 上需要 `libudev`
- 工业协议：串口（`/dev/ttyUSB*`）或 PLC 的网络访问权限

## 编译

```bash
git clone https://github.com/celestia-island/evernight.git
cd evernight
cargo build --release
```

主二进制位于 `target/release/evernight`。

## 快速上手

CLI 使用子命令。运行 `evernight --help` 查看全部命令。

### SSH — 执行远程命令

```bash
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"
```

### SSH — 文件操作

```bash
# 上传本地文件到远程主机
evernight file cp ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# 下载远程文件
evernight file get root@192.168.1.100:/var/log/syslog ./syslog

# 列出远程目录
evernight file ls root@192.168.1.100:/etc/
```

### SSH — SOCKS5 代理

```bash
# 通过 SSH 跳板机启动本地 SOCKS5 代理（端口 1080）
evernight proxy 1080 --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519
```

### 硬件遥测

```bash
evernight hw
```

### 网络协议探测

```bash
# 探测主机的常见工业端口
evernight probe 192.168.1.20 --ports 502,102,4840,22
```

### 工业传感器轮询

```bash
# 从硬件 manifest 轮询传感器，向 entelecheia 发送告警
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### NAT 类型检测

```bash
evernight nat
```

### API 服务（基于 WebSocket 的 JSON-RPC）

```bash
evernight api-serve --transport ws --host 0.0.0.0 --port 50000
```

## Feature 开关

Evernight 采用 feature 开关，只编译你需要的部分：

```toml
[dependencies]
# 最小：SSH + 硬件遥测
evernight = { version = "0.1", features = ["remote-ssh", "hardware"] }

# 工业：Modbus + S7comm + manifest 支持
evernight = { version = "0.1", features = ["serial", "s7comm", "manifest"] }

# 全部（默认）
evernight = { version = "0.1", features = ["full"] }
```

| Feature | 启用内容 |
|---------|---------|
| `remote-ssh` | SSH 执行、文件传输、终端、端口转发、SOCKS5 代理 |
| `remote-vnc` | VNC（RFB）客户端 |
| `remote-rdp` | RDP 传输层骨架（TPKT/COTP/MCS） |
| `serial` | 串口 + Modbus RTU（通过 aoba） |
| `s7comm` | S7comm 客户端 + 块下载/烧录（通过 rust7 + snap7-client） |
| `protocol` | 协议探测 + ProtocolBackend trait 抽象 |
| `sensor` | 传感器轮询循环、告警评估、时序存储 |
| `manifest` | 硬件 manifest TOML/JSON schema + 运行时转换器 |
| `container` | Docker / Podman 容器管理 |
| `hardware` | CPU/GPU/内存/存储遥测 |
| `screen` | 屏幕捕获 + JPEG/VP9 编码 |
| `webrtc` | WebRTC 屏幕推流 |
| `tunnel` | TCP 端口转发 + NAT 穿透 |
| `api` | JSON-RPC 2.0 API 服务（ws/wss/ipc） |

## 核心功能

- **屏幕捕获** — 枚举显示器，捕获原始 RGBA 帧
- **WebRTC 推流** — DataChannel 上的 JPEG 或 VP9 视频轨；ICE/STUN 支持
- **SSH 远程 shell** — 通过 `russh` 执行命令、传输文件、打开终端
- **文件传输** — 通过 SSH 上传/下载，带进度回调
- **硬件遥测** — CPU、GPU、内存、存储、PCI 设备
- **工业协议** — Modbus RTU/TCP、S7comm（西门子）、OPC-UA 探测
- **传感器轮询** — 声明式 manifest 驱动的轮询循环，带 ISA-18.2 告警路由
- **TCP 隧道** — 本地/远程端口转发 + SOCKS5 动态转发
- **NAT 发现** — 基于 STUN 的 NAT 类型检测
- **API 服务** — 基于 WebSocket/IPC 的 JSON-RPC 2.0，供 Web 前端使用

## 后续步骤

- 阅读 **[工业协议接入指南](./protocols.md)** 了解 Modbus/S7comm 用法
- 运行 `evernight <命令> --help` 查看各命令选项
- 运行 `cargo doc --open` 查看完整 API 参考
- 运行集成测试验证环境（无需硬件）：
```bash
  cargo test --features full --test s7comm_integration    # S7comm 对 snap7-server
  cargo test --features full --test serial_integration    # Modbus 对虚拟串口
```
