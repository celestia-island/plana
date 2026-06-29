+++
title = "屏幕捕获架构"
description = """架构决策记录 —— 屏幕捕获架构。"""
lang = "zhs"
category = "design"
subcategory = "router"
+++

# 屏幕捕获架构

- **状态**：已采纳
- **日期**：2025-06-09
- **作者**：Evernight 核心团队

## 背景

Evernight 需要一个跨平台的屏幕捕获子系统，为 WebRTC 流媒体管线提供视频帧。该子系统须在 Windows、macOS 和 Linux 上运行，延迟尽可能低，并同时支持 GPU 加速和软件捕获两种路径。

关键约束：

- **延迟预算**：从捕获到编码的端到端延迟在 60 FPS 下须控制在 16 ms 以内
- **尽量零拷贝**：帧数据到达编码器前应避免多余拷贝
- **热插拔支持**：显示器和 GPU 可能在运行时出现或消失
- **权限处理**：macOS 需要屏幕录制权限；Linux 需要 XDG/X11 或 Wayland 协议访问

## 决策

我们采用**基于 trait 的捕获后端**架构，通过 Cargo feature 在编译时选择平台相关实现。每个后端实现 `FrameProvider` trait：

```rust
#[async_trait]
pub trait FrameProvider: Send + Sync {
    async fn enumerate_outputs(&self) -> Result<Vec<OutputInfo>>;
    async fn start_capture(&mut self, output: OutputId, config: CaptureConfig) -> Result<FrameReceiver>;
    async fn stop_capture(&mut self, output: OutputId) -> Result<()>;
}
```

### 后端选择策略

| 平台    | 主后端                          | 备用后端                  |
|---------|--------------------------------|--------------------------|
| Windows | DXGI Desktop Duplication       | GDI BitBlt               |
| macOS   | ScreenCaptureKit (SCStream)    | CGWindowListCreateImage  |
| Linux   | PipeWire（通过 xdg-desktop-portal） | XShm / XFixes           |

### 帧生命周期

1. `FrameProvider::start_capture` 返回 `FrameReceiver`——一个携带 `Frame` 结构体的有界 MPSC 通道
2. 每个 `Frame` 持有共享内存缓冲区（`Arc<FrameBuffer>`），在可用时直接引用 GPU 显存
3. WebRTC 编码器从通道消费帧数据；当所有 `Arc` 引用释放后，缓冲区归还至复用池
4. 捕获线程不会因编码而阻塞——当通道已满时丢弃最旧的帧并递增 `FrameDropped` 计数器

### 色彩空间与格式

- 所有后端协商可用的最高位深格式（BGRA8、NV12 或 P010）
- `ColorSpaceConverter` 步骤负责转换至编码器偏好的格式
- 当源提供 HDR 元数据时予以保留

## 影响

### 积极影响

- 捕获与编码的清晰分离支持独立测试
- Windows（DXGI）和 macOS（ScreenCaptureKit）上的零拷贝路径可将延迟控制在预算以内
- 基于 trait 的设计允许第三方后端（如虚拟显示器、测试源）无需修改核心代码
- 帧缓冲池化降低持续捕获场景下的分配压力

### 消极影响

- Linux 上的 PipeWire 引入 D-Bus 依赖，在无头/嵌入式场景下增加复杂度
- macOS 屏幕录制权限在首次启动时需要用户交互——无法静默绕过
- 维护四套后端实现增加了测试范围

### 风险与缓解措施

- **风险**：不同发行版之间 PipeWire API 可能存在差异。**缓解**：绑定到稳定的 `pw_stream` C API 并 vendoring Rust 绑定。
- **风险**：混合 GPU 笔记本上 DXGI 可能出现卡顿。**缓解**：启动时检测 GPU 拓扑，当独立 GPU 负责渲染时优先使用集成显卡进行捕获。

## 参考资料

- [DXGI Desktop Duplication API](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)
- [ScreenCaptureKit 文档](https://developer.apple.com/documentation/screencapturekit)
- [PipeWire 屏幕录制 Portal](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org-freedesktop-portal-ScreenCast)
