+++
title = "使用 tokio 作为异步运行时"
description = """架构决策记录 —— 使用 tokio 作为异步运行时。"""
lang = "zhs"
category = "design"
subcategory = "router"
+++

# 使用 tokio 作为异步运行时

- **状态**：已接受
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

Evernight 的各子系统（屏幕捕获、SSH、WebRTC 信令、隧道）均涉及大量异步 I/O 操作。需要一个统一的异步运行时来调度任务、管理定时器和处理并发网络连接。Rust 生态中主要的异步运行时选项包括 tokio、async-std 和 smol。

关键约束：

- 项目依赖的多个 crate（russh、webrtc、tower）原生支持 tokio
- 需要同时支持 TCP、Unix socket 和文件 I/O 的异步操作
- 运行时须在 Linux、macOS 和 Windows 上一致运行

## 决策

选择 **tokio** 作为 Evernight 的异步运行时。使用多线程调度器（`tokio::runtime::MultiThread`）作为默认运行时配置。

具体策略：

- 全局使用 `#[tokio::main]` 入口宏
- 通过 `tokio::spawn` 管理并发任务
- 使用 tokio 提供的 `AsyncRead`/`AsyncWrite` trait 进行 I/O 抽象
- 利用 `tokio::sync` 中的通道原语（mpsc、broadcast、watch）进行模块间通信
- 采用 `tokio::net` 处理所有网络连接

## 影响

### 积极影响

- tokio 是 Rust 生态中最为成熟和广泛使用的异步运行时，社区支持完善
- 项目核心依赖（russh、webrtc 等）已原生适配 tokio，无需额外桥接层
- 丰富的内置工具集（定时器、信号处理、文件系统异步操作）减少外部依赖
- 多线程调度器能充分利用多核 CPU

### 消极影响

- tokio 运行时体积较大，增加了二进制大小
- 深度绑定 tokio 生态，未来迁移成本高
- 部分异步代码无法直接在其他运行时中复用
