# 螢幕擷取架構

- **狀態**：已接受
- **日期**：2025-06-09
- **作者**：Evernight Core Team

## 背景

Evernight 需要一個跨平台的螢幕擷取子系統，將幀饋送到 WebRTC 串流管線中。該子系統必須在 Windows、macOS 和 Linux 上運作，具有最低的延遲，並支援 GPU 加速和基於軟體的擷取路徑。

關鍵約束：

- **延遲預算**：端到端的擷取到編碼必須在 60 FPS 下保持在 16 毫秒以內
- **盡可能零拷貝**：幀應在不進行不必要拷貝的情況下送達編碼器
- **熱插拔支援**：顯示器和 GPU 可能在執行期間出現或消失
- **權限處理**：macOS 需要螢幕錄製權限；Linux 需要 XDG/X11 或 Wayland 協定存取

## 決策

我們採用**基於 trait 的擷取後端**，透過 Cargo 功能在編譯時期選擇平台特定的實作。每個後端實作 `FrameProvider` trait：

```rust
#[async_trait]
pub trait FrameProvider: Send + Sync {
    async fn enumerate_outputs(&self) -> Result<Vec<OutputInfo>>;
    async fn start_capture(&mut self, output: OutputId, config: CaptureConfig) -> Result<FrameReceiver>;
    async fn stop_capture(&mut self, output: OutputId) -> Result<()>;
}
```

### 後端選擇

| 平台     | 主要後端                         | 備援方案                 |
|----------|--------------------------------|--------------------------|
| Windows  | DXGI Desktop Duplication       | GDI BitBlt               |
| macOS    | ScreenCaptureKit (SCStream)    | CGWindowListCreateImage  |
| Linux    | PipeWire（透過 xdg-desktop-portal） | XShm / XFixes            |

### 幀生命週期

1. `FrameProvider::start_capture` 回傳一個 `FrameReceiver`——一個攜帶 `Frame` 結構體的有界 MPSC 通道
2. 每個 `Frame` 擁有一個共享記憶體緩衝區（`Arc<FrameBuffer>`），在可用時引用 GPU 記憶體
3. WebRTC 編碼器從通道中消費；當所有 `Arc` 引用釋放後，緩衝區會返回到重用池
4. 擷取執行緒絕不會因編碼而阻塞——如果通道已滿，最舊的幀會被丟棄，並且 `FrameDropped` 計數器會遞增

### 色彩空間與格式

- 所有後端協商可用的最高位深度格式（BGRA8、NV12 或 P010）
- `ColorSpaceConverter` 步驟處理到編碼器偏好格式的轉換
- 當來源提供 HDR 中繼資料時，會予以保留

## 後果

### 正面

- 擷取與編碼之間的清晰分離允許獨立測試
- Windows（DXGI）和 macOS（ScreenCaptureKit）上的零拷貝路徑使延遲完全在預算範圍內
- 基於 trait 的設計允許第三方後端（例如虛擬顯示器、測試來源）而無需修改核心程式碼
- 幀緩衝區池化減少了持續擷取下的分配壓力

### 負面

- Linux 上的 PipeWire 引入了 D-Bus 相依，使無頭／嵌入式場景變得複雜
- macOS 螢幕錄製權限在首次啟動時需要使用者互動——沒有靜默的解決方法
- 維護四個後端實作增加了測試範圍

### 風險與緩解措施

- **風險**：各發行版之間的 PipeWire API 變更。**緩解措施**：固定到穩定的 `pw_stream` C API，並將 Rust 綁定納入倉庫。
- **風險**：混合 GPU 筆記型電腦上的 DXGI 卡頓。**緩解措施**：在啟動時偵測 GPU 拓撲，當獨立 GPU 正在渲染時，優先使用整合 GPU 進行擷取。

## 參考資料

- [DXGI Desktop Duplication API](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)
- [ScreenCaptureKit 文件](https://developer.apple.com/documentation/screencapturekit)
- [PipeWire 螢幕投影入口](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org-freedesktop-portal-ScreenCast)
