+++
title = "画面キャプチャアーキテクチャ"
description = """アーキテクチャ意思決定記録 —— 画面キャプチャアーキテクチャ。"""
lang = "ja"
category = "design"
subcategory = "router"
+++

# 画面キャプチャアーキテクチャ

- **Status**: Accepted
- **Date**: 2025-06-09
- **Authors**: Evernight Core Team

## Context

Evernight は、WebRTC ストリーミングパイプラインにフレームを供給するクロスプラットフォーム画面キャプチャサブシステムを必要とする。このサブシステムは Windows、macOS、Linux 上で動作し、最小限のレイテンシで、GPU アクセラレーションとソフトウェアベースの両方のキャプチャパスをサポートしなければならない。

主要な制約:

- **レイテンシ予算**: キャプチャからエンコードまでのエンドツーエンドで 60 FPS 時に 16 ms 未満
- **可能な限りゼロコピー**: フレームは不要なコピーなしでエンコーダに到達すべきである
- **ホットプラグ対応**: ディスプレイと GPU は実行時に出現または消失する可能性がある
- **権限処理**: macOS は画面収録の許可が必要。Linux は XDG/X11 または Wayland プロトコルアクセスが必要

## Decision

**トレイトベースのキャプチャバックエンド**を採用し、Cargo フィーチャを介してコンパイル時にプラットフォーム固有の実装を選択する。各バックエンドは `FrameProvider` トレイトを実装する:

```rust
#[async_trait]
pub trait FrameProvider: Send + Sync {
    async fn enumerate_outputs(&self) -> Result<Vec<OutputInfo>>;
    async fn start_capture(&mut self, output: OutputId, config: CaptureConfig) -> Result<FrameReceiver>;
    async fn stop_capture(&mut self, output: OutputId) -> Result<()>;
}
```

### バックエンドの選択

| プラットフォーム | プライマリバックエンド             | フォールバック           |
|-----------------|-----------------------------------|--------------------------|
| Windows         | DXGI Desktop Duplication          | GDI BitBlt               |
| macOS           | ScreenCaptureKit (SCStream)       | CGWindowListCreateImage  |
| Linux           | PipeWire (xdg-desktop-portal 経由) | XShm / XFixes            |

### フレームライフサイクル

1. `FrameProvider::start_capture` は `FrameReceiver`（`Frame` 構造体を運ぶバウンド付き MPSC チャネル）を返す
2. 各 `Frame` は共有メモリバッファ（`Arc<FrameBuffer>`）を所有し、利用可能な場合は GPU メモリを参照する
3. WebRTC エンコーダはチャネルから消費する。すべての `Arc` 参照がドロップされると、バッファは再利用プールに戻される
4. キャプチャスレッドはエンコードをブロックしない — チャネルが満杯の場合、最も古いフレームがドロップされ、`FrameDropped` カウンタがインクリメントされる

### 色空間とフォーマット

- すべてのバックエンドは利用可能な最高ビット深度のフォーマット（BGRA8、NV12、または P010）をネゴシエートする
- `ColorSpaceConverter` ステップはエンコーダの優先フォーマットへの変換を処理する
- ソースが提供する場合、HDR メタデータは保存される

## Consequences

### Positive

- キャプチャとエンコードの明確な分離により、独立したテストが可能
- Windows（DXGI）と macOS（ScreenCaptureKit）でのゼロコピーパスにより、レイテンシを予算内に十分に収める
- トレイトベースの設計により、コアコードを変更せずにサードパーティバックエンド（例: 仮想ディスプレイ、テストソース）が可能
- フレームバッファプーリングにより、持続的なキャプチャ時の割り当て負荷が低減

### Negative

- Linux の PipeWire は D-Bus 依存を導入し、ヘッドレス/組み込みシナリオを複雑にする
- macOS の画面収録許可は初回起動時にユーザーの操作を必要とする — サイレントな回避策はない
- 4つのバックエンド実装を維持することでテスト対象が増加する

### リスクと緩和策

- **リスク**: ディストリビューション間での PipeWire API の変更。**緩和策**: 安定した `pw_stream` C API に固定し、Rust バインディングをベンダー化する。
- **リスク**: ハイブリッド GPU ラップトップでの DXGI スタッタリング。**緩和策**: 起動時に GPU トポロジを検出し、ディスクリート GPU がレンダリング中の場合、キャプチャには統合 GPU を優先する。

## References

- [DXGI Desktop Duplication API](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)
- [ScreenCaptureKit documentation](https://developer.apple.com/documentation/screencapturekit)
- [PipeWire screen cast portal](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org-freedesktop-portal-ScreenCast)
