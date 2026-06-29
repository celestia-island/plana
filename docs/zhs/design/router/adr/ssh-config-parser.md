+++
title = "SSH 配置解析器"
description = """架构决策记录 —— SSH 配置解析器。"""
lang = "zhs"
category = "design"
subcategory = "router"
+++

# SSH 配置解析器

- **状态**：已采纳
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

用户期望 `evernight` 像 `ssh` 命令一样读取 `~/.ssh/config`，获取主机别名、跳板机和密钥文件配置。没有此功能，用户每次调用都必须重复所有连接参数。

## 决策

实现纯 Rust SSH 配置解析器，处理标准指令：`Host`、`HostName`、`User`、`Port`、`IdentityFile`、`ProxyJump`、`ForwardAgent`、`ServerAliveInterval` 等。对 `Host` 条目使用 glob 模式匹配。与连接池集成，实现透明的基于配置的连接。

## 影响

### 积极影响

- 与 SSH 命令无缝兼容；支持从配置文件解析跳板机。

### 消极影响

- 需要跟踪 OpenSSH 配置语法变更。
- `Match` 指令支持较为复杂。
