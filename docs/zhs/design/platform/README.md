+++
title = "平台级设计文档"
description = """跨项目（平台级）设计文档。与按项目划分的 core/webui/router 子分类不同，这里的文档覆盖横跨三个项目（entelecheia、shittim-chest、evernight）的关切——例如三者共享的统一监督、滚动更新与副本架构。"""
lang = "zhs"
category = "design"
subcategory = "platform"
+++

# 平台级设计文档

> **范围。** 本文档为*平台级*：横切 `core`（entelecheia）、`webui`
>（shittim-chest）与 `router`（evernight）。各项目自身的设计位于其各自的
> 子分类下。

## 索引

| 文档 | 摘要 |
| --- | --- |
| [统一监督、滚动更新与副本架构](supervision-and-rolling-update.md) | 三项目共享的同一套监督树骨架：统一的信号/排空语义、基于 systemd socket activation 的零停机交接、可插拔的协调锁 trait，以及构建在同一套 Worker + Supervisor 原语之上的两种容错策略（副本 = 负载均衡 ⊃ 滚动更新；主备 = 边缘 HA）。 |

## 语言目录

| 代码 | 语言 |
| --- | --- |
| `en/` | 英语（权威） |
| `zhs/` | 简体中文 |
| `zht/` | 繁體中文 |
| `ja/` | 日本語 |
| `ko/` | 한국어 |
| `fr/` | Français |
| `es/` | Español |
| `ru/` | Русский |
