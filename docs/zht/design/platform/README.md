+++
title = "平臺級設計文件"
description = """跨專案（平臺級）設計文件。與按專案劃分的 core/webui/router 子分類不同，這裡的文件覆蓋橫跨三個專案（entelecheia、shittim-chest、evernight）的關切——例如三者共享的統一監督、滾動更新與副本架構。"""
lang = "zht"
category = "design"
subcategory = "platform"
+++

# 平臺級設計文件

> **範圍。** 本文件為*平臺級*：橫切 `core`（entelecheia）、`webui`
>（shittim-chest）與 `router`（evernight）。各專案自身的設計位於其各自的
> 子分類下。

## 索引

| 文件 | 摘要 |
| --- | --- |
| [統一監督、滾動更新與副本架構](supervision-and-rolling-update.md) | 三專案共享的同一套監督樹骨架：統一的訊號/排空語義、基於 systemd socket activation 的零停機交接、可插拔的協調鎖 trait，以及構建在同一套 Worker + Supervisor 原語之上的兩種容錯策略（副本 = 負載均衡 ⊃ 滾動更新；主備 = 邊緣 HA）。 |

## 語言目錄

| 程式碼 | 語言 |
| --- | --- |
| `en/` | 英語（權威） |
| `zhs/` | 簡體中文 |
| `zht/` | 繁體中文 |
| `ja/` | 日本語 |
| `ko/` | 한국어 |
| `fr/` | Français |
| `es/` | Español |
| `ru/` | Русский |
