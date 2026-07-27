# 小米 MiMo

## 简介

小米 MiMo 是小米开发的 AI 模型系列，专为编程和通用任务设计。Token Plan 提供基于订阅的 MiMo 模型访问，支持 OpenAI 兼容和 Anthropic 兼容的 API 端点，覆盖多个区域集群。

## 组织

小米公司是一家中国电子科技公司，旗下 MiMo 品牌开发 AI 模型。MiMo 模型针对编程辅助进行了优化，支持函数调用、流式输出等 OpenAI 兼容特性。

## Token Plan

Token Plan 是基于订阅的访问模式，API Key 格式为 `tp-xxxxx`（与按量付费的 `sk-xxxxx` 不同）。可用集群：

- 中国：`https://token-plan-cn.xiaomimimo.com/v1`
- 新加坡：`https://token-plan-sgp.xiaomimimo.com/v1`
- 欧洲：`https://token-plan-ams.xiaomimimo.com/v1`

## 鉴权

Token Plan 使用自定义 `api-key` 请求头（非标准的 `Authorization: Bearer`）。系统在 auth type 设置为 `api-key` 时自动处理。

## 官网

https://platform.xiaomimimo.com
