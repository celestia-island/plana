+++
title = "内嵌前端策略"
description = """shittim-chest 支持两种前端托管模式：开发模式下，`dev.py` 监听前端源码并在变更时触发 `pnpm build`，后端在 `:3000` 端口同时提供静态文件和 API；发布模式下，前端静态文件在编译时嵌入 Rust 二进制并在 `:80` 端口提供服务。模式通过 `embedded-frontend` Cargo 特性切换，使用 `#[cfg(feature = "embedded-frontend")]` 进行代码级条件编译。"""
lang = "zhs"
category = "design"
subcategory = "webui"
+++

# 内嵌前端策略

## 概述

shittim-chest 支持两种前端托管模式：开发模式下，`dev.py` 监听前端源码并在变更时触发 `pnpm build`，后端在 `:3000` 端口同时提供静态文件和 API；发布模式下，前端静态文件在编译时嵌入 Rust 二进制并在 `:80` 端口提供服务。模式通过 `embedded-frontend` Cargo 特性切换，使用 `#[cfg(feature = "embedded-frontend")]` 进行代码级条件编译。

## 架构对比

```mermaid
flowchart TB
    subgraph Dev[开发模式：dev.py + 后端]
        D1[dev.py 监听前端源码] --> D2[pnpm build → dist/]
        D2 --> D3[shittim_chest :3000 提供静态 + API]
    end
    subgraph Release[发布模式：内嵌]
        R1[浏览器] --> R2[shittim_chest :80]
        R2 --> R3[API + LLM]
        R2 --> R4[/static/*\n内嵌 SPA]
    end
```

| 维度 | 开发模式（无特性） | 发布模式（embedded-frontend） |
| --- | --- | --- |
| 前端来源 | Vite 构建，后端托管 | `include_dir!` 编译时嵌入 |
| 热重载 | 通过 dev.py 自动重构建 | 不支持（静态） |
| API 请求路由 | 浏览器直连（同源） | 浏览器直连 |
| 二进制大小 | 仅后端 | + 前端 dist/ 目录 |
| 需要 Node | 是（仅构建） | 否 |
| 启动方式 | `dev.py`（监听 + 重构建） | `just up` 一次性启动 |

## 实现细节

### 条件编译

```rust
# [cfg(feature = "embedded-frontend")]
static ARONA_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../dist/arona");

async fn serve_arona() -> impl IntoResponse {
    #[cfg(feature = "embedded-frontend")]
    {
        // 从编译时嵌入的 Dir 读取
    }
    #[cfg(not(feature = "embedded-frontend"))]
    {
        // 从文件系统 ./dist/arona/index.html 读取
    }
}
```

条件编译在**函数体级别**而非模块级别操作，保持公共 API 在两种模式间完全一致。

### SPA 回退

应用为单页应用。所有不匹配静态资源的路由返回 `index.html`：

```text
GET /               → index.html
GET /chat/123       → index.html（前端路由器处理）
GET /backend        → index.html
GET /backend/providers → index.html（前端路由器处理）
```

### MIME 类型检测

静态文件服务根据文件扩展名返回正确的 Content-Type：

| 扩展名 | Content-Type |
| --- | --- |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.html` | `text/html` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff/.woff2` | `font/woff2` |
| 其他 | `application/octet-stream` |

## Dockerfile 中的前端构建

```text
Stage 1（前端）：
  node:22-slim → pnpm install → pnpm build:all → /app/dist/arona/

Stage 2（构建器）：
  rust:1.85-slim → COPY /app/dist/ → cargo build --features embedded-frontend

Stage 3（运行时）：
  debian:bookworm-slim → COPY 二进制 → ENTRYPOINT ["./shittim_chest"]
```

前端构建和 Rust 编译在同一 Dockerfile 内完成。最终运行时镜像仅包含编译后的二进制。

## 设计决策

1. **开发模式使用 dev.py 自动重构建**：`dev.py` 监听前端源码并在变更时重构建，后端在单一端口提供所有服务。
1. **发布模式无需反向代理**：二进制内嵌 SPA，实现单进程部署，降低运维复杂度。
1. **前端不在运行时动态加载**：避免文件系统依赖和版本不一致。发布镜像仅包含单个二进制文件。
1. **单 SPA**：前端在 `/` 提供服务，管理面板在 `/backend`。
