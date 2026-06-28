+++
title = "構建指南"
description = """- [先決條件](#先決條件)"""
lang = "zht"
category = "guides"
subcategory = "core"
+++

# 構建指南

---

## 目錄

- [先決條件](#先決條件)
- [安裝](#安裝)
- [組態](#組態)
- [構建](#構建)
- [執行](#執行)
- [資料庫管理](#資料庫管理)
- [開發環境](#開發環境)
- [部署](#部署)
- [故障排除](#故障排除)
- [執行 Webhook 機器人](#執行-webhook-機器人)

---

## 先決條件

### 系統需求

- **作業系統**: Linux、macOS 或 Windows（需要 Docker CLI）
- **記憶體**: 最低 8GB，推薦 16GB
- **儲存**: 最低 20GB 可用空間
- **CPU**: 推薦 4 核心以上

> 說明（設計意圖）
> Windows 側的核心需求是 Docker CLI 可用，命令可以直接在 PowerShell 或 Windows Terminal 執行。
> 但容器最終仍需要 Linux 執行時來承載：
> 1. 本機方案通常是 Docker Desktop（一般依賴 WSL2 後端）。
> 2. 替代方案是本機僅安裝 Docker CLI，並透過 `docker context` 轉發到遠端 Linux Docker 主機。

### 軟體需求

#### 必需軟體

- **Docker 或 Podman**（容器執行時環境）

```bash
docker --version
docker compose version
```

請按當前平台使用官方推薦安裝方式：

- Linux：安裝 Docker Engine、Docker Desktop for Linux，或發行版自帶的 Podman
- macOS：安裝 Docker Desktop 或 Podman Desktop
- Windows：安裝 Docker Desktop 或 Podman Desktop

**重要說明**：

- PostgreSQL 等執行時依賴已包含在容器化環境中
- 但如果要執行 `just` 配方或倉庫內輔助腳本，宿主機仍需要安裝 Python 3.8+
- 無需在宿主機上單獨安裝 PostgreSQL
- Windows 下命令可直接在 PowerShell 或 Windows Terminal 中執行，但部署仍要求可用的 Docker/Podman Linux 執行時。本地部署通常意味著使用帶 WSL2 後端的 Docker Desktop；也可透過本機 Docker CLI/context 轉發到遠端 Linux Docker 主機。

- **Rust 1.85+**（僅開發構建需要）

```bash
rustup update stable
```

請按平台使用官方 rustup 安裝方式：

- Linux/macOS：訪問 <https://rustup.rs>
- Windows：從 <https://rustup.rs> 下載並執行 `rustup-init.exe`，然後執行 `rustup update stable`

#### 推薦軟體

- **just**（命令執行器）

```bash
  # 使用 cargo
  cargo install just

  # 使用 brew（macOS）
  brew install just
  ```

- **VS Code** 並安裝 rust-analyzer 擴充

---

## 安裝

### 步驟 1: 複製倉庫

```bash
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia
```

### 步驟 2: 組態環境變數

```bash
# 在從 .env.example 建立 .env 後編輯組態
nano .env  # 或使用您喜歡的編輯器
```

請使用當前 shell 或檔案管理器把 `.env.example` 複製為 `.env`。

POSIX shell：

```bash
cp .env.example .env
```

PowerShell：

```powershell
Copy-Item .env.example .env
```

#### 基本組態

```bash
# 資料庫組態（容器內部自動組態）
# DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
# DATABASE_MAX_CONNECTIONS=10

# LLM 快速初始化，啟動後匯入 ApoRia
# 單個 provider：
# LLM_API_KEY=your-api-key-here
# LLM_BASE_URL=https://api.openai.com/v1
# LLM_MODEL=gpt-4
# 多 provider（分號分隔）：
# LLM_API_KEY=key1;key2
# LLM_BASE_URL=https://api.one/v1;https://api.two/v1
# LLM_PROTOCOL=openai;openai,api-key
# LLM_MODEL_DEEP=model-a1,model-a2;model-b1
# LLM_MODEL_NORMAL=model-a3;model-b2
# LLM_MODEL_BASIC=model-a4;model-b3

# provider 級快捷入口（推薦）
OPENAI_API_KEY=your-api-key-here
# ANTHROPIC_API_KEY=
# DEEPSEEK_API_KEY=
# DASHSCOPE_API_KEY=
# BIGMODEL_API_KEY=
# ZAI_API_KEY=

# WebSocket 組態
WS_BIND_ADDRESS=127.0.0.1:42470
WS_MAX_CONNECTIONS=100
```

#### LLM 環境變數組態說明

> **重要提示**：當前 LLM provider 組態由 ApoRia 統一管理。環境變數只作為啟動引導入口，不再是長期組態來源。

**工作機制**：

1. 當 TUI 需要自動啟動 server 時，會讀取通用 `LLM_*` 快速初始化變數，或 `OPENAI_API_KEY` 這類 provider 級變數。多 provider 組態使用分號分隔的平行陣列：`LLM_API_KEY`、`LLM_BASE_URL`、`LLM_PROTOCOL`、`LLM_MODEL_DEEP`、`LLM_MODEL_NORMAL`、`LLM_MODEL_BASIC`。程式設計套餐環境變數（如 `BIGMODEL_API_KEY_CODING_PRO`）也支援分號分隔多個金鑰，自動編號 `(#2)`、`(#3)`。自訂 provider 會在括號中顯示域名。
1. 在 server 啟動前，TUI 會先把首批 provider 組態預寫到 `res/prompts/agents/aporia/config.toml`
1. 預寫完成後，以 ApoRia 組態和 TUI 的 Models 頁面為準
1. 已存在且 API key 非空的 provider 不會被環境變數覆寫

**建議用法**：

- 使用環境變數完成首次引導
- 後續統一透過 Models 頁面或 `res/prompts/agents/aporia/config.toml` 維護

### 步驟 3: 啟動服務

```bash
# 使用 Docker Compose 啟動所有服務
docker compose up -d

# 或者使用 just 命令（如果已安裝）
just dev
```

---

## 組態

### LLM 提供商組態

Entelecheia（玄樞） 支援多個 LLM 提供商。組態您首選的提供商：

#### OpenAI

```bash
OPENAI_API_KEY=sk-...
```

#### Anthropic

```bash
ANTHROPIC_API_KEY=sk-ant-...
```

#### 本地 LLM（Ollama）

```bash
# 透過 Models 頁面或 res/prompts/agents/aporia/config.toml 組態本地 provider
# endpoint = http://localhost:11434
# model = llama2
```

### Docker 組態

```bash
# Docker socket（通常自動檢測）
DOCKER_HOST=unix:///var/run/docker.sock

# 容器設定
CONTAINER_NETWORK=entelecheia-network
CONTAINER_REGISTRY=127.0.0.1:5000
```

---

## 構建

### 開發構建

```bash
# 快速開發構建
just build-dev
```

### 生產構建

```bash
# 最佳化的發布構建
just build
```

### 構建特定組件

```bash
# 僅構建伺服器
cargo build -p scepter

# 僅構建 TUI
cargo build -p entelecheia-tui

# 構建特定代理
cargo build -p haplotes
```

### 構建產物

構建完成後，您將找到：

- **二進位檔案**: `target/debug/` 或 `target/release/`
- **Docker 映像**: 在 `just dev` 期間自動構建

---

## 執行

### 開發模式

```bash
# 啟動完整開發環境（包含 TUI）
just dev

# 僅啟動伺服器（無 TUI）
just dev --no-tui

# 清潔啟動（刪除所有資料）
just dev-clean
```

### 生產模式

```bash
# 啟動伺服器
just server

# 啟動 TUI 客戶端
just tui

# 啟動所有代理
just agents-up
```

### 終端相容性參數

TUI 依賴 ANSI 跳脫序列、滑鼠事件和影像渲染（Sixel/Kitty 協定）。在受限的終端環境中——如 SSH 工作階段、序列埠控制台、CI 執行器或舊版終端模擬器——可以使用三個漸進式降級參數：

#### `--no-image-render`

停用所有影像渲染。其餘功能——顏色、滑鼠、差異刷新——保持完全正常。

```bash
just tui -- --no-image-render
```

適用場景：終端支援顏色和滑鼠，但缺少 Sixel/Kitty 影像協定支援（最常見的情況）。

#### `--no-ansi`

停用滑鼠擷取和特殊按鍵監聽。顏色和差異（部分）螢幕刷新保留。當滑鼠事件干擾終端選取、複製貼上或回滾歷史時很有用。

```bash
just tui -- --no-ansi
```

適用場景：需要顏色但滑鼠擷取造成問題（終端多工器、`screen`、基礎 `tmux` 組態等）。

#### `--no-ansi-pure`

純單色模式——最激進的降級。停用所有 ANSI 顏色（全域強制 `Color::Reset`），停用滑鼠擷取，每影格進行全螢幕重繪。啟動畫面 Logo 替換為純 ASCII 藝術字版本。此參數隱含 `--no-ansi`。

```bash
just tui -- --no-ansi-pure
```

適用場景：透過最小化終端支援的 SSH、序列埠控制台、`docker exec`、CI 環境執行，或任何不能正確處理 ANSI 顏色程式碼的終端。

#### 參數對比

| 功能 | 預設 | `--no-image-render` | `--no-ansi` | `--no-ansi-pure` |
| --- | --- | --- | --- | --- |
| 顏色 | 完整 | 完整 | 完整 | 停用 |
| 滑鼠擷取 | 是 | 是 | 否 | 否 |
| 影像渲染 | 是 | 否 | 否 | 否 |
| 螢幕刷新 | 差異 | 差異 | 差異 | 全螢幕重繪 |
| 啟動 Logo | ANSI 彩色 | ANSI 彩色 | ANSI 彩色 | 純 ASCII 藝術字 |

### 服務管理

```bash
# 檢查服務狀態
just dev-status

# 檢視日誌
just dev-logs

# 停止服務
just dev-down

# 強制終止所有服務
just dev-kill
```

---

## 資料庫管理

### 初始化資料庫

```bash
# 建立資料庫
just db-create

# 執行遷移
just db-migrate

# 使用種子資料初始化
just db-init
```

### 資料庫操作

```bash
# 檢查資料庫狀態
just db-status

# 備份資料庫
just db-backup

# 恢復資料庫
just db-restore backups/backup_xxx.sql

# 重設資料庫（警告：刪除所有資料）
just db-reset
```

### 遷移管理

```bash
# 建立新遷移
cargo test -p scepter test_create_migration -- --nocapture --ignored

# 回滾上次遷移
just db-migrate-down
```

---

## 開發環境

### 環境設定

```bash
# 初始化所有依賴
just init

# 檢查 Python 依賴

# 格式化程式碼
just fmt

# 執行程式碼檢查
just clippy
```

### 測試

```bash
# 執行所有測試
just test

# 執行特定類型的測試
just test unit
just test integration
just test e2e
just test llm-providers

# 詳細輸出
just test verbose
```

### 程式碼品質

```bash
# 格式化程式碼
just fmt

# 檢查格式
just fmt-check

# 執行 clippy
just clippy

# 類型檢查
just check
```

---

## 部署

### Docker 部署

#### 構建映像

```bash
docker build -t entelecheia:latest .
```

#### 執行容器

```bash
docker run -d --name entelecheia \
  --env-file .env \
  -p 8424:8424 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  entelecheia:latest
```

### Docker Compose 部署

```bash
# 啟動所有服務
docker compose up -d

# 檢視日誌
docker compose logs -f

# 停止服務
docker compose down
```

---

## 故障排除

### 常見問題

#### Docker 權限被拒絕

```bash
# 將使用者新增到 docker 組
sudo usermod -aG docker $USER

# 登出並重新登入
```

#### 埠已被佔用

```bash
# 檢查佔用埠的處理程序
lsof -i :8424

# 終止處理程序
kill -9 <PID>
```

#### 構建失敗

```bash
# 清理構建產物
cargo clean

# 更新依賴
cargo update

# 重新構建
just build
```

#### 容器無法啟動

```bash
# 檢查 Docker 日誌
docker compose logs

# 重新構建容器
docker compose down
docker compose build --no-cache
docker compose up -d
```

### 獲取幫助

1. 搜尋 [GitHub Issues](https://github.com/celestia-island/entelecheia/issues)
1. 加入我們的[討論區](https://github.com/celestia-island/entelecheia/discussions)

---

## 執行 Webhook 機器人

Webhook 機器人位於 `plugins/github-webhook/` 下。每個平台都有獨立的目錄。

### 前置條件

- Python 3.10+（當前機器人）
- Node.js 18+（未來的 TypeScript 遷移）
- 各平台的 bot token（參見 [Webhook 組態指南](webhook-setup.md)）

### 執行單個機器人

```bash
# GitHub
cd plugins/github-webhook/github
pip install -r requirements.txt
python bot.py

# Gitee
cd plugins/github-webhook/gitee
pip install -r requirements.txt
python bot.py

# Discord
cd plugins/github-webhook/discord
pip install -r requirements.txt
python bot.py
```

### 執行所有機器人

```bash
just webhooks-up
```

### 環境變數

複製範例環境檔案並進行組態：

```bash
cp plugins/github-webhook/.env.example plugins/github-webhook/.env
```

各平台的具體組態詳情請參見 [Webhook 組態指南](webhook-setup.md)。

---

## 下一步

- 閱讀[基礎指南](fundamentals.md)以了解架構
- 瀏覽[代理文件](../../agents/)以了解可用的代理

---

**祝您構建愉快！** 🚀
