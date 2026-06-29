+++
title = "クイックスタート — Evernight"
description = """evernight のはじめ方 —— ビルド、実行、最初のコマンド。"""
lang = "ja"
category = "guides"
subcategory = "router"
+++

# クイックスタート — Evernight

Evernight（長夜月）は Rust で書かれたクロスプラットフォーム対応のリモート制御ライブラリ兼デーモンです。スクリーンキャプチャ、WebRTC ストリーミング、SSH リモートシェル、リモートターミナル、ファイル転送、ハードウェアテレメトリ、産業用プロトコルサポート（Modbus、S7comm、OPC-UA プローブ）、NAT トラバーサルを、再利用可能なクレートとスタンドアロン CLI バイナリに統合しています。

## 前提条件

- Rust 1.85 以降（2024 エディション）
- プラットフォームに対応する C コンパイラ（Windows は MSVC、Linux/macOS は GCC/Clang）
- ハードウェアテレメトリ：`nvidia-smi`（NVIDIA GPU）、Linux では `libudev` が必要
- 産業用プロトコル：シリアルポート（`/dev/ttyUSB*`）または PLC へのネットワークアクセス権限

## ビルド

```bash
git clone https://github.com/celestia-island/evernight.git
cd evernight
cargo build --release
```

メインバイナリは `target/release/evernight` に出力されます。

## クイックスタート

CLI はサブコマンド方式を採用しています。すべてのコマンドは `evernight --help` で確認できます。

### SSH — リモートコマンドの実行

```bash
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"
```

### SSH — ファイル操作

```bash
# ローカルファイルをリモートホストへアップロード
evernight file cp ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# リモートファイルをダウンロード
evernight file get root@192.168.1.100:/var/log/syslog ./syslog

# リモートディレクトリの一覧表示
evernight file ls root@192.168.1.100:/etc/
```

### SSH — SOCKS5 プロキシ

```bash
# SSH 踏み台経由でローカル SOCKS5 プロキシを起動（ポート 1080）
evernight proxy 1080 --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519
```

### ハードウェアテレメトリ

```bash
evernight hw
```

### ネットワークプロトコルプローブ

```bash
# ホストの一般的な産業用ポートをプローブ
evernight probe 192.168.1.20 --ports 502,102,4840,22
```

### 産業用センサーのポーリング

```bash
# ハードウェアマニフェストからセンサーをポーリングし、entelecheia へアラートを送信
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### NAT タイプ検出

```bash
evernight nat
```

### API サービス（WebSocket ベースの JSON-RPC）

```bash
evernight api-serve --transport ws --host 0.0.0.0 --port 50000
```

## フィーチャーフラグ

Evernight はフィーチャーフラグを採用しており、必要な部分だけをコンパイルできます：

```toml
[dependencies]
# 最小構成：SSH + ハードウェアテレメトリ
evernight = { version = "0.1", features = ["remote-ssh", "hardware"] }

# 産業用：Modbus + S7comm + マニフェストサポート
evernight = { version = "0.1", features = ["serial", "s7comm", "manifest"] }

# すべて（デフォルト）
evernight = { version = "0.1", features = ["full"] }
```

| フィーチャー | 有効になる内容 |
|---------|---------|
| `remote-ssh` | SSH 実行、ファイル転送、ターミナル、ポートフォワード、SOCKS5 プロキシ |
| `remote-vnc` | VNC（RFB）クライアント |
| `remote-rdp` | RDP トランスポート層スケルトン（TPKT/COTP/MCS） |
| `serial` | シリアルポート + Modbus RTU（aoba 経由） |
| `s7comm` | S7comm クライアント + ブロックダウンロード／書き込み（rust7 + snap7-client 経由） |
| `protocol` | プロトコルプローブ + ProtocolBackend trait 抽象化 |
| `sensor` | センサーのポーリングループ、アラーム評価、時系列ストレージ |
| `manifest` | ハードウェアマニフェスト TOML/JSON スキーマ + ランタイムコンバータ |
| `container` | Docker / Podman コンテナ管理 |
| `hardware` | CPU/GPU/メモリ/ストレージのテレメトリ |
| `screen` | スクリーンキャプチャ + JPEG/VP9 エンコード |
| `webrtc` | WebRTC スクリーンストリーミング |
| `tunnel` | TCP ポートフォワード + NAT トラバーサル |
| `api` | JSON-RPC 2.0 API サービス（ws/wss/ipc） |

## コア機能

- **スクリーンキャプチャ** — ディスプレイの列挙、生 RGBA フレームのキャプチャ
- **WebRTC ストリーミング** — DataChannel 上の JPEG または VP9 映像トラック、ICE/STUN サポート
- **SSH リモートシェル** — `russh` 経由でのコマンド実行、ファイル転送、ターミナルオープン
- **ファイル転送** — SSH 経由のアップロード／ダウンロード、進捗コールバック付き
- **ハードウェアテレメトリ** — CPU、GPU、メモリ、ストレージ、PCI デバイス
- **産業用プロトコル** — Modbus RTU/TCP、S7comm（シーメンス）、OPC-UA プローブ
- **センサーポーリング** — 宣言型マニフェスト駆動のポーリングループ、ISA-18.2 アラームルーティング付き
- **TCP トンネル** — ローカル／リモートポートフォワード + SOCKS5 動的フォワード
- **NAT 検出** — STUN ベースの NAT タイプ検出
- **API サービス** — WebSocket/IPC ベースの JSON-RPC 2.0、Web フロントエンド向け

## 次のステップ

- **[産業用プロトコル接続ガイド](./protocols.md)** を読んで Modbus/S7comm の使い方を理解する
- 各コマンドのオプションは `evernight <コマンド> --help` で確認
- 完全な API リファレンスは `cargo doc --open` で参照
- 環境を検証するための統合テストを実行（ハードウェア不要）：
```bash
  cargo test --features full --test s7comm_integration    # snap7-server に対する S7comm
  cargo test --features full --test serial_integration    # 仮想シリアルポートに対する Modbus
```
