+++
title = "容器运行时客户端（Docker/Podman）"
description = """架构决策记录 —— 容器运行时客户端（Docker/Podman）。"""
lang = "zhs"
category = "design"
subcategory = "router"
+++

# 容器运行时客户端（Docker/Podman）

- **状态**：已采纳
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

管理容器（Docker、Podman）是通用连接管理器的关键用例。操作包括列出容器、exec shell、查看日志和端口转发。

## 决策

通过 Unix socket（Windows 上为命名管道）使用 Docker Engine API。Podman 的 Docker 兼容 API 通过相同代码路径支持。定义类型化的容器模型（`ContainerInfo`、`ContainerState`、`ContainerPort`）。后续计划：通过 Docker exec attach 实现 `TerminalBackend`，用于交互式容器 shell。

## 影响

### 积极影响

- Docker API 文档完善且稳定；Podman 兼容性免费获得。

### 消极影响

- 需要 Docker/Podman 守护进程处于运行状态。
- 不同 Docker 版本之间存在 API 版本差异。
