+++
title = "CLI 使用指南"
description = """`entelecheia-cli` 是 Entelecheia（玄樞）多智慧體協作平台的命令列介面。它透過 Unix socket JSON-RPC 與 scepter 伺服器通訊，提供聊天互動、服務生命週期管理、智慧體控制、組態等功能。"""
lang = "zht"
category = "guides"
subcategory = "core"
+++

# CLI 使用指南

`entelecheia-cli` 是 Entelecheia（玄樞）多智慧體協作平台的命令列介面。它透過 Unix socket JSON-RPC 與 scepter 伺服器通訊，提供聊天互動、服務生命週期管理、智慧體控制、組態等功能。

> 說明：CLI 目前尚未達到與 TUI 完全同等的功能。當前狀態請參見 [ARCHITECTURE.md](../../ARCHITECTURE.md)。

---

## 目錄

- [安裝](#安裝)
- [基本用法](#基本用法)
- [全域選項](#全域選項)
- [聊天命令](#聊天命令)
- [智慧體管理](#智慧體管理)
- [服務生命週期](#服務生命週期)
- [組態](#組態)
- [連接上下文](#連接上下文)
- [狀態與監控](#狀態與監控)
- [訂閱（Layer3）](#訂閱layer3)
- [執行智慧體](#執行智慧體)
- [時間線](#時間線)
- [Docker 映像](#docker-映像)
- [進階用法](#進階用法)

---

## 安裝

### 從原始碼構建

```bash
# 複製倉庫
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia

# 構建 CLI 二進位檔案
cargo build --package entelecheia-cli

# 或使用 just
just cli
```

二進位檔案位於 `target/debug/entelecheia-cli`（debug）或 `target/release/entelecheia-cli`（release）。

### 預構建二進位檔案

預構建的二進位檔案可從 [GitHub Releases](https://github.com/celestia-island/entelecheia/releases) 獲取。下載適合您平台的壓縮套件，並將二進位檔案放入 `PATH` 中。

---

## 基本用法

```bash
# 顯示幫助
entelecheia-cli --help

# 透過技能鏈傳送訊息
entelecheia-cli send 解釋一下這個專案的架構

# 透過管道傳送訊息
echo "總結這個檔案" | entelecheia-cli send

# 檢查系統狀態
entelecheia-cli status
```

---

## 全域選項

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | 日誌級別（trace、debug、info、warn、error） | `warn` |
| `-d, --daemon` | 背景派發命令後立即退出 | — |
| `-c, --clean` | 清理 Cosmos 容器和 socket 檔案 | — |
| `-a, --auto-approve` | 自動核准操作（確保伺服器正在執行） | — |
| `-t, --table` | 人類可讀表格輸出（ANSI 格式） | 預設 |
| `-j, --json` | JSON 輸出（機器可讀） | — |
| `-r, --raw` | 原始純文字輸出（無格式） | — |
| `--format <FORMAT>` | 輸出格式（table、json、raw） | `table` |

輸出格式選項：

- `table` — 人類可讀的表格輸出
- `json` — 機器可讀的 JSON 輸出

**範例：**

```bash
# 清理容器
entelecheia-cli --clean

# 以 JSON 格式獲取狀態
entelecheia-cli status --format json

# 除錯模式傳送訊息
entelecheia-cli -l debug send "除錯連接問題"

# 背景模式執行 agent（立即返回）
entelecheia-cli -d run my-agent --ci
```

---

## 聊天命令

`chat` 子命令管理工作階段智慧體系統的對話互動。

### 傳送訊息

```bash
entelecheia-cli chat send [OPTIONS]
```

| 選項 | 描述 |
| --- | --- |
| `-m, --message <MSG>` | 要傳送的訊息文字 |
| `--stdin` | 從標準輸入讀取訊息 |
| `-f, --file <PATH>` | 從檔案讀取訊息 |

每次只能使用一個輸入來源。

**範例：**

```bash
# 直接傳送訊息
entelecheia-cli chat send -m "你好，你能做什麼？"

# 從標準輸入
echo "分析 src/main.rs 中的程式碼" | entelecheia-cli chat send --stdin

# 從檔案
entelecheia-cli chat send -f ./prompts/review.txt
```

`chat send` 命令將訊息透過**技能鏈**——協調多個智慧體的核心執行管道。執行過程中會透過旋轉動畫顯示進度。

### 聊天歷史

```bash
entelecheia-cli chat history [OPTIONS]
```

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `--conversation <ID>` | 按工作階段 ID 篩選 | — |
| `--agent <TYPE>` | 按智慧體類型篩選 | — |
| `--role <ROLE>` | 按角色篩選（user/assistant/system） | — |
| `--from <ISO8601>` | 開始日期時間（ISO 8601） | — |
| `--to <ISO8601>` | 結束日期時間（ISO 8601） | — |
| `--limit <N>` | 返回的最大訊息數 | `50` |
| `--offset <N>` | 分頁偏移量 | `0` |

**範例：**

```bash
entelecheia-cli chat history --agent ApoRia --limit 20 --from 2026-05-01T00:00:00Z
```

### 最近訊息

```bash
entelecheia-cli chat recent [OPTIONS]
```

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `--timeline <ID>` | 按時間線/工作階段 ID 篩選 | — |
| `--agent <TYPE>` | 按智慧體類型篩選 | — |
| `--limit <N>` | 返回的最大訊息數 | `20` |

---

## 智慧體管理

管理智慧體生命週期（列出、啟動、停止、重啟）。

```bash
entelecheia-cli agent <COMMAND>
```

### 命令

```bash
# 列出所有智慧體及其狀態
entelecheia-cli agent list

# 按類型啟動智慧體
entelecheia-cli agent start <AGENT_TYPE>

# 停止正在執行的智慧體
entelecheia-cli agent stop <AGENT_TYPE>

# 重啟智慧體
entelecheia-cli agent restart <AGENT_TYPE>
```

**可用的智慧體類型：** ApoRia、EleOs、EpieiKeia、Haplotes、HubRis、Kalos、NeiKos、OreXis、PhiLia、Polemos、SkeMma、SkoPeo。

> 說明：智慧體作為函式庫 crate 在 scepter 執行時內部執行，而非獨立可執行檔案。`agent start` 命令嘗試產生一個與智慧體名稱匹配的二進位檔案，這主要適用於智慧體被編譯為單獨二進位檔案的情況。實際使用中，智慧體透過 scepter 伺服器啟用。

---

## 服務生命週期

使用 Docker 容器管理 Entelecheia（玄樞）服務棧。

### 初始化服務

```bash
entelecheia-cli init [OPTIONS]
```

設定完整的服務棧：PostgreSQL（含 pgvector）、Docker 登錄表、scepter 伺服器和 WebUI。建立所需的 Docker 網路並拉取/構建映像。

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `--prefix <STR>` | 容器名稱前綴 | `e-` |
| `--source-build` | 從原始碼構建映像而非拉取 | `false` |
| `--webui-port <PORT>` | WebUI 埠 | `3424` |

**範例：**

```bash
entelecheia-cli init --prefix ent- --webui-port 8080
```

### 啟動所有服務

```bash
entelecheia-cli serve [OPTIONS]
```

啟動所有之前已初始化的容器。需要先執行 `init`。

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `--prefix <STR>` | 容器名稱前綴 | `e-` |
| `--webui-port <PORT>` | WebUI 埠 | `3424` |

### 停止所有服務

```bash
entelecheia-cli stop [OPTIONS]
```

按順序停止所有正在執行的容器：webui → scepter → registry → postgres。

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `--prefix <STR>` | 容器名稱前綴 | `e-` |

### 僅啟動 WebUI

```bash
entelecheia-cli webui [OPTIONS]
```

僅啟動或建立 WebUI 容器。

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `--prefix <STR>` | 容器名稱前綴 | `e-` |
| `--webui-port <PORT>` | WebUI 埠 | `3424` |

---

## 組態

檢視和驗證系統組態。

### 顯示組態

```bash
entelecheia-cli config show
```

顯示當前組態，包括：

- 資料庫 URL 和連接設定
- ApoRia LLM 提供商組態（名稱、模型、端點）
- WebSocket 繫結位址
- 日誌級別

API 金鑰在輸出中被遮蔽（顯示為 `***`）。

### 驗證組態

```bash
entelecheia-cli config validate
```

執行驗證檢查：

- 資料庫 URL 已設定
- 至少組態了一個具有完整設定的 ApoRia 提供商
- WebSocket 繫結位址已設定

返回通過/失敗結果，並附帶任何問題的詳細資訊。

**輸出範例：**

```text
Validate Configuration:

Validating database configuration...
  [ OK ]  Database URL set

Validating ApoRia LLM configuration...
  [ OK ]  ApoRia providers configured

Validating WebSocket configuration...
  [ OK ]  WebSocket Bind Address set

[ OK ]  Configuration validation passed
```

---

## 連接上下文

`context` 子命令用於管理命名的連接設定檔，允許您在本地（Unix socket）和遠端（WebSocket）scepter 伺服器之間切換。其使用方式與 Docker 的 `docker context` 命令類似。

### 概念

一個**上下文**是一個命名的設定檔，記錄了 CLI 如何連接 scepter 伺服器：

- **local** — Unix socket 連接（預設，自動解析為 `/run/.../entelecheia-tui.sock`）
- **remote** — 帶 Bearer token 認證的 WebSocket 連接

上下文儲存在 `~/.config/entelecheia/contexts/contexts.toml` 中。

### 列出上下文

```bash
entelecheia-cli context list
```

當前活動的上下文以 `*` 標記。

### 顯示當前上下文

```bash
entelecheia-cli context show
```

顯示活動上下文的類型、socket 路徑、WS URL 和描述資訊。

### 建立上下文

```bash
# 遠端 WebSocket 上下文
entelecheia-cli context create staging \
  --ws-url ws://scepter.example.com:8424/ws \
  --bearer-token <TOKEN> \
  --description "Staging server"

# 額外的本地上下文
entelecheia-cli context create dev --description "Development server"
```

從遠端伺服器獲取 Bearer token：

```bash
# 在伺服器機器上
docker exec e-scepter cat /home/entelecheia/.config/entelecheia/scepter.token
```

### 切換上下文

```bash
entelecheia-cli context use staging
# 此後所有命令（send、status、chat 等）都將透過 staging 連接路由
```

### 移除上下文

```bash
entelecheia-cli context remove staging
```

`default` 上下文不可被移除。

### 範例工作流

```bash
# 檢視當前上下文
entelecheia-cli context list

# 為預發布伺服器建立遠端上下文
entelecheia-cli context create staging \
  --ws-url ws://192.168.1.100:8424/ws \
  --bearer-token $(cat /path/to/token)

# 切換到預發布環境
entelecheia-cli context use staging

# 透過遠端伺服器傳送訊息
entelecheia-cli send "列出當前待辦事項"

# 檢查遠端伺服器狀態
entelecheia-cli status

# 切換回本地
entelecheia-cli context use default
```

---

## 狀態與監控

### 系統狀態

```bash
entelecheia-cli status
```

顯示：

- 伺服器版本
- 連接狀態（socket 狀態）
- LLM 提供商摘要
- WebSocket 繫結位址
- 智慧體列表及執行/停止狀態
- 系統資源（記憶體使用量、平均負載）

### 狀態路徑查詢

`status` 命令接受類路徑參數來查詢特定子系統。語法支援按 agent 範圍的時間線、聊天歷史檢查和裝置列舉。

```bash
entelecheia-cli status <PATH> [--raw]
```

| 路徑語法 | 描述 |
| --- | --- |
| `timeline.#agent[-N]` | 顯示某 agent 最近 N 次 skill 呼叫記錄 |
| `timeline.#agent[N][M]` | 顯示第 N 次 skill 呼叫中的第 M 個 MCP/工具呼叫 |
| `history[-N]` | 顯示最近 N 條聊天訊息（所有角色） |
| `history[-N].body` | 顯示倒數第 N 條訊息的正文 |
| `device` | 列出所有 Polemos 識別的邊緣裝置 |
| `device[N]` | 顯示第 N 個 Polemos 裝置的詳細資訊 |

**範例：**

```bash
# Haplotes #001 agent 最近 30 次 skill 排程歷史
entelecheia-cli status timeline.#hap_lotes.001[-30]

# 第 3 次 skill 呼叫的第 2 個 MCP/工具呼叫
entelecheia-cli status timeline.#hap_lotes.001[3][2]

# 最近 30 條訊息
entelecheia-cli status history[-30]

# 倒數第 3 條訊息正文（純文字）
entelecheia-cli status history[-3].body --raw

# 所有 Polemos 裝置
entelecheia-cli status device

# 第 3 個 Polemos 裝置詳情
entelecheia-cli status device[3]
```

> **Shell 注意:** 在 bash/zsh 中，請用單引號包裹含 `[...]` 的路徑以防 glob 展開：`entelecheia-cli status 'history[-30]'`。`#` 字元嵌在單詞中間時無需轉義。在 fish shell 中，以上路徑均無需引號。

狀態路徑查詢透過 Unix socket JSON-RPC 與伺服器通訊。`timeline.*` 和 `history.*` 查詢需要伺服器正在執行。`device` 查詢需要伺服器上有 Polemos 工作區註冊。

### 檢視日誌

```bash
entelecheia-cli logs [OPTIONS]
```

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `-a, --agent <NAME>` | 按智慧體名稱篩選日誌 | 所有智慧體 |
| `-l, --lines <N>` | 顯示的行數（尾部） | `100` |

**範例：**

```bash
# 顯示所有智慧體日誌的最後 200 行
entelecheia-cli logs -l 200

# 顯示 ApoRia 日誌
entelecheia-cli logs -a ApoRia
```

日誌從 `./logs/` 目錄讀取。每個智慧體都有各自的日誌檔案（`ApoRia.log`、`EleOs.log` 等）。

---

## 訂閱（Layer3）

管理 Layer3 智慧體訂閱——可以安裝和執行的外部智慧體套件。

### 列出訂閱

```bash
entelecheia-cli subscribe list
```

顯示所有已組態的訂閱，包括狀態（已安裝/待處理）、啟用狀態、自動更新設定和來源。

### 新增訂閱

```bash
entelecheia-cli subscribe add [OPTIONS]
```

| 選項 | 描述 |
| --- | --- |
| `--name <NAME>` | 訂閱名稱（必需） |
| `--source <SOURCE>` | 來源類型：`official`、`github` 或 `url`（必需） |
| `--repository <REPO>` | GitHub 倉庫（用於 github 來源） |
| `--url <URL>` | 直接 URL（用於 url 來源） |
| `--version <VER>` | 版本約束 |
| `--auto-update` | 啟用自動更新 |
| `--disabled` | 新增為停用狀態 |

**範例：**

```bash
entelecheia-cli subscribe add --name my-agent --source github --repository user/repo
```

### 移除訂閱

```bash
entelecheia-cli subscribe remove <NAME>
```

### 同步訂閱

```bash
# 同步所有訂閱
entelecheia-cli subscribe sync

# 同步特定訂閱
entelecheia-cli subscribe sync --name my-agent
```

### 自動更新

```bash
entelecheia-cli subscribe auto-update
```

更新所有啟用了 `auto_update` 的訂閱。

---

## 執行智慧體

```bash
entelecheia-cli run <AGENT> [OPTIONS]
```

執行 Layer3 智慧體腳本。在當前目錄中查找 `.amphoreus/<AGENT>/run.py`。首次執行時會執行預檢稽核。

| 選項 | 描述 |
| --- | --- |
| `--ci` | 啟用 CI 模式 |
| `--auto-pr` | 啟用自動 PR 模式 |
| `--dry-run` | 試執行（不進行實際變更） |
| `--providers <LIST>` | 逗號分隔的提供商列表 |
| `--output-dir <DIR>` | 輸出目錄 |

**範例：**

```bash
# 以試執行模式執行 Layer3 智慧體
entelecheia-cli run my-agent --dry-run

# 使用指定提供商執行
entelecheia-cli run my-agent --providers openai,anthropic

# CI 模式並自動提交 PR
entelecheia-cli run my-agent --ci --auto-pr

# 背景模式執行（立即返回，子處理程序背景執行）
entelecheia-cli -d run my-agent --ci --auto-pr
```

### 背景模式（`-d` / `--daemon`）

背景模式旗標會使 CLI 以剝離 `--daemon` 參數的方式重新產生一個分離的子處理程序，並立即返回。子處理程序繼承原始命令並獨立執行。之後可使用 `status` 檢視進度。

適用於 `run`、`init`、`deploy` 等長時間執行的操作：

```bash
# 背景派發 agent 執行
entelecheia-cli -d run my-agent

# 背景派發服務初始化
entelecheia-cli -d init --prefix prod-

# 稍後檢視狀態
entelecheia-cli status
entelecheia-cli status history[-5]
```

---

## 時間線

檢視工作階段時間線。

### 列出時間線

```bash
entelecheia-cli timeline list [OPTIONS]
```

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `--agent <TYPE>` | 按智慧體類型篩選 | — |
| `--limit <N>` | 最大結果數 | `50` |
| `--offset <N>` | 分頁偏移量 | `0` |

### 顯示時間線詳情

```bash
entelecheia-cli timeline show <CONVERSATION_ID> [OPTIONS]
```

| 選項 | 描述 | 預設值 |
| --- | --- | --- |
| `--include-messages` | 輸出中包含訊息 | `true` |

---

## Docker 映像

```bash
entelecheia-cli init-docker-images [OPTIONS]
```

構建或拉取平台所需的 Docker 映像。

| 選項 | 描述 |
| --- | --- |
| `--source-build` | 從原始碼構建映像而非拉取 |
| `--tag <TAG>` | 映像標籤（預設：`latest`） |

**範例：**

```bash
# 從原始碼構建所有映像
entelecheia-cli init-docker-images --source-build

# 使用自訂標籤拉取
entelecheia-cli init-docker-images --tag v0.2.0
```

管理的映像：

- `entelecheia` — 編排伺服器（含內嵌 cosmos 執行時）
- `pgvector/pgvector` — 帶向量擴充的 PostgreSQL

---

## 進階用法

### 用於腳本的 JSON 輸出

使用 `--format json` 獲取機器可讀的輸出，可管道傳輸至 `jq` 或其他工具：

```bash
entelecheia-cli status --format json | jq '.server_version'
entelecheia-cli chat history --format json | jq '.messages[].content'
```

### 鏈式清理與初始化

```bash
# 完全拆除並重建
entelecheia-cli --clean && entelecheia-cli init --prefix my-
```

### 除錯模式

```bash
# 啟用 trace 級別日誌進行除錯
entelecheia-cli -l trace send "測試訊息"
```

### 與 TUI 搭配使用

CLI 與 TUI 連接到同一個 scepter 伺服器。兩者可以同時使用：

- 啟動 TUI 進行互動式工作階段：`cargo run --bin entelecheia-tui`
- 使用 CLI 進行腳本編寫、自動化和快速查詢

---

## 故障排除

### "No command specified"

執行 `--help` 檢視可用命令，或使用 `send "訊息"` 快速傳送訊息。

### "Failed to connect to Docker"

確保 Docker（或 Podman）正在執行：

```bash
docker info
docker run hello-world
```

### "Agent binary not found"

智慧體是 scepter 執行時的內部函式庫 crate，而非獨立二進位檔案。啟動 scepter 伺服器以啟用智慧體：

```bash
entelecheia-cli init && entelecheia-cli serve
```

### "No LLM providers configured"

透過環境變數設定 ApoRia 提供商組態。有關提供商設定說明，請參見[構建指南](building.md)。

### "Configuration validation failed"

執行 `entelecheia-cli config validate` 檢視哪些檢查失敗。常見問題：

- 缺少 `DATABASE_URL` 環境變數
- ApoRia 提供商設定不完整（名稱、模型、`api_key`）
- 缺少 WebSocket 繫結位址
