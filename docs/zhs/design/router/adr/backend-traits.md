# TerminalBackend / ViewportBackend / FileBackend Trait 抽象

- **状态**：已采纳
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

Evernight 需要多态的后端接口，使前端（CLI、TUI、GUI）能够统一地消费任意协议。如果没有共享 trait，每个前端都需要为终端、图形显示和文件操作编写协议相关的代码路径。

## 决策

在 crate 根目录定义三个对象安全的异步 trait（始终可用，无需 feature flag）：

- **`TerminalBackend`** — `read` / `write` / `resize` / `close`
- **`ViewportBackend`** — `render` / `input` / `clipboard` / `close`
- **`FileBackend`** — `list` / `stat` / `get` / `put` / `rm` / `mkdir` / `rename`

每个协议后端实现相应的 trait。前端通过 `Box<dyn TerminalBackend>` 等方式消费。

## 影响

### 积极影响

- 前端与协议解耦；新增后端（如 RDP）无需修改前端代码。

### 消极影响

- 异步 trait object 需要 `Box::pin` 开销。
- trait 设计必须保持稳定，因为修改会破坏所有实现者。
