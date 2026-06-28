+++
title = "Entelecheia 正式環境部署檢查清單"
description = """> 將 Entelecheia 部署至正式環境的 12 步驟檢查清單。"""
lang = "zht"
category = "design"
subcategory = "core"
+++

# Entelecheia 正式環境部署檢查清單

> 將 Entelecheia 部署至正式環境的 12 步驟檢查清單。

## 部署前

- [ ] **1. 選擇資料庫模式**
  - 嵌入式 pglite：單一二進位檔，無需外部資料庫。適用於 <50 個並行 Agent。
  - PostgreSQL：正式環境推薦。設定 `DATABASE_URL`。

  ```bash
  # 嵌入式模式
  docker run -d -p 8080:8080 -v data:/data entelecheia:latest

  # PostgreSQL 模式
  docker-compose up -d
  ```

- [ ] **2. 設定使用者身份**

  ```bash
  export ENTELECHEIA_USER_UUID=$(uuidgen)
  ```

此 UUID 為工作區所有者身份。所有 Agent 操作均以此為範圍。

- [ ] **3. 設定 LLM 提供者**

  ```bash
  entelecheia-cli config set-provider openai --api-key sk-...
  entelecheia-cli config set-provider anthropic --api-key sk-ant-...
  ```

API 金鑰透過 Aporia Agent 以 AES-256-GCM 靜態加密。

- [ ] **4. 配置容器執行時**
  - Docker（預設）：`--container-backend docker`
  - Youki（無 root OCI）：`--container-backend youki`
  - 驗證 seccomp 配置：`configs/seccomp/`

- [ ] **5. 檢視安全策略**

  ```bash
  # 列出已註冊的安全策略
  entelecheia-cli security policy-list

  # 檢視 OreXis 哨兵配置
  entelecheia-cli config show orexis
  ```

## 部署

- [ ] **6. 建置或拉取映像**

  ```bash
  # 從原始碼建置
  docker build -t entelecheia:latest .

  # 或使用發行版
  curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
  ```

- [ ] **7. 啟動服務**

  ```bash
  # 使用 Docker Compose（推薦）
  docker-compose up -d

  # 或獨立執行
  docker run -d --name entelecheia \
    -p 8080:8080 \
    -v entelecheia-data:/data \
    -e ENTELECHEIA_USER_UUID=$ENTELECHEIA_USER_UUID \
    --restart unless-stopped \
    entelecheia:latest
  ```

- [ ] **8. 驗證健康狀態**

  ```bash
  entelecheia-cli status
  curl http://localhost:8080/health
  ```

- [ ] **9. 初始化 Agent 的 Docker 映像**

  ```bash
  entelecheia-cli init-docker-images
  ```

這會建置每個第 1 層 Agent 用於隔離執行的容器映像。

## 部署後

- [ ] **10. 設定監控**

  ```bash
  # 啟用追蹤
  export RUST_LOG=info,entelecheia=debug

  # 檢查時間線是否有問題
  entelecheia-cli timeline list --agent orexis
  ```

- [ ] **11. 配置備份**
  - 嵌入式模式：備份 `/data` 目錄
  - PostgreSQL：`pg_dump` 或 WAL 歸檔
  - 時間線審計日誌：定期匯出

- [ ] **12. 負載測試**

  ```bash
  # 發送測試訊息
  entelecheia-cli send "你好，驗證系統是否正常運作"

  # 檢查 Agent 狀態
  entelecheia-cli agent list

  # 驗證審計追蹤
  entelecheia-cli trace-chain demiurge.001
  ```

## 安全強化（建議）

| 檢查項目 | 指令 |
| --- | --- |
| 驗證環境變數中無機敏資料 | `env \| grep -i key` |
| 檢視 RBAC 群組 | `entelecheia-cli security rbac-list` |
| 檢查速率限制 | `entelecheia-cli config show channel.rate_limit` |
| 驗證容器隔離 | `docker inspect entelecheia \| grep SecurityOpt` |
| 檢視 OreXis 審計日誌 | `entelecheia-cli logs --agent orexis --lines 100` |

## 疑難排解

| 症狀 | 診斷方式 |
| --- | --- |
| Agent 無回應 | `entelecheia-cli status` → 檢查 scepter 是否執行中 |
| LLM 調用失敗 | 檢查 API 金鑰：`entelecheia-cli config show providers` |
| 容器錯誤 | `docker logs entelecheia` → 尋找 Youki/Docker 錯誤 |
| 資料庫問題 | 檢查 `DATABASE_URL` 或 pglite 資料目錄權限 |
| 工具權限被拒 | `entelecheia-cli security policy-list` → 檢視被拒絕的調用 |
