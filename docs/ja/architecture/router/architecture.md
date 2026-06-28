+++
title = "Evernight アーキテクチャ"
description = """evernight —— クロスプラットフォーム遠隔制御ライブラリとデーモン：モジュールマップ、プロトコル層、接続モデル。"""
lang = "ja"
category = "architecture"
subcategory = "router"
+++

# Evernight アーキテクチャ

> **evernight** はクロスプラットフォーム遠隔制御ライブラリとデーモンである。
> celestia-island エコシステムの必須ハードウェア/プロトコル能力ブローカーで
> あり、上流の crate は物理デバイスと直接通信しない。

## 概要

| 能力 | モジュール | Feature |
|---|---|---|
| 画面キャプチャ（X11/DXGI/CoreGraphics） | `screen` | `screen` |
| WebRTC 画面ストリーミング | `stream` | `webrtc` |
| SSH リモートシェル + SFTP | `remote` | `remote-ssh` |
| VNC（RFB）クライアント | `vnc` | `remote-vnc` |
| RDP クライアント | `rdp` | `remote-rdp` |
| ハードウェアテレメトリ | `hardware` | `hardware` |
| 工業プロトコル | `protocol` | `protocol` / `s7comm` / `opcua` / `ethercat` |
| シリアル / Modbus | `serial`, `sensor` | `serial` |
| TCP トンネル + NAT 越え | `tunnel` | `tunnel` / `upnp` |
| 接続カタログ（URI） | `connection`, `connection_chain` | コア |
| 暗号化クレデンシャル保管庫 | `vault` | `vault` |
| コンテナ / K8s / libvirt | `container`, `vm_manager` | `container` / `k8s` / `libvirt` / `vm` |
| API サーバ（JSON-RPC） | `api` | `api` |

## 3 つの多態バックエンド trait

- `TerminalBackend` —— テキスト端末の読み/書き/resize（SSH、シリアル、Docker）
- `ViewportBackend` —— グラフィカルデスクトップの描画/入力（VNC、RDP、ローカル画面）
- `FileBackend` —— ファイル操作の list/get/put/rm（SFTP、shell、ローカル FS）

新しいトランスポートの追加はプラグイン挿入のみ —— 消費側は変更不要。

## プロトコル層

工業 I/O は 2 つの trait を通じて仲介される：

- `ProtocolBackend` —— connect / read / write / ping
- `ProtocolProbe` —— 未知エンドポイントのプロトコルを自動識別

```
ProtocolRegistry::auto_detect(transport)  →  ProtocolProbeResult
```

バックエンド：Modbus（aoba）、S7comm（rust7 + snap7-client）、MC Protocol、
EtherCAT（ethercrab）、EtherNet/IP + CIP、OPC UA（opcua crate、client + server）、
CAN（SocketCAN）。

### S7 自己組網（auto-provision）

evernight に生 IP を渡すだけで自己組網する：

```rust
use evernight::protocol::auto_provision;
let profile = auto_provision("192.168.1.10").await?;
```

プローブ → 接続 → DB スキャン → 構造プローブのパイプラインが
`S7DeviceProfile` を返し、シンボルの手入力はゼロ。一回限りの PLC 準備は
[TIA Portal 前提準備ガイド](../../guides/router/tia-portal-setup.md) 参照。

## 接続モデル

接続は URI 型付けされカタログ管理される：

```
ssh://user@host:22          s7://10.0.0.5?rack=0&slot=1
vnc://host:5900             opcua://10.0.0.5:4840
serial:///dev/ttyUSB0?baud=9600
```

`connection_chain` はターゲットを順序付きホップチェーン（汎用 ProxyJump）に解
決しトンネリングに使う。

## Feature flags

`full`（既定）ですべて有効化。各能力は独立にゲート可能で、最小依存ビルドに使
える —— 例えば `--features s7comm,serial` は工業プロトコル部分集合のみを出荷
する。
