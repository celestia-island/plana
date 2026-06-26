# Webhook 平台设置

> 当前 webhook 布局与集成范围说明

## 概述

仓库中已经包含面向代码托管平台和聊天平台的 webhook 集成，但整体仍处于过渡阶段，并不是一个已经完全统一、成熟定型的方案。

当前目录结构同时存在：

- 旧的分平台目录，如 `plugins/github-webhook/github/`、`gitee/`、`gitlab/`、`telegram/`、`qq/`、`lark/`
- 较新的 TypeScript 实现：`plugins/github-webhook/ts/`

TypeScript 包当前接入了：

- GitHub
- Gitee
- GitLab
- 飞书 / Lark
- QQ
- Discord
- Telegram

## 当前已经能做什么

- 接收 webhook 或 bot 事件
- 通过 WebSocket 或 HTTP 辅助调用把事件转发给 Scepter
- 在 TypeScript 服务中提供 `/health` 健康检查接口

## 当前还不能默认保证什么

- 所有平台都有统一稳定的部署方案
- 每个平台都已经形成完整的 issue 驱动 skill chain
- 所有平台接入都达到同一成熟度

## TypeScript 包

位置：`plugins/github-webhook/ts/`

开发运行方式：

```bash
cd plugins/github-webhook/ts
npm install
npm run dev
```

生产构建方式：

```bash
cd plugins/github-webhook/ts
npm run build
npm start
```

## 关键环境变量

- `PORT`：webhook 服务端口，默认 `8000`
- `SCEPTER_URL`：HTTP 转发地址，默认 `http://localhost:8424`
- `SCEPTER_WS_URL`：WebSocket 转发地址，默认 `ws://localhost:8424/ws`

## 使用建议

可以把 webhook 能力视为“已经存在，但成熟度不均衡”。如果你依赖某个平台，请先核对 `plugins/github-webhook/` 下对应 router 或 bot 的实际实现，再决定是否把它描述为可稳定生产使用。
