+++
title = "錯誤處理 — thiserror 搭配 crate Result"
description = """架構決策記錄 —— 錯誤處理 — thiserror 搭配 crate Result。"""
lang = "zht"
category = "design"
subcategory = "router"
+++

# 錯誤處理 — thiserror 搭配 crate Result

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

Evernight 需要一個統一的錯誤型別，涵蓋所有模組（螢幕、SSH、硬體、網路、通道）。函式庫必須暴露一個乾淨的錯誤 API，同時保持內部錯誤處理的人體工學。

## 決策

使用 `thiserror` 來派生 `EvernightError` 枚舉，並為每個變體提供 `#[error(...)]` 顯示實作。定義 `pub type Result<T> = std::result::Result<T, EvernightError>` 作為 crate 範圍的結果型別。每個變體以 `String` 形式擷取特定領域的上下文（ScreenCapture、Ssh、Tunnel 等），而非包裝外部錯誤型別，以避免洩漏內部相依套件的細節。

## 後果

### 正面

- 穩定的公開錯誤 API；使用者無需了解內部細節即可匹配枚舉變體

### 負面

- 轉換為 `String` 會造成一些資訊損失
- 沒有結構化的錯誤鏈結
