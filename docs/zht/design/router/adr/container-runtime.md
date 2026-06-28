# 容器執行環境客戶端（Docker/Podman）

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

管理容器（Docker、Podman）是通用連線管理器的關鍵使用案例。操作包括列出容器、執行 exec shell、檢視日誌和連接埠轉發。

## 決策

透過 Unix socket（或 Windows 上的命名管道）使用 Docker Engine API。Podman 的 Docker 相容 API 透過相同的程式碼路徑獲得支援。定義型別化的容器模型（`ContainerInfo`、`ContainerState`、`ContainerPort`）。未來：透過 Docker exec attach 實作 `TerminalBackend`，用於互動式容器 shell。

## 後果

### 正面

- Docker API 文件完善且穩定；Podman 相容性免費獲得。

### 負面

- 需要 Docker/Podman 守護程序正在執行。
- Docker 版本之間的 API 版本差異。
