+++
title = "Entelecheia プロダクションデプロイメントチェックリスト"
description = """> Entelecheiaをプロダクションにデプロイするための12ステップチェックリスト。"""
lang = "ja"
category = "design"
subcategory = "core"
+++

# Entelecheia プロダクションデプロイメントチェックリスト

> Entelecheiaをプロダクションにデプロイするための12ステップチェックリスト。

## デプロイ前

- [ ] **1. データベースモードの選択**
  - 組み込みpglite: シングルバイナリ、外部DB不要。50未満の同時エージェントに適する。
  - PostgreSQL: プロダクションに推奨。`DATABASE_URL` を設定。

  ```bash
  # 組み込みモード
  docker run -d -p 8080:8080 -v data:/data entelecheia:latest

  # PostgreSQLモード
  docker-compose up -d
  ```

- [ ] **2. ユーザーIDの設定**

  ```bash
  export ENTELECHEIA_USER_UUID=$(uuidgen)
  ```

このUUIDはワークスペース所有者IDである。すべてのエージェント操作はこれにスコープされる。

- [ ] **3. LLMプロバイダの設定**

  ```bash
  entelecheia-cli config set-provider openai --api-key sk-...
  entelecheia-cli config set-provider anthropic --api-key sk-ant-...
  ```

APIキーはAporiaエージェントを介してAES-256-GCMで保存時に暗号化される。

- [ ] **4. コンテナランタイムの設定**
  - Docker（デフォルト）: `--container-backend docker`
  - Youki（ルートレスOCI）: `--container-backend youki`
  - seccompプロファイルの確認: `configs/seccomp/`

- [ ] **5. セキュリティポリシーの確認**

  ```bash
  # 登録済みセキュリティポリシーの一覧表示
  entelecheia-cli security policy-list

  # OreXisセンチネル設定の確認
  entelecheia-cli config show orexis
  ```

## デプロイ

- [ ] **6. イメージのビルドまたはプル**

  ```bash
  # ソースからビルド
  docker build -t entelecheia:latest .

  # またはリリースを使用
  curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
  ```

- [ ] **7. サービスの起動**

  ```bash
  # Docker Composeを使用（推奨）
  docker-compose up -d

  # またはスタンドアロン
  docker run -d --name entelecheia \
    -p 8080:8080 \
    -v entelecheia-data:/data \
    -e ENTELECHEIA_USER_UUID=$ENTELECHEIA_USER_UUID \
    --restart unless-stopped \
    entelecheia:latest
  ```

- [ ] **8. ヘルスチェックの確認**

  ```bash
  entelecheia-cli status
  curl http://localhost:8080/health
  ```

- [ ] **9. エージェント用Dockerイメージの初期化**

  ```bash
  entelecheia-cli init-docker-images
  ```

これにより、各レイヤー1エージェントが隔離実行に使用するコンテナイメージがビルドされる。

## デプロイ後

- [ ] **10. 監視の設定**

  ```bash
  # トレーシングの有効化
  export RUST_LOG=info,entelecheia=debug

  # タイムラインで問題を確認
  entelecheia-cli timeline list --agent orexis
  ```

- [ ] **11. バックアップの設定**
  - 組み込みモード: `/data` ディレクトリをバックアップ
  - PostgreSQL: `pg_dump` またはWALアーカイブ
  - タイムライン監査ログ: 定期的にエクスポート

- [ ] **12. 負荷テスト**

  ```bash
  # テストメッセージの送信
  entelecheia-cli send "Hello, システムが動作中であることを確認してください"

  # エージェント状態の確認
  entelecheia-cli agent list

  # 監査トレイルの検証
  entelecheia-cli trace-chain demiurge.001
  ```

## セキュリティ強化（推奨）

| チェック | コマンド |
| --- | --- |
| 環境変数にシークレットがないか確認 | `env \| grep -i key` |
| RBACグループの確認 | `entelecheia-cli security rbac-list` |
| レート制限の確認 | `entelecheia-cli config show channel.rate_limit` |
| コンテナ隔離の確認 | `docker inspect entelecheia \| grep SecurityOpt` |
| OreXis監査ログの確認 | `entelecheia-cli logs --agent orexis --lines 100` |

## トラブルシューティング

| 症状 | 診断方法 |
| --- | --- |
| エージェントが応答しない | `entelecheia-cli status` → scepterが実行中か確認 |
| LLM呼び出しが失敗する | APIキーを確認: `entelecheia-cli config show providers` |
| コンテナエラー | `docker logs entelecheia` → Youki/Dockerエラーを確認 |
| データベースの問題 | `DATABASE_URL` またはpgliteデータディレクトリの権限を確認 |
| ツール権限拒否 | `entelecheia-cli security policy-list` → 拒否された呼び出しを確認 |
