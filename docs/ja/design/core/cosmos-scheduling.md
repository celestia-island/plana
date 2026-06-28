+++
title = "Cosmosコンテナスケジューリングとトークンルーティング設計"
description = """このドキュメントはCosmosコンテナスケジューリングアーキテクチャについて説明します：ToolLocation::CosmosでマークされたMCPツールがどのようにunix-socket JSON-RPCを通じて対応するコンテナにルーティング"""
lang = "ja"
category = "design"
subcategory = "core"
+++

# Cosmosコンテナスケジューリングとトークンルーティング設計

## 概要

このドキュメントはCosmosコンテナスケジューリングアーキテクチャについて説明します：`ToolLocation::Cosmos`でマークされたMCPツールがどのようにunix-socket JSON-RPCを通じて対応するコンテナにルーティングされるか、そしてトークン（エージェント番号）システムがコンテナIDとルーティングにどのように結びつくかについてです。

## I. ツールロケーションモデル

### 二重実行環境

```mermaid
flowchart LR
    subgraph Scepter["Scepter (中央プロセス)"]
        A1[LLM Calls]
        A2[RAG Queries]
        A3[Task Management]
        A4[Credential Storage]
    end

    subgraph Cosmos["Cosmos (エージェントごとのコンテナ)"]
        P1[File System Access]
        P2[Script Execution]
        P3[Hardware Access]
        P4[REPL Sessions]
    end

    Scepter -->|ToolLocation::Scepter| Local[Local Invoke]
    Cosmos -->|ToolLocation::Cosmos| Socket[Unix Socket RPC]
```

### ToolLocation列挙型

| バリアント | 実行サイト | トランスポート |
| --- | --- | --- |
| `Scepter`（デフォルト） | プロセス内、`McpToolInvoker`経由 | 直接関数呼び出し |
| `Cosmos` | コンテナ内、`CosmosConnector`経由 | UnixソケットJSON-RPC |

### ロケーション決定基準

```mermaid
flowchart TD
    Tool[MCP Tool] --> Q1{コンテナリソースが必要か？}
    Q1 -->|はい: ファイルシステム、スクリプト、ハードウェア| Cosmos[ToolLocation::Cosmos]
    Q1 -->|いいえ: LLM、RAG、状態管理| Scepter[ToolLocation::Scepter]
```

コンテナリソース（ファイルシステム、スクリプト実行、ハードウェアアクセス）を必要とするツールは`Cosmos`とマークされます。集中サービス（LLM、RAG、タスク管理、人間との対話）は`Scepter`のままです。

## II. トークンシステムとコンテナID

### エージェント番号割り当て

```mermaid
sequenceDiagram
    participant SM as SkillChain Manager
    participant AIM as AgentIdManager
    participant SC as SnowflakeContainer
    participant PC as CosmosConnector

    SM->>AIM: エージェント番号を要求
    AIM-->>SM: 000-999トークンを割り当て
    SM->>SC: トークン用のコンテナを作成
    SC-->>SM: コンテナUUID + ソケットパス
    SM->>PC: connect(UUID, socket_path)
    PC-->>SM: 接続確立
```

### トークンプロパティ

| プロパティ | 説明 |
| --- | --- |
| フォーマット | 3桁の数字: `000`-`999` |
| アロケータ | スキルチェーン内の`AgentIdManager` |
| バインディング | スキルチェーンパネルごとに1トークン |
| 表示 | TUI統計行に`cosmos#NNN`として表示 |
| 永続性 | エージェント再起動後も存続 |

## III. リクエストルーティングフロー

### TUI発信のMCP呼び出し

```mermaid
sequenceDiagram
    participant TUI as TUI Client
    participant MSR as mcp_skill_router
    participant AM as AgentManager
    participant BI as BridgeInvoker
    participant BS as HapLotesBridgeServer
    participant PR as McpRouter (Cosmos)

    TUI->>MSR: McpMessage::CallTool(tool_name, agent_type, params)
    MSR->>AM: get_tool_location(tool_name)
    AM-->>MSR: ToolLocation

    alt ToolLocation::Cosmos
        MSR->>AM: invoke_tool(tool_name, agent_type, params)
        AM->>BI: 正しいエージェントにルーティング
        BI->>BS: HapLotesブリッジ経由で転送
        BS->>PR: bridge.call(tool_name, params)
        PR-->>BS: result
        BS-->>BI: JSON-RPCレスポンス
        BI-->>AM: McpToolResult
        AM-->>MSR: McpToolResult
        MSR-->>TUI: McpMessage::ToolResponse
    else ToolLocation::Scepter
        MSR->>AM: HapLotesゲートウェイ経由でルーティング
        AM->>AM: mcp_tools.invoke() ローカル
        AM-->>TUI: WS経由でMcpMessage::ToolResponse
    end
```

### 主要ルーティングロジック

ルーティング決定は`mcp_skill_router.rs`で行われます：

1. `agent_manager.get_tool_location(tool_name)`をチェック
1. `ToolLocation::Cosmos`でコンテナ化モードがアクティブな場合：

   - `agent_manager.invoke_tool()`を呼び出し、`BridgeInvoker` → HapLotesブリッジ → Cosmosの`McpRouter`を通じてルーティング
   - Cosmosの`McpRouter`はローカル（skemma）でディスパッチするか、リモートエージェントの場合はブリッジ経由でScepterに戻す
   - `McpMessage::ToolResponse`を直接TUIに返す

1. それ以外：HapLotesゲートウェイを通じてエージェントプロセスにルーティング

## IV. CosmosConnector / ブリッジアーキテクチャ

### HapLotesブリッジ（現在）

HapLotesブリッジはScepterとCosmosコンテナ間の**唯一の通信チャネル**です。

```mermaid
flowchart LR
    subgraph Cosmos["Cosmos (コンテナ)"]
        MR[McpRouter] -->|ToolSource::Local| SK[skemma Boa JS]
        MR -->|ToolSource::Bridge| BC[HapLotesBridgeClient]
    end

    subgraph Scepter["Scepter (ホスト)"]
        BS[HapLotesBridgeServer] --> BI[BridgeInvoker]
        BI --> AG1[Aporia]
        BI --> AG2[KaLos]
        BI --> AG3[...すべてのエージェント]
    end

    BC -->|Unix Socket JSON-RPC| BS
```

### 接続プール（CosmosConnector — Scepter側）

```mermaid
classDiagram
    class CosmosConnector {
        -connections: RwLock~HashMap~String, CosmosConnection~~
        +connect(instance_uuid, socket_path) Result
        +invoke_tool(instance_uuid, tool_name, params) Result~Value~
        +list_tools(instance_uuid) Result~Vec~String~~
        +disconnect(instance_uuid)
    }

    class CosmosConnection {
        -transport: Mutex~JsonRpcTransport~
    }

    class JsonRpcTransport {
        +send_request(request) Result~JsonRpcResponse~
    }

    CosmosConnector --> CosmosConnection
    CosmosConnection --> JsonRpcTransport
```

### JSON-RPCプロトコル

すべてのメソッド名はコンパイル時の型安全性のために`UnixMethod`列挙型を使用します：

| UnixMethodバリアント | 方向 | パラメータ |
| --- | --- | --- |
| `UnixMethod::McpCall` | Scepter → Cosmos | `{ tool_name, parameters }` |
| `UnixMethod::McpListTools` | Scepter → Cosmos | なし |
| `UnixMethod::ReplSnapshot` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::ReplRestore` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::BridgeCall` | Cosmos → Scepter | `{ tool_name, parameters }` |
| `UnixMethod::BridgeListTools` | Cosmos → Scepter | なし |

### レスポンスフォーマット

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## V. コンテナライフサイクル

```mermaid
stateDiagram-v2
    [*] --> Pending: スキルチェーン開始
    Pending --> Creating: SnowflakeManager.create()
    Creating --> Starting: コンテナ実行中
    Starting --> Connected: CosmosConnector.connect()
    Connected --> Ready: ヘルスチェック合格

    Ready --> Executing: invoke_tool()
    Executing --> Ready: 結果が返される

    Ready --> Stopping: スキルチェーン完了
    Stopping --> Disconnected: CosmosConnector.disconnect()
    Disconnected --> [*]

    Creating --> Failed: タイムアウト
    Starting --> Failed: 接続エラー
    Failed --> [*]
```

### コンテナエージェント

Cosmosコンテナ内では、skemmaのみがローカルで実行されます（Boa JSエンジン）。他のすべてのエージェントツールはHapLotesブリッジを通じてScepterにルーティングされます：

| エージェント | 役割 | Cosmos内？ |
| --- | --- | --- |
| SkeMma | スクリプト実行（Boa JS） | **ローカル**（インプロセス） |
| Aporia | LLMチャット | ブリッジ経由 → Scepter |
| KaLos | ファイルI/O | ブリッジ経由 → Scepter |
| NeiKos | コンテナ管理 | ブリッジ経由 → Scepter |
| EleOs | Web検索 | ブリッジ経由 → Scepter |
| その他すべて | 様々 | ブリッジ経由 → Scepter |

## VI. 統計行の統合

### 表示フォーマット

TUI AgentDetailPageでは、統計行に以下が表示されます：

```mermaid
flowchart LR
    BORDER["|"] --> TOK["1.2k tokens"] --> SEP1["|"] --> DUR["3.5s"] --> SEP2["|"] --> COSMOS["cosmos#042"] --> TIER["[T2]"]

    TOK -.->|"McpToolResult.token_usage"| SRC1["Token Usage"]
    DUR -.->|"Instant::now()"| SRC2["Duration"]
    COSMOS -.->|"AgentIdManager"| SRC3["Agent Number"]
    TIER -.->|"McpToolConfig.tier"| SRC4["Model Tier"]
```

| セグメント | ソース |
| --- | --- |
| `1.2k tokens` | `McpToolResult.token_usage` |
| `3.5s` | `Instant::now()`からの経過時間 |
| `cosmos#042` | `AgentIdManager`からのエージェント番号 |
| `[T2]` | `McpToolConfig.tier`からのモデルティア |

## VII. エラーハンドリング

### 障害モード

```mermaid
flowchart TD
    Call[ツール呼び出し] --> Q1{コンテナがオンライン？}
    Q1 -->|いいえ| E1[エラー: AGENT_OFFLINE]
    Q1 -->|はい| Q2{ソケット接続済み？}
    Q2 -->|いいえ| E2[エラー: Connection Lost]
    Q2 -->|はい| Q3{ツールが存在する？}
    Q3 -->|いいえ| E3[エラー: Tool Not Found]
    Q3 -->|はい| Q4{実行成功？}
    Q4 -->|いいえ| E4[エラー: Execution Failed]
    Q4 -->|はい| Result[結果を返す]

    E1 --> Fallback[フォールバック: Scepterでの実行を試行]
    E2 --> Retry[リトライ: ソケット再接続]
```

### グレースフルデグラデーション

コンテナが利用できない場合、ツールにローカル実装が登録されていれば、システムはオプションで`Scepter`ローカル実行にフォールバックできます。

## VIII. 将来の拡張

| 機能 | 説明 | 優先度 |
| --- | --- | --- |
| コンテナプーリング | スキルチェーン間でコンテナを再利用 | 中 |
| ヘルスモニタリング | 定期的なコンテナヘルスチェック | 高 |
| リソース制限 | コンテナごとのCPU/メモリ制限 | 高 |
| マルチコンテナツール | 複数のコンテナにまたがるツール | 低 |
| コンテナ移行 | 実行中のコンテナをホスト間で移動 | 低 |
