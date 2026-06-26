+++
title = "Webhook 平台設定"
description = """> 當前 webhook 佈局與整合範圍說明"""
lang = "zht"
category = "guides"
subcategory = "core"
+++

# Webhook 平台設定

> 當前 webhook 佈局與整合範圍說明

## 概述

倉庫中已經包含面向程式碼託管平台和聊天平台的 webhook 整合，但整體仍處於過渡階段，並不是一個已經完全統一、成熟定型的方案。

當前目錄結構同時存在：

- 舊的分平台目錄，如 `plugins/github-webhook/github/`、`gitee/`、`gitlab/`、`telegram/`、`qq/`、`lark/`
- 較新的 TypeScript 實作：`plugins/github-webhook/ts/`

TypeScript 套件當前接入了：

- GitHub
- Gitee
- GitLab
- 飛書 / Lark
- QQ
- Discord
- Telegram

## 當前已經能做什麼

- 接收 webhook 或 bot 事件
- 透過 WebSocket 或 HTTP 輔助呼叫把事件轉發給 Scepter
- 在 TypeScript 服務中提供 `/health` 健康檢查介面

## 當前還不能預設保證什麼

- 所有平台都有統一穩定的部署方案
- 每個平台都已經形成完整的 issue 驅動 skill chain
- 所有平台接入都達到同一成熟度

## TypeScript 套件

位置：`plugins/github-webhook/ts/`

開發執行方式：

```bash
cd plugins/github-webhook/ts
npm install
npm run dev
```

生產構建方式：

```bash
cd plugins/github-webhook/ts
npm run build
npm start
```

## 關鍵環境變數

- `PORT`：webhook 服務埠，預設 `8000`
- `SCEPTER_URL`：HTTP 轉發位址，預設 `http://localhost:8424`
- `SCEPTER_WS_URL`：WebSocket 轉發位址，預設 `ws://localhost:8424/ws`

## 使用建議

可以把 webhook 能力視為「已經存在，但成熟度不均衡」。如果你依賴某個平台，請先核對 `plugins/github-webhook/` 下對應 router 或 bot 的實際實作，再決定是否把它描述為可穩定生產使用。
