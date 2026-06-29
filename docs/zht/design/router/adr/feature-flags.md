+++
title = "功能標誌架構"
description = """架構決策記錄 —— 功能標誌架構。"""
lang = "zht"
category = "design"
subcategory = "router"
+++

# 功能標誌架構

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

Evernight 的單體相依圖強制所有使用者引入每一個相依套件（`webrtc`、`russh`、`screenshots`、`sysinfo`），即使他們只需要 SSH 或硬體遙測。這會增加下游使用者的編譯時間和二進位檔案大小。

## 決策

使用 Cargo 功能標誌將 crate 拆分為以下功能：

| 功能         | 控制範圍                                       |
|--------------|----------------------------------------------|
| `screen`     | 螢幕擷取模組 + `screenshots` crate              |
| `webrtc`     | WebRTC 模組 + `webrtc` crate（隱含 `screen`）     |
| `remote-ssh` | SSH 處理器模組 + `russh` crate                    |
| `hardware`   | 硬體遙測模組 + `sysinfo` crate                    |
| `protocol`   | 協定／訊息型別（無重型相依套件）                     |
| `tunnel`     | TCP 通道模組                                     |
| `full`       | 所有功能（預設）                                   |

每個功能同時控制該模組及其相依套件。`webrtc` 功能隱含 `screen`，因為 WebRTC 會話需要螢幕擷取。

## 後果

### 正面

- 使用者只編譯其需要的內容
- 部分使用時減少編譯時間

### 負面

- 功能標誌組合矩陣不斷增長；必須在 CI 中測試每個功能組合
