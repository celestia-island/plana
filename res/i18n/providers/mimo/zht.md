# Xiaomi MiMo

## 簡介

小米 MiMo 是小米開發的 AI 模型系列，專為程式設計和通用任務設計。Token Plan 提供基於訂閱的 MiMo 模型存取，支援 OpenAI 相容和 Anthropic 相容的 API 端點，涵蓋多個區域叢集。

## 組織

小米公司是一家中國電子科技公司，旗下 MiMo 品牌開發 AI 模型。MiMo 模型針對程式設計輔助進行了最佳化，支援函式呼叫、串流輸出等 OpenAI 相容特性。

## Token Plan

Token Plan 是基於訂閱的存取模式，API Key 格式為 `tp-xxxxx`（與按量付費的 `sk-xxxxx` 不同）。可用叢集：

- 中國：`https://token-plan-cn.xiaomimimo.com/v1`
- 新加坡：`https://token-plan-sgp.xiaomimimo.com/v1`
- 歐洲：`https://token-plan-ams.xiaomimimo.com/v1`

## 鑑權

Token Plan 使用自訂 `api-key` 請求標頭（非標準的 `Authorization: Bearer`）。系統在 auth type 設定為 `api-key` 時自動處理。

## 官方網站

https://platform.xiaomimimo.com
