<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Arona logo" width="200"/>

# Arona

**Entelecheia 多智能体平台的 JSON-RPC 2.0 共享协议类型**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[英语](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日语](../ja/README.md)** &bull;
**[韩语](../ko/README.md)** &bull; **[法语](../fr/README.md)** &bull;
**[西班牙语](../es/README.md)** &bull; **[俄语](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **版本 0.1.0** — 由 [entelecheia](https://github.com/celestia-island/entelecheia) 和 [shittim-chest](https://github.com/celestia-island/shittim-chest) 使用。

JSON-RPC 2.0 消息类型、Agent 分类法（14 种变体）、约 100 种 WebSocket 参数类型。通过 `ts-rs` 自动生成 TypeScript 绑定。

## 用法

**Rust：**
```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript：**
```bash
pnpm add @celestia-island/arona
```

## 许可证

Business Source License 1.1 — 非商业使用遵循 Apache-2.0 或 MIT 协议。
