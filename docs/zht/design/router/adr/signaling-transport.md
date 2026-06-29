+++
title = "信令傳輸 — 雙重 Unix Socket / TCP"
description = """架構決策記錄 —— 信令傳輸 — 雙重 Unix Socket / TCP。"""
lang = "zht"
category = "design"
subcategory = "router"
+++

# 信令傳輸 — 雙重 Unix Socket / TCP

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

信令客戶端連接到中繼伺服器以交換 WebRTC SDP 提案／應答和 ICE 候選。最初它僅使用 Unix domain socket（`tokio::net::UnixStream`），這在 Windows 上不可用。為了跨平台支援，需要備援方案。

## 決策

實作一個雙重傳輸的 `SignalingStream` 枚舉，優先嘗試 Unix domain socket（在支援該功能的平台上，透過路徑格式偵測——以 `/` 開頭或以 `.sock` 結尾），並回退到 TCP。`TransportWriter` 適配器對兩種傳輸方式抽象化 `AsyncWrite`。在非 Unix 平台上，僅 TCP 可用。

## 後果

### 正面

- 跨平台信令；兩種傳輸方式上使用相同的 JSON-RPC 協定

### 負面

- TCP 信令預設為未加密
- Windows 使用者必須確保僅綁定到回環介面
