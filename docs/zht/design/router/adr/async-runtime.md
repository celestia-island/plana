# 非同步執行環境 — tokio

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

Evernight 需要一個非同步執行環境，用於網路 I/O（SSH、WebRTC、TCP 通道）、基於計時器的操作（幀擷取迴圈、協定逾時）以及並行任務管理。Rust 生態系統提供了 tokio、async-std 和 smol。

## 決策

使用 `tokio` 作為唯一的非同步執行環境。tokio 是 Rust 非同步生態系統中的事實標準，擁有最豐富的驅動程式支援。關鍵相依套件（`russh`、`webrtc`、`reqwest`）已經需要 tokio。使用 `tokio::runtime::Handle::current()` 從同步上下文中生成任務。

## 後果

### 正面

- 生態系統相容性；無需執行環境橋接

### 負面

- tokio 是一個重量級相依套件
- 無法使用不支援 tokio 的 async-std 或 smol 套件
