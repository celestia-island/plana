+++
title = "连接条目 URI 方案"
description = """架构决策记录 —— 连接条目 URI 方案。"""
lang = "zhs"
category = "design"
subcategory = "router"
+++

# 连接条目 URI 方案

- **状态**：已采纳
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

连接目录需要统一的方式来表示不同协议的连接（SSH、VNC、RDP、串口、Docker）。URI 方案提供了人类可读、可序列化的描述符。

## 决策

使用协议相关的 URI 方案：

- `ssh://user@host:port`
- `vnc://host:5900`
- `rdp://host:3389`
- `serial:///dev/ttyUSB0?baud=9600`
- `docker:///var/run/docker.sock?container=name`

`ConnectionEntry` 将 URI 解析为带类型的结构体，包含 scheme、host、port、username、path 和查询参数。目录是由 `ConnectionCategory` 节点组成的树，节点中保存连接条目。

## 影响

### 积极影响

- 熟悉的 URI 格式；易于序列化；支持复制粘贴分享连接。

### 消极影响

- 部分协议无法干净地映射到 URI（如 Kubernetes context）。
- 查询字符串参数缺乏结构化约束。
