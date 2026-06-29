+++
title = "VNC（RFB）協定客戶端"
description = """架構決策記錄 —— VNC（RFB）協定客戶端。"""
lang = "zht"
category = "design"
subcategory = "router"
+++

# VNC（RFB）協定客戶端

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

Evernight 需要一個支援 VNC 的圖形化遠端桌面客戶端。RFB 協定（RFC 6143）是 VNC 的標準。選項：使用現有的 Rust crate、綁定到 libvncclient，或從頭開始實作。

## 決策

從頭開始實作一個純 Rust 的 RFB 003.008 客戶端。支援版本握手、安全協商（None、VncAuth）、DES 挑戰-回應認證、像素格式協商，以及 Raw/CopyRect 編碼。實作 `ViewportBackend` trait 以進行前端整合。

## 後果

### 正面

- 無 C 相依；完全控制協定流程。

### 負面

- ZRLE 和 Tight 編碼尚未實作（實作工作量龐大）。
- VNC 認證需要 DES 實作。
