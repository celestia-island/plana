+++
title = "増分同期アーキテクチャ"
description = """Automerge CRDTに基づくマルチクライアント状態増分同期機構。リアルタイム増分更新と接続/再接続時の完全同期をサポートし、全TUIパネルをカバーする"""
lang = "ja"
category = "design"
subcategory = "core"
+++

# 増分同期アーキテクチャ

## 概要

Automerge CRDTに基づくマルチクライアント状態増分同期機構。リアルタイム増分更新と接続/再接続時の完全同期をサポートし、全TUIパネルをカバーする。

## アーキテクチャ図

```mermaid
flowchart TB
    subgraph Clients["TUIクライアント（複数）"]
        C1["クライアント1"]
        C2["クライアント2"]
        C3["クライアントN"]
    end

    subgraph Server["サーバー"]
        SM["SyncManager<br/>単一状態ツリー"]
        BH["BroadcastHelper"]
        WS["WebSocketブロードキャスト"]
        REG["StateRegistry<br/>完全状態"]
    end

    subgraph Storage["Automerge CRDT"]
        AD["AgentDoc<br/>エージェント単位"]
        V["バージョンベクトル"]
    end

    %% 完全同期リクエスト（プルモード）
    C1 -->|"接続時"| WS
    C2 -->|"接続時"| WS
    C3 -->|"接続時"| WS

    WS -->|"RequestFullSnapshot"| BH
    BH -->|"list_agents"| SM
    SM -->|"AgentSnapshot"| BH
    BH -->|"broadcast"| WS
    WS -->|"AgentSnapshot"| C1
    WS -->|"AgentSnapshot"| C2
    WS -->|"AgentSnapshot"| C3

    %% 増分更新（プッシュモード）
    SM -->|"状態変更"| BH
    BH -->|"update_agent"| SM
    SM -->|"AgentPatch生成"| BH
    BH -->|"broadcast"| WS
    WS -->|"AgentPatch"| C1
    WS -->|"AgentPatch"| C2
    WS -->|"AgentPatch"| C3

    %% Automergeストレージ
    SM <-->|"agent_docs"| AD
    SM <-->|"version"| V

    style SM fill:#e1f5fe
    style BH fill:#fff3e0
    style WS fill:#f3e5f5
    style AD fill:#e8f5e9
    style REG fill:#fff9c4
```

## 同期戦略マトリックス

| パネル | 同期方式 | トリガー | 頻度 | メッセージタイプ |
| --- | --- | --- | --- | --- |
| **エージェントタイムライン** | 増分 + 完全 | 接続時同期 + リアルタイムプッシュ | 接続時 / リアルタイム | `AgentPatch` / `GlobalSnapshot` |
| **コンテナ** | 増分 + 完全 | 接続時同期 + リアルタイムプッシュ | 接続時 / リアルタイム | `ContainerPatch` / `GlobalSnapshot` |
| **タスク** | 増分 + 完全 | 接続時同期 + リアルタイムプッシュ | 接続時 / リアルタイム | `TaskPatch` / `GlobalSnapshot` |
| **モデル一覧** | 完全 | クライアント能動リクエスト | パネル表示時 | `ModelsSnapshot` |
| **プロバイダ設定** | 完全 | クライアント能動リクエスト | パネル表示時 | `ProvidersSnapshot` |

## メッセージフロー

### 増分更新フロー（エージェント）

```mermaid
sequenceDiagram
    participant Agent as エージェントランタイム
    participant SM as SyncManager
    participant BH as BroadcastHelper
    participant WS as WebSocket
    participant Client as TUIクライアント

    Agent->>SM: 状態更新
    SM->>SM: update_agent()
    SM->>SM: AgentPatch生成
    SM->>BH: パッチ返却
    BH->>WS: broadcast(AgentPatch)
    WS->>Client: AgentPatchメッセージ
    Client->>Client: apply_agent_patch()
    Client->>Client: UI更新
```

### 完全同期フロー

```mermaid
sequenceDiagram
    participant Client as TUIクライアント
    participant WS as WebSocket
    participant BH as BroadcastHelper
    participant SM as SyncManager
    participant Registry as 状態レジストリ

    Note over Client: 接続/再接続時
    Client->>WS: RequestGlobalSnapshot
    WS->>BH: send_full_snapshot()
    BH->>Registry: list_agents()
    Registry-->>BH: Vec<AgentInfo>
    BH->>SM: create_snapshot(agents)
    SM-->>BH: AgentSnapshot
    BH->>WS: broadcast(AgentSnapshot)
    WS-->>Client: AgentSnapshotメッセージ
    Client->>Client: ローカル状態を置換
```

### モデル一覧同期フロー

```mermaid
sequenceDiagram
    participant Client as TUIクライアント
    participant WS as WebSocket
    participant Server as サーバー

    Note over Client: モデルパネル表示時
    Client->>WS: モデル一覧リクエスト
    WS->>Server: モデル設定クエリ
    Server-->>WS: モデル一覧
    WS-->>Client: ModelsSnapshotメッセージ
    Client->>Client: モデルパネル更新
```

### コンテナ完全同期フロー

```mermaid
sequenceDiagram
    participant Client as TUIクライアント
    participant WS as WebSocket
    participant Server as サーバー

    Note over Client: コンテナパネル表示時
    Client->>WS: RequestContainerSnapshot
    WS->>Server: コンテナ状態クエリ
    Server-->>WS: ContainerSnapshot
    WS-->>Client: ContainerSnapshotメッセージ
    Client->>Client: ローカルコンテナ状態を置換
```

### タスク完全同期フロー

```mermaid
sequenceDiagram
    participant Client as TUIクライアント
    participant WS as WebSocket
    participant Server as サーバー

    Note over Client: タスクパネル表示時
    Client->>WS: RequestTasksSnapshot
    WS->>Server: タスク状態クエリ
    Server-->>WS: TasksSnapshot
    WS-->>Client: TasksSnapshotメッセージ
    Client->>Client: ローカルタスク状態を置換
```

## データ構造

### AgentPatch（増分更新）

```rust
pub struct AgentPatch {
    pub agent_id: String,
    pub version: u64,
    pub llm_working_changed: Option<bool>,
    pub work_status: Option<String>,
    pub current_model: Option<String>,
    pub token_usage_delta: Option<(u32, u32)>,
    pub token_usage_absolute: Option<(u32, u32)>,
    pub request_state: Option<RequestState>,
    pub cpu_usage: Option<f64>,
    pub memory_mb: Option<u64>,
}
```

### AgentSnapshot（完全スナップショット）

```rust
pub struct AgentSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
}
```

### GlobalSnapshot（グローバルスナップショット）

```rust
pub struct GlobalSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
    pub models: Vec<ModelInfo>,
    pub providers: Vec<ProviderInfo>,
    pub active_tasks: Vec<TaskInfo>,
}
```

### ModelsSnapshot（モデル一覧）

```rust
pub struct ModelsSnapshot {
    pub models: Vec<ModelInfo>,
}
```

### ContainerPatch（コンテナ状態増分）

```rust
pub struct ContainerPatch {
    pub container_id: String,
    pub version: u64,
    pub status_changed: Option<String>,
    pub cpu_usage_changed: Option<f64>,
    pub memory_usage_changed: Option<u64>,
}
```

### ContainerSnapshot（コンテナ状態完全）

```rust
pub struct ContainerSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub containers: Vec<ContainerInfo>,
}
```

### TaskPatch（タスク状態増分）

```rust
pub struct TaskPatch {
    pub task_id: Uuid,
    pub version: u64,
    pub status_changed: Option<String>,
    pub progress_changed: Option<u8>,
}
```

### TasksSnapshot（タスク状態完全）

```rust
pub struct TasksSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub tasks: Vec<TaskInfo>,
}
```

## 同期戦略

| 種別 | 方向 | トリガー | 頻度 |
| --- | --- | --- | --- |
| エージェント増分更新 | サーバー → クライアント | 状態変更 | リアルタイム |
| エージェント完全同期 | サーバー → クライアント | 接続時 | 接続/再接続時 |
| コンテナ増分 | サーバー → クライアント | 状態変更 | リアルタイム |
| コンテナ完全同期 | サーバー → クライアント | 接続時 | 接続/再接続時 |
| タスク増分 | サーバー → クライアント | 状態変更 | リアルタイム |
| タスク完全同期 | サーバー → クライアント | 接続時 | 接続/再接続時 |
| モデル一覧 | クライアント → サーバー | 能動リクエスト | パネル表示時 |
| プロバイダ設定 | クライアント → サーバー | 能動リクエスト | パネル表示時 |

## 主要機能

- **単一状態ツリー**: サーバーは単一の `SyncManager` を保持し、全クライアントが同一の状態更新を受信する
- **CRDT競合解決**: Automergeに基づく自動競合解決
- **増分更新**: 変更フィールドのみを転送しネットワークトラフィックを削減
- **結果整合性**: 接続時の完全同期が結果整合性を保証
- **オンデマンドプル**: モデルとプロバイダはパネル表示時にオンデマンドでリクエストされ、不要なネットワーク転送を回避
- **ホーム画面同期**: エージェント、コンテナ、タスクはホーム画面に表示されるため接続時に同期

## 実装状況

- ✅ エージェント増分/完全同期
- ✅ モデル一覧同期
- ✅ プロバイダ設定同期
- ✅ コンテナ増分/完全同期
- ✅ タスク増分/完全同期
- ✅ 状態永続化（/tmpストレージ、再起動時リロード）
