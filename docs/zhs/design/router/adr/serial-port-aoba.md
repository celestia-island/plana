# 通过 aoba 进行串口通信

- **状态**：已采纳
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

Evernight 需要串口支持，用于嵌入式设备管理和工业协议（Modbus RTU）探测。兄弟 crate `aoba` 已提供跨平台串口枚举、VID/PID/序列号提取以及 Modbus RTU/TCP 主站功能。

## 决策

将所有串口操作委托给 aoba。Evernight 的 `serial` 模块定义类型（`SerialConfig`、`SerialPortInfo`），并通过串口传输实现 `TerminalBackend` trait，调用 aoba 执行实际的端口 I/O。协议自动检测（Modbus RTU 波特率/校验位扫描）同样委托给 aoba。

## 影响

### 积极影响

- 复用经过验证的代码；aoba 处理跨平台边界情况。

### 消极影响

- 增加了对 aoba 的依赖；serial feature 需要 aoba 可用。
