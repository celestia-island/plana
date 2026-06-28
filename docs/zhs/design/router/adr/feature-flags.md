# Feature Flag 架构

- **状态**：已接受
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

Evernight 的单体依赖图迫使所有消费者引入全部依赖（webrtc、russh、screenshots、sysinfo），即使他们只需要 SSH 或硬件遥测功能。这增加了下游用户的编译时间和二进制体积。

## 决策

使用 Cargo feature flags 将 crate 拆分为以下特性：

| Feature      | 控制范围                                      |
|--------------|----------------------------------------------|
| `screen`     | 屏幕捕获模块 + `screenshots` crate            |
| `webrtc`     | WebRTC 模块 + `webrtc` crate（隐含 `screen`） |
| `remote-ssh` | SSH 处理模块 + `russh` crate                  |
| `hardware`   | 硬件遥测模块 + `sysinfo` crate                |
| `protocol`   | 协议/消息类型（无重量级依赖）                   |
| `tunnel`     | TCP 隧道模块                                  |
| `full`       | 全部特性（默认）                               |

每个 feature 同时控制模块及其依赖。`webrtc` feature 隐含 `screen`，因为 WebRTC 会话需要屏幕捕获。

## 影响

### 积极影响

- 消费者只编译所需的功能
- 部分使用场景下编译时间显著减少

### 消极影响

- Feature flag 组合矩阵增大；必须在 CI 中测试每种特性组合
