# Xiaomi MiMo

## 概要

Xiaomi MiMo は、Xiaomi が開発した AI モデルシリーズで、コーディング支援と汎用タスク向けに設計されています。Token Plan は、複数のリージョンクラスタにわたる OpenAI 互換および Anthropic 互換の API エンドポイントを通じて、MiMo モデルへのサブスクリプションベースのアクセスを提供します。

## 組織

Xiaomi Corporation は、MiMo ブランドで AI モデルを開発する中国の電子機器企業です。MiMo モデルはコーディング支援に最適化されており、関数呼び出し、ストリーミング、その他の OpenAI 互換機能をサポートしています。

## トークンプラン

Token Plan はサブスクリプションベースのアクセスモデルで、API キーは `tp-xxxxx` 形式（従量課金の `sk-xxxxx` キーとは異なります）を使用します。利用可能なクラスタ：

- 中国: `https://token-plan-cn.xiaomimimo.com/v1`
- シンガポール: `https://token-plan-sgp.xiaomimimo.com/v1`
- ヨーロッパ: `https://token-plan-ams.xiaomimimo.com/v1`

## 認証

Token Plan はカスタムの `api-key` ヘッダーを使用します（標準の `Authorization: Bearer` ではありません）。認証タイプが `api-key` に設定されている場合、システムが自動的にこれを処理します。

## 公式ウェブサイト

https://platform.xiaomimimo.com
