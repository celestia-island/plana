+++
title = "CLI 日誌記錄慣例"
description = """shittim-chest CLI 封裝器的日誌輸出遵循與 entelecheia 一致的慣例，使用 `tracing` 生態系統，以緊湊且易讀的格式輸出到 stderr。"""
lang = "zht"
category = "design"
subcategory = "webui"
+++

# CLI 日誌記錄慣例

## 概述

shittim-chest CLI 封裝器的日誌輸出遵循與 entelecheia 一致的慣例，使用 `tracing` 生態系統，以緊湊且易讀的格式輸出到 stderr。

## 框架選擇

| 元件 | 選擇 | 原因 |
| --- | --- | --- |
| 日誌框架 | `tracing` | Rust 生態系標準，與 entelecheia 一致 |
| 訂閱器 | `tracing-subscriber` fmt 層 | 緊湊輸出，無需 JSON 解析 |
| 時間格式 | `ShortTimer` (HH:MM:SS) | 終端機友善，與 entelecheia CLI 一致 |
| 輸出目標 | stderr | 與 stdout 分離，不干擾管線 |

## 初始化程式碼

```rust
use chrono::Local;
use tracing_subscriber::fmt::time::FormatTime;

struct ShortTimer;

impl FormatTime for ShortTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{} ", now.format("%H:%M:%S"))
    }
}

// 初始化
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(&args.log_level))
    .with_target(false)          // 隱藏模組路徑
    .with_timer(ShortTimer)      // HH:MM:SS 格式
    .compact()                   // 緊湊模式
    .with_writer(std::io::stderr) // 輸出到 stderr
    .init();
```

## 格式比較

| 模式 | 範例輸出 | 使用情境 |
| --- | --- | --- |
| CLI（目前） | `14:23:05  INFO creating network shittim-chest...` | 開發、運維 |
| 伺服器（未來） | `{"timestamp":"...","level":"INFO","message":"..."}` | 生產日誌收集 |

## --log-level 參數

CLI 接受 `--log-level` / `-l` 參數（預設 `info`）：

```text
shittim-chest --log-level debug dev
shittim-chest -l trace status
```

支援的層級：`trace`、`debug`、`info`、`warn`、`error`。

## 日誌層級使用慣例

| 層級 | 用途 | 典型 CLI 場景 |
| --- | --- | --- |
| `info` | 重要操作 | 容器建立/啟動/停止、遷移開始/完成 |
| `warn` | 潛在問題 | 遷移重試、容器存在但處於異常狀態 |
| `error` | 錯誤 | 容器崩潰、遷移失敗、網路建立失敗 |
| `debug` | 除錯資訊 | （目前未使用，保留供未來使用） |
| `trace` | 詳細流程 | （目前未使用，保留供未來使用） |

## 設計原則

1. **CLI 不吞掉錯誤**：所有錯誤透過 `anyhow::Result` 向上傳播；`main()` 自動列印錯誤鏈。
1. **每個操作開始都有日誌**：`creating network...`、`running migrations...`、`building shittim_chest...`——使用者知道 CLI 正在做什麼。
1. **每個操作完成都有確認**：`shittim-chest started on 0.0.0.0:80`、`all services started`。
1. **靜默成功的操作不記錄**：`ensure_network` 若網路已存在則不列印，以避免雜訊。
1. **容器日誌透過 Docker API 取得**：CLI 本身不寫入業務日誌，僅寫入編排操作日誌。

## 與 entelecheia 的一致性

| 功能 | entelecheia CLI | shittim-chest CLI | 一致 |
| --- | --- | --- | --- |
| 框架 | `tracing` | `tracing` | ✅ |
| 時間格式 | `ShortTimer` (HH:MM:SS) | `ShortTimer` (HH:MM:SS) | ✅ |
| 輸出目標 | stderr | stderr | ✅ |
| 緊湊模式 | `.compact()` | `.compact()` | ✅ |
| 隱藏目標 | `.with_target(false)` | `.with_target(false)` | ✅ |
| --log-level | 支援 | 支援 | ✅ |

兩個專案的 CLI 日誌輸出在視覺上完全相同，使開發者能輕鬆在兩個專案之間切換。
