+++
title = "VNC (RFB) 协议客户端"
description = """架构决策记录 —— VNC (RFB) 协议客户端。"""
lang = "zhs"
category = "design"
subcategory = "router"
+++

# VNC (RFB) 协议客户端

- **状态**：已采纳
- **日期**：2026-06-09
- **作者**：Evernight 核心团队

## 背景

Evernight 需要支持 VNC 的图形化远程桌面客户端。RFB 协议（RFC 6143）是 VNC 的标准协议。可选方案：使用现有 Rust crate、绑定 libvncclient，或从零实现。

## 决策

从零实现纯 Rust 的 RFB 003.008 客户端。支持版本握手、安全协商（None、VncAuth）、DES 挑战-响应认证、像素格式协商以及 Raw/CopyRect 编码。实现 `ViewportBackend` trait 以集成前端。

## 影响

### 积极影响

- 无 C 依赖；完全掌控协议流程。

### 消极影响

- ZRLE 和 Tight 编码尚未实现（实现工作量较大）。
- VNC 认证需要 DES 实现。
