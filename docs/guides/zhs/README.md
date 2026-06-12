<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

# Arona

**Entelecheia 多智能体平台共享 JSON-RPC 2.0 协议类型**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](../../README.md)** &bull; **[简体中文](README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

## Arona 是什么

Arona 定义了 entelecheia 智能体编排核心与其用户界面层（TUI、CLI、Web UI、IDE 插件、Tauri 应用）之间的通信协议。包含：

- **JSON-RPC 2.0** 消息类型
- **智能体分类** — `Agent` 枚举（14 个变体）
- **WebSocket 参数类型** — 约 100 个结构体，覆盖流式传输、快照、补丁、任务、提供商配置、知识库、YOLO 巡航控制、WebRTC 信令、仲裁器、VM 快照
- **TypeScript 类型生成** — 所有类型通过 `ts-rs` 导出至 `bindings/WsTypes.ts`

### 命名来源

Arona（アロナ）——在什亭之匣内协调任务、路由指令的 AI 助手。

## 用法

**Rust：**

```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript（pnpm / npm）：**

```bash
pnpm add @celestia-island/arona
```

```ts
import type { Agent, TuiAgentInfo, SkillStage } from "@celestia-island/arona";
```

## 参与贡献

欢迎提交 Issue 和 Pull Request。

## 许可证

Business Source License 1.1，附带 Apache-2.0 / MIT 双路径：个人、学术和非商业用途适用 Apache 2.0 或 MIT。商业用途（托管、转售、付费服务）需要 BUSL 许可证。

翻译版本：[繁體中文](../../LICENSE.zht) · [Español](../../LICENSE.es) · [Français](../../LICENSE.fr) · [Русский](../../LICENSE.ru) · [العربية](../../LICENSE.ar)
