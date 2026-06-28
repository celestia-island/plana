+++
title = "关于什亭之匣"
description = """版本 0.1.0"""
lang = "zhs"
category = "architecture"
subcategory = "webui"
+++

# Shittim Chest（什亭之匣）

**版本 0.1.0**

Shittim Chest 是 [entelecheia](https://github.com/celestia-island/entelecheia) 多智能体协作平台面向用户的壳层，使用 Rust 和 Vue 3 构建。

## 架构

Shittim Chest 由多个组件组成，它们协同工作以提供完整的用户体验：

- **arona** — 您正在使用的聊天 UI，支持流式响应、图像生成、智能体状态监控、思考窗口、远程设备查看器和多语言支持。
- **`shittim_chest`** — 统一的 Rust + Axum 后端，处理认证（JWT + OAuth）、独立 LLM 路由、聊天 API、图像生成、webhook 入口、scepter 代理和远程设备信令。

## 与 Entelecheia 的关系

[entelecheia](https://github.com/celestia-island/entelecheia) 是核心的多智能体编排引擎。它提供智能体运行时（scepter、13 个专用智能体、Cosmos/IEPL 运行时）。Shittim Chest 处理用户直接交互的一切 — 身份、展示和通信。

这两个项目按设计分离：entelecheia 管理智能体编排，而 shittim-chest 管理用户身份和展示。它们通过 JWT 认证的 HTTP/WebSocket 进行通信。登录凭证存储在 `shittim_chest_db` 中；权限和身份数据存储在 `entelecheia_db` 中。这种分离允许前端壳层独立于智能体核心演进。

## 与 Hikari 的关系

[hikari](https://github.com/celestia-island/hikari) 是 Celestia Island 生态系统的网关和路由层。它作为所有外部流量的入口点，处理 shittim-chest、entelecheia 和其他服务之间的请求路由、负载均衡和 API 网关功能。

## 与 Tairitsu 的关系

[tairitsu](https://github.com/celestia-island/tairitsu) 是 Celestia Island 生态系统的跨平台原生应用框架。它提供基于 Tauri 的桌面和移动客户端，将 arona 包装为原生应用，以及为开发工作流提供支持的浏览器自动化和测试基础设施。

## 许可证

Shittim Chest 根据 **Business Source License 1.1（BSL-1.1）** 授权。

对于**非商业用途** — 包括内部运营、学术研究、教学、个人学习、评估、政府和公共服务以及教育用途 — 授予的权利等同于 **Synthetic Source License 1.0（SySL-1.0）**（"免费使用许可证"）。您可以为这些目的自由地使用、研究、修改和运行本软件。

**商业用途** — 例如将软件作为托管服务提供给第三方、将其作为独立产品重新分发，或将其用作商业产品的核心组件 — 需要从许可方获得单独的商业许可证。

有关详细信息，请参阅[完整许可证文本](https://github.com/celestia-island/shittim-chest/blob/main/LICENSE)。

---

由 [Celestia Island](https://github.com/celestia-island) 用 ❤ 构建。
