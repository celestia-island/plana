+++
title = "CLI 日志规范"
description = """shittim-chest CLI 封装器的日志输出遵循与 entelecheia 一致的规范，使用 `tracing` 生态，以紧凑的人类可读格式输出到 stderr。"""
lang = "zhs"
category = "design"
subcategory = "webui"
+++

# CLI 日志规范

## 概述

shittim-chest CLI 封装器的日志输出遵循与 entelecheia 一致的规范，使用 `tracing` 生态，以紧凑的人类可读格式输出到 stderr。

## 框架选择

| 组件 | 选择 | 原因 |
| --- | --- | --- |
| 日志框架 | `tracing` | Rust 生态标准，与 entelecheia 一致 |
| Subscriber | `tracing-subscriber` fmt layer | 紧凑输出，无需 JSON 解析 |
| 时间格式 | `ShortTimer`（HH:MM:SS） | 终端友好，与 entelecheia CLI 一致 |
| 输出目标 | stderr | 与 stdout 分离，不干扰管道操作 |

## 初始化代码

```rust
use chrono::Local;
use tracing_subscriber::fmt::time::FormatTime;

struct ShortTimer;

impl FormatTime for ShortTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{} ", now.format("%H:%M:%S"))
    }
}

// 初始化
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(&args.log_level))
    .with_target(false)          // 隐藏模块路径
    .with_timer(ShortTimer)      // HH:MM:SS 格式
    .compact()                   // 紧凑模式
    .with_writer(std::io::stderr) // 输出到 stderr
    .init();
```

## 格式对比

| 模式 | 示例输出 | 使用场景 |
| --- | --- | --- |
| CLI（当前） | `14:23:05  INFO 正在创建网络 shittim-chest...` | 开发、运维 |
| 服务端（未来） | `{"timestamp":"...","level":"INFO","message":"..."}` | 生产日志收集 |

## --log-level 参数

CLI 接受 `--log-level` / `-l` 参数（默认 `info`）：

```text
shittim-chest --log-level debug dev
shittim-chest -l trace status
```

支持的级别：`trace`、`debug`、`info`、`warn`、`error`。

## 日志级别使用规范

| 级别 | 用途 | 典型 CLI 场景 |
| --- | --- | --- |
| `info` | 重要操作 | 容器创建/启动/停止、迁移开始/完成 |
| `warn` | 潜在问题 | 迁移重试、容器存在但处于异常状态 |
| `error` | 错误 | 容器崩溃、迁移失败、网络创建失败 |
| `debug` | 调试信息 | （当前未使用，预留未来使用） |
| `trace` | 详细流程 | （当前未使用，预留未来使用） |

## 设计原则

1. **CLI 不吞没错误**：所有错误通过 `anyhow::Result` 向上传播；`main()` 自动打印错误链。
1. **每个操作开始均有日志**：`正在创建网络...`、`正在运行迁移...`、`正在构建 shittim_chest...`——用户知道 CLI 正在做什么。
1. **每个操作完成均有确认**：`shittim-chest 已在 0.0.0.0:80 启动`、`所有服务已启动`。
1. **静默成功的操作不记录日志**：如果网络已存在，`ensure_network` 不打印日志，避免噪音。
1. **容器日志通过 Docker API 获取**：CLI 自身不写入业务日志，仅写入编排操作日志。

## 与 entelecheia 对齐

| 特性 | entelecheia CLI | shittim-chest CLI | 对齐 |
| --- | --- | --- | --- |
| 框架 | `tracing` | `tracing` | ✅ |
| 时间格式 | `ShortTimer`（HH:MM:SS） | `ShortTimer`（HH:MM:SS） | ✅ |
| 输出目标 | stderr | stderr | ✅ |
| 紧凑模式 | `.compact()` | `.compact()` | ✅ |
| 隐藏目标 | `.with_target(false)` | `.with_target(false)` | ✅ |
| --log-level | 支持 | 支持 | ✅ |

两个项目的 CLI 日志输出在视觉上完全相同，方便开发者在两个项目间切换。
