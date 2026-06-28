+++
title = "産業回廊統合計画"
description = """> 目標: 本システムは、完全に未知の産業用実証回廊に対して、自律的な自己インターフェースを実証しなければなりません"""
lang = "ja"
category = "architecture"
subcategory = "core"
+++

# 産業回廊自律統合計画

> **目標**: 本システムは、完全に未知の産業用実証回廊に対して**自律的な自己インターフェース**を実証しなければなりません — ハードウェアの発見、データモデルの推論、監視設定の生成、アラーム→応答ループの閉鎖 — を手動の機器ごとのエンジニアリングなしで行います。
> **政府の厳格な期限**: この能力は政府プロジェクトのマイルストーンに紐付けられています。

---

## 残存作業

完全な発見 → 推論 → 監視 → アラーム → **書き込み承認**チェーンが出荷されました（フェーズA.1–A.3、B、C、D.1、**D.2 ✓**）。残る作業は**エンドツーエンドのドッグフード検証（フェーズE）**のみです — コードではなく、運用上の検証です。

### D.2 — 書き込み承認ラウンドトリップ（ヒューマンインザループ） ✓

```text
エージェントが書き込みが必要と判断
  → verify_write_safety → 拒否
    → orexis.request_write_approval → WriteApprovalRequestをブロードキャスト
      → shittim-chestが承認ダイアログを表示（industrial.approveWrite）
        → [承認] → 一時的ホワイトリストエントリ → 実行 + 読み戻し検証
        → [拒否]   → エージェントが拒否を受信、計画を調整
```

**実装済み:**

| # | タスク | ファイル | 状態 |
| --- | --- | --- | --- |
| A.2.4.1 | `orexis.request_write_approval` MCPツール — `WriteApprovalRequest`を構築、`TuiMessage::IndustrialWriteApprovalPush`をブロードキャスト、オペレーターが応答するまでサスペンド（oneshot + タイムアウト） | `packages/agents/orexis/src/mcp/tools/industrial_write_tools.rs` | ✓ |
| A.2.4.2 | `industrial.approveWrite` WSハンドラ — 共有`WriteApprovalRegistry`経由で保留中のリクエストを解決。承認時に一時的なホワイトリストエントリを追加し、後続の書き込みが`verify_write_safety`を通過できるようにする | `packages/scepter/src/tui_connection/mod.rs` | ✓ |

プロデューサー/リゾルバーはプロセス全体で共有される`WriteApprovalRegistry`（`_shared_security_policy::write_approval_registry`）を通じて分離されており、起動時にorexisに注入され、オペレーターが応答する際にscepterによって使用されます。

---

## フェーズE: エンドツーエンドドッグフード

運用上の検証であり、純粋なコードではありません。ハードウェアシミュレーターの実行が必要です。

### E.1 — テスト環境

| # | コンポーネント | セットアップ |
| --- | --- | --- |
| E.1.1 | S7commシミュレーター | 仮想S7-1500として`snap7-server`クレートを実行。オフセット0にREAL温度、オフセット4にREAL圧力、オフセット8にINT流量、オフセット10にBOOLバルブ、および50バイトのランダムデータをDB1にプリロード |
| E.1.2 | Modbusシミュレーター | 仮想シリアルポート（`socat pty pty`）上でaobaスレーブモードを実行。既知のレジスタ値でステーション5をプリロード |
| E.1.3 | Entelecheia + evernight | 標準docker-compose起動。evernight `sensor-poll`は`--manifest`フラグ付きで準備完了 |

### E.2 — ドッグフードシナリオ

| # | シナリオ | 手順 | 合格基準 |
| --- | --- | --- | --- |
| E.2.1 | **未知のS7comm回廊** | (1) システムにターゲット`192.168.1.10:102`を与える。(2) `industrial_discover`スキルチェーンが自律的に実行。(3) システムがS7commプロトコル、DB1を発見、フィールドセマンティクスを推論、マニフェストを生成。(4) オペレーターがTUIでマニフェストをレビュー。(5) 承認 → evernightがポーリング開始。(6) アラーム値を注入 → Hubris alarm_responseがトリガー → 是正措置を提案。 | 3つ以上の正しく推論されたフィールドを含むマニフェストが生成される。アラームが`alarm_response → task_decompose → plan_execute`チェーンをトリガー。 |
| E.2.2 | **未知のModbus回廊** | 仮想シリアルポート上のModbus RTUで同じフロー。異なるステーションレイアウト。 | 同じ基準。 |
| E.2.3 | **混合プロトコル発見** | 両方のシミュレーターを同時に実行。システムが両方を発見し、結合マニフェストを生成。 | 両方のステーションが正しいプロトコルでマニフェストに表示。 |
| E.2.4 | **書き込み承認フロー** | エージェントがバルブを閉じることを提案（発見されたBOOLフィールドへの書き込み）。`verify_write_safety`がブロック（ホワイトリスト未登録）。WriteApprovalRequestがオペレーターに送信。オペレーターが承認。読み戻し検証付きで書き込み実行。 | 完全なラウンドトリップ: 提案 → ブロック → 要求 → 承認 → 実行 → 検証。**（D.2は現在出荷済み — ドッグフードの準備完了。）** |

### E.3 — デモ録画

| # | タスク | 備考 |
| --- | --- | --- |
| E.3.1 | 発見→監視→アラーム→応答の全サイクルを画面キャプチャとして録画 | 未知のハードウェアへの自律的適応を実証 |
| E.3.2 | 発見レポート成果物を生成（自動生成マニフェストTOML + 推論フィールドテーブル） | 政府マイルストーンレビュー用の有形の成果物 |

---

## 兄弟プロジェクトへの依存関係（残存）

| 兄弟 | 必要とするもの | 時期 | 状態 |
| --- | --- | --- | --- |
| **arona** | `WriteApprovalRequest`のWSブロードキャストパス（A.2.4） | ~~A.2.4 / D.2をブロック~~ 完了 — `TuiMessage::IndustrialWriteApprovalPush`（arona型から再エクスポート）に乗る | ✓ |
| **shittim-chest** | オペレーター承認ダイアログ（`industrial.approveWrite`コンシューマ）+ 発見進捗レンダリング | E.2.4ドッグフードをブロック（scepter内のWSハンドラは準備完了。shittim-chestがダイアログをレンダリングしレスポンスをPOSTする必要がある） | 兄弟PLAN |

---

## 明示的に範囲外（2週間スプリント）

- OPC UAクライアント/サーバー（Rustエコシステムが未対応）
- EtherNet/IP / CIP（Rockwell）
- EtherCAT（Beckhoff）
- CANバス
- フロントエンドテストカバレッジ（shittim-chestはガイダンス計画のみ、テスト作成なし）
- TUIとのCLI機能同等性

---

# 技術ロードマップ — アーキテクチャ深化

> **日付**: 2026-06-26
> **文脈**: 700以上の古くなったドキュメント/ファイルをリポジトリからクリーンアップし、すべてのプロンプトを`res/prompts/`に統合した後、残りの設計ドキュメントを実際のソースコードに対して監査し、どの野心的な設計が実装する価値があるかを特定しました。

---

## 1. サブバッジアドレッシング + 並列スキル実行

**判定**: 実装する価値あり。インフラは約80%構築済みで、最後の20%のみが不足。

**現在の状態**:

- `BadgeRegistry`（`packages/scepter/src/state_machine/badge_registry.rs:92-120`）はすでに親子`link_sessions()`をサポート。
- `#001.005`サブバッジ構文解析は`find_by_container_id_or_sub()`に存在するが、個別の子コンテナに解決する代わりにサブ番号を除去。
- `SnowflakeContainer.parent_id`と`branch_level`フィールドは存在するがメタデータのみ — ルーティングに使用されたことはない。
- エッジノード優先度キューイング（`edge_node_registry.rs:73-126`）はきめ細かいリソースロックの準備ができている。
- スキルチェーンは厳密に**シリアル** — `pipeline.rs:68-226`は一度に1スキルずつループ。独立した`next_targets`を持つコーディネータースキルは、並列実行可能な場合でもシリアルに実行。

**不足しているもの**:

1. ✅ `find_by_container_id_or_sub()`が`#001.005`を → 親コンテナの最も深いアクティブなフォークされた子に解決。フォークが存在しない場合は親にフォールバック（後方互換）。
1. ✅ `SnowflakeManager`に子/子孫検索を追加: `children_of`、`children_of_badge`、`most_recent_child_of`、`deepest_descendant`（`parent_id` → 逆インデックス）。
1. ✅ `next_targets`の`FuturesUnordered`ベースの並列実行: `dispatch_parallel_targets`はコーディネーターの独立した**リーフ**ターゲットを`parallel_dispatch::fan_out`（`Semaphore`で制限）を通じて同時に展開。シリアル`invoke_skill_with_retries`パスの2つのグローバルシングルトンブロッカーは以下のように処理:

   - **共有ローカルcosmos名前空間** → 各ターゲットはフェーズ1で**独自のcosmosコンテナ**にフォーク（`fork_container_for_skill` + `assign_container_id` + `register_container_badge_in_registry`）。そのため`dump/restore_cosmos_namespace`はブランチごとにno-opであり、並行実行は分離される。`MAX_BRANCH_DEPTH`（項目4）がフォークチェーンを制限。
   - **`active_streaming_skill` UI競合** → 許容（`Option`に対する最終書き込み優先。各ブランチ後に`None`にリセット）。
   - **`&mut SkillChainInput`スレッド** → `BranchOwner`がブランチごとに可変部分をミラーリング。`as_input`がそれらを短命の`SkillChainInput`に借用し戻すため、変更されていないパイプラインヘルパーが再利用可能。

フェーズ1（フォーク + 準備 + プロンプト構築 + ツールホワイトリスト）は`rag_buffer`競合を避けるために**シリアル化**。フェーズ2（レイテンシが支配的なLLM呼び出し）のみが並列実行。フェーズ3はクリーンアップとレポートのマージ（`merge_branch_reports`）を親コンテキストに。`SKILL_CHAIN_PARALLEL_TARGETS`（デフォルト**off**）+ `parallel_targets_eligible`（コンテナ化 + 全リーフターゲット）の背後にゲート。`route_to_next_skill`のシリアルスタックアンワインドがデフォルトのまま。

1. ✅ 両方のフォークパスで`MAX_BRANCH_DEPTH`（`COSMOS_MAX_BRANCH_DEPTH`、デフォルト4）を強制。子はハードコードされた`1`の代わりに`source.branch_level + 1`で登録されるようになった。

**期待される影響**: `industrial_discover`のようなコーディネータースキルからの並列ファイル書き込み、並列分析により、エンドツーエンドのレイテンシが大幅に削減されます。

---

## 2. メモリ堆積パイプライン

**判定**: 品質乗数、重要ではない。長期ロードマップ向けに予約。

**現在の状態**:

- `PhiliaMemoryService`は代謝のないフラットな「保存 → 埋め込み → 取得」グラフ。
- `memory_consolidate`はささいなもの — エピソードノードを作成するのみで、抽象化/要約はない。
- メモリの減衰、経年変化、陳腐化スコア、ノード間の品質勾配はない。
- すべてのノードは未分化の`MemoryNode` — エピソード/手続き/原子的分離はない。
- インメモリベクトル検索はO(n)のブルートフォース（長期的にスケールしない）。
- `KnowledgeStore`（別システム）はライフサイクル段階（Created→Vectorized→Searchable→Consolidated→Deprecated）と合意検証を持つ — 堆積に最も近い既存の類似物。

**なぜ緊急ではないか**:

- RAGコンテキスト注入（`RagContextBuffer` → LLMクエリ書き換え → `bundle_search`）は現在のツール呼び出しエージェントに十分なコンテキストを提供。
- pgvector HNSWインデックスはプロダクションレベルの検索を処理。
- システムは「保存と取得」として動作 — 堆積は「代謝」させるが、これは漸進的な品質向上であり、機能的なギャップではない。

**将来の作業**（タイムラインなし）:

- 自動統合: LM駆動による関連ノードの上位レベルの「エピソード」への定期的要約。
- 品質勾配: アクセス数、時間的減衰、信頼度スコアリング。
- 差別化された検索戦略を持つ3チャネルプロトタイプ（エピソード/手続き/原子的）。

---

## 3. エージェント間交渉

**判定**: 低優先度。プリミティブは低レベルの構成要素として存在。即時のユースケースはない。

**現在の状態**:

- `deliver_message(message_type="Question")`が存在（`epieikeia/src/mcp/tools/deliver_message.rs:63`） — 別のエージェントのメールボックスに質問をプッシュ可能。
- `inject_user_prompt` / `consume_injected_prompts`は存在するが**ポーリングベース** — パイプライン統合なし。エージェントは明示的に`consume_injected_prompts`を呼び出してメールをチェックする必要がある。
- `Haplotes`は`AskAgent` / `ReplyAgent` / `Escalated`会話ルーティング型を持つ — しかしすべてがビジネスロジックゼロのno-op ACK。
- `NEGOTIATION_ROUND_TIMEOUT_SECS` / `NEGOTIATION_TOTAL_TIMEOUT_SECS`環境変数は`RuntimeTuningConfig`で定義されているが**どこでも消費されていない** — デッドコード。

**なぜ低優先度か**:

- 現在のシーケンシャルスキルチェーンディスパッチ + 文字列としてのコンテキスト渡しがすべての現在のユースケースを処理。
- マージ競合は単一スキルディスパッチ（`resolve_merge_conflict`）で処理され、これで十分。
- 交渉ループ（スキルチェーンをインターセプト → エージェントに質問 → 応答を待つ → 組み込む）は構築とテストが複雑。まだ要求するプロダクションユースケースはない。

**再検討する時期**: エージェントが動的にチェーン中の決定を交渉する必要がある場合（単なるディスパッチアンドウェイトではなく）、プリミティブは40%構築されている。ギャップはパイプライン統合ループ。

---

## 要約

| 機能 | インフラ構築率 | 優先度 | 次のステップ |
| --- | --- | --- | --- |
| サブバッジ + 並列実行 | 100% | **高** | ✅ 完了 — サブバッジ→子、子インデックス、ブランチ深度＆インループ並列ディスパッチすべて出荷済み（並列はデフォルト無効） |
| メモリ堆積 | 20% | **長期** | 即時アクションなし。並列実行後に再検討 |
| エージェント間交渉 | 40% | **低** | 具体的なユースケースを待つ。プリミティブは準備完了 |
