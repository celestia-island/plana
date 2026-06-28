# SSH 后端 -- russh 共享连接

- **状态**：已接受
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

Evernight 需要 SSH 来实现远程 Shell、文件操作和终端访问。最初代码库中存在三个独立的 SSH 处理器实现（FileHandler、SshHandler、TerminalHandler），连接逻辑重复。项目要求纯 Rust 实现（不使用 `Command::new("ssh")` 外部调用）。

## 决策

使用 `russh`（纯 Rust SSH-2 实现）作为 SSH 后端。将所有 SSH 处理器实现合并为 `remote/connection.rs` 中的单个 `DefaultSshHandler`，通过共享的 `connect_session()` 函数建立连接。所有 SSH 操作（Shell、文件、终端）共享此连接抽象。

## 影响

### 积极影响

- SSH 认证逻辑的单一来源
- 便于后续添加连接池

### 消极影响

- `russh` 在边缘场景下可能落后于 OpenSSH
- 尚未内置 SSH Agent 转发支持
