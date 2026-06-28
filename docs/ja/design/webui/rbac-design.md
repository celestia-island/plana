+++
title = "RBAC システム詳細設計ドキュメント"
description = """Shittim Chestに完全なロールベースアクセス制御システムを実装し、以下をサポートします："""
lang = "ja"
category = "design"
subcategory = "webui"
+++

# RBAC システム詳細設計ドキュメント

## 1. 目標

Shittim Chestに完全なロールベースアクセス制御システムを実装し、以下をサポートします：

- **ユーザー管理**：管理者はユーザーの招待/作成/無効化/削除が可能
- **グループ管理**：アカウントグループをサポート、ユーザーは複数のグループに所属可能
- **細粒度権限**：特定のモデルプロバイダー、MCPツール、Layer3エージェント、IMチャネルなどの追加/変更/使用を制御
- **機能スイッチ**：自動クルーズモードなどの高度な機能の使用を制御
- **柔軟な認可モード**：管理者はグローバル統一設定、アカウント個別設定、アカウントグループ共有から選択可能

## 2. 核心概念

### 2.1 ロール (Role)

| ロール | 説明 |
| --- | --- |
| `admin` | スーパー管理者、全権限を持ち、RBAC自体を管理可能 |
| `operator` | 運用担当者、ほとんどのリソース（プロバイダー、チャネル、エージェントなど）を管理可能 |
| `member` | 一般メンバー、認可されたリソースを使用可能 |
| `viewer` | 読み取り専用ユーザー、閲覧のみ可能、変更不可 |

ロールは**プリセット**であり、カスタムロールは提供しません（実装簡略化）。各ユーザーは1つのメインロールを持ちます。

### 2.2 権限 (Permission)

権限形式：`<resource>.<action>`

| カテゴリ | 権限 | 説明 |
| --- | --- | --- |
| **プロバイダー** | `provider.list` | プロバイダーリストの表示 |
| | `provider.create` | プロバイダーの追加 |
| | `provider.update` | プロバイダー設定の変更 |
| | `provider.delete` | プロバイダーの削除 |
| | `provider.use` | プロバイダーのモデルを使用した会話 |
| **MCPツール** | `mcp.list` | MCPツールリストの表示 |
| | `mcp.create` | MCPツールの登録 |
| | `mcp.update` | MCPツール設定の変更 |
| | `mcp.delete` | MCPツールの削除 |
| | `mcp.use` | 会話でのMCPツールの使用 |
| **エージェント** | `agent.list` | エージェントリストの表示 |
| | `agent.create` | エージェントの作成 |
| | `agent.update` | エージェント設定の変更 |
| | `agent.delete` | エージェントの削除 |
| | `agent.use` | 分析モードでのエージェントの使用 |
| **IMチャネル** | `channel.list` | IMチャネルリストの表示 |
| | `channel.create` | IMチャネルの作成 |
| | `channel.update` | チャネル設定の変更 |
| | `channel.delete` | チャネルの削除 |
| | `channel.use` | チャネル経由のメッセージ送受信 |
| **クルーズモード** | `yolo.use` | 自動クルーズモードの使用 |
| **ワークスペース** | `workspace.list` | ワークスペースの表示 |
| | `workspace.create` | ワークスペースの作成 |
| | `workspace.manage` | ワークスペースの管理（削除、エクスポート） |
| **デバイス** | `device.list` | リモートデバイスの表示 |
| | `device.connect` | リモートデバイスへの接続 |
| **システム** | `system.read` | システム設定の表示 |
| | `system.write` | システム設定の変更 |
| | `rbac.manage` | RBACの管理（ユーザー/グループ/権限） |
| **OAuth** | `oauth.read` | OAuth設定の表示 |
| | `oauth.write` | OAuth設定の変更 |

### 2.3 ロールのデフォルト権限

| 権限 | admin | operator | member | viewer |
| --- | --- | --- | --- | --- |
| `provider.*` | ✅ | ✅ | `list` + `use` | `list` |
| `mcp.*` | ✅ | ✅ | `list` + `use` | `list` |
| `agent.*` | ✅ | ✅ | `list` + `use` | `list` |
| `channel.*` | ✅ | ✅ | `list` + `use` | `list` |
| `yolo.use` | ✅ | ✅ | ❌ (デフォルト無効) | ❌ |
| `workspace.*` | ✅ | ✅ | `list` + `create` | `list` |
| `device.*` | ✅ | ✅ | `list` + `connect` | `list` |
| `system.*` | ✅ | ❌ | ❌ | ❌ |
| `rbac.manage` | ✅ | ❌ | ❌ | ❌ |
| `oauth.*` | ✅ | ✅ | ❌ | ❌ |

### 2.4 認可モード

プロバイダー、MCP、エージェント、チャネルなどのリソースについて、3つの認可モードをサポートします：

| モード | 説明 | 適用シーン |
| --- | --- | --- |
| **グローバル設定** | 全ユーザーが同じ権限を共有 | 小規模チーム、個人利用 |
| **ユーザー別設定** | 各ユーザーが独立したリソース権限を持つ | 細かな制御が必要なシーン |
| **グループ別設定** | 同一グループのユーザーが権限を共有 | 部門/チーム別の区分 |

管理者は「権限マトリックス」ページで認可モードを選択し、具体的な許可/拒否ルールを設定します。

**優先順位**：ユーザー別設定 > グループ別設定 > グローバル設定 > ロールのデフォルト権限

## 3. データベーススキーマ

### 3.1 新規テーブル

#### `rbac_groups` — ユーザーグループ

```sql
CREATE TABLE rbac_groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `rbac_user_groups` — ユーザー-グループ関連

```sql
CREATE TABLE rbac_user_groups (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id   UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);
```

#### `rbac_grants` — 権限付与（統合テーブル）

```sql
CREATE TABLE rbac_grants (
    id           UUID PRIMARY KEY,
    -- 認可対象（三択一）
    scope        VARCHAR(16) NOT NULL, -- 'global' | 'group' | 'user'
    user_id      UUID REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id     UUID REFERENCES rbac_groups(id) ON DELETE CASCADE,
    -- 権限
    permission   VARCHAR(64) NOT NULL, -- e.g. 'provider.use', 'yolo.use'
    resource_id  VARCHAR(128),         -- オプション：特定のリソースに限定 (provider name, channel id など)、NULLはそのカテゴリの全リソースを表す
    -- 付与タイプ
    granted      BOOLEAN NOT NULL DEFAULT TRUE, -- TRUE=許可, FALSE=拒否
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 制約：scopeと対応するFKが一致すること
    CONSTRAINT rbac_grants_scope_check CHECK (
        (scope = 'global' AND user_id IS NULL AND group_id IS NULL) OR
        (scope = 'user'   AND user_id IS NOT NULL AND group_id IS NULL) OR
        (scope = 'group'  AND user_id IS NULL AND group_id IS NOT NULL)
    )
);
CREATE INDEX idx_rbac_grants_user ON rbac_grants(user_id);
CREATE INDEX idx_rbac_grants_group ON rbac_grants(group_id);
CREATE INDEX idx_rbac_grants_permission ON rbac_grants(permission);
```

### 3.2 既存テーブルの変更

#### `auth_users`にロールフィールドを追加

```sql
ALTER TABLE auth_users ADD COLUMN role VARCHAR(16) NOT NULL DEFAULT 'member';
-- 移行：is_admin=trueのユーザーを'admin'に設定
UPDATE auth_users SET role = 'admin' WHERE is_admin = TRUE;
```

`is_admin`フィールドは互換性のために保持しますが、新しいコードでは`role`を優先して使用します。

### 3.3 権限チェックロジック（疑似コード）

```rust
fn has_permission(user, permission, resource_id=None) -> bool {
    // 1. adminロールは直接通過
    if user.role == "admin" { return true; }

    // 2. マッチするすべてのgrantsを収集し、優先順位でソート
    let grants = [];

    // 2a. ロールのデフォルト権限（最低優先度）
    grants.push(role_defaults(user.role, permission));

    // 2b. グローバル設定
    grants.extend(query_grants(scope="global", permission, resource_id));

    // 2c. グループ設定（ユーザーが所属するすべてのグループ）
    for group in user.groups:
        grants.extend(query_grants(scope="group", group_id=group.id, permission, resource_id));

    // 2d. ユーザーレベル設定（最高優先度）
    grants.extend(query_grants(scope="user", user_id=user.id, permission, resource_id));

    // 3. 優先順位に従う：user > group > global > role_default
    // 同一scope内では、deniedがgrantedより優先
    // user scopeにdeniedがあれば → 拒否
    // group scopeにdeniedがあれば → 拒否（user scopeがgrantedの場合を除く）
    // 最終結果
    resolve_grants(grants)
}
```

## 4. API設計

### 4.1 ユーザー管理 (`/api/rbac/users`)

| メソッド | パス | 権限 | 説明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/users` | `rbac.manage` | 全ユーザーをリスト（ロール、グループ含む） |
| POST | `/api/rbac/users` | `rbac.manage` | ユーザーを招待（メール送信またはアカウント作成） |
| PUT | `/api/rbac/users/:id` | `rbac.manage` | ユーザーロールの更新、有効/無効の切り替え |
| DELETE | `/api/rbac/users/:id` | `rbac.manage` | ユーザーの削除 |

### 4.2 グループ管理 (`/api/rbac/groups`)

| メソッド | パス | 権限 | 説明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/groups` | `rbac.manage` | 全グループをリスト |
| POST | `/api/rbac/groups` | `rbac.manage` | グループを作成 |
| PUT | `/api/rbac/groups/:id` | `rbac.manage` | グループを更新（名前、説明） |
| DELETE | `/api/rbac/groups/:id` | `rbac.manage` | グループを削除 |
| POST | `/api/rbac/groups/:id/members` | `rbac.manage` | メンバーを追加 |
| DELETE | `/api/rbac/groups/:id/members/:userId` | `rbac.manage` | メンバーを削除 |

### 4.3 権限管理 (`/api/rbac/grants`)

| メソッド | パス | 権限 | 説明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/grants` | `rbac.manage` | 全権限ルールをリスト（?scope=&permission= フィルタ対応） |
| PUT | `/api/rbac/grants` | `rbac.manage` | 権限を一括設定（完全なルールリストを送信、対応するscopeのルールを上書き） |
| DELETE | `/api/rbac/grants/:id` | `rbac.manage` | 単一ルールを削除 |

### 4.4 権限チェック (`/api/rbac/check`)

| メソッド | パス | 権限 | 説明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/check?permission=xxx&resource_id=yyy` | (認証済みユーザー) | 現在のユーザーが指定された権限を持っているか確認 |
| GET | `/api/rbac/my-permissions` | (認証済みユーザー) | 現在のユーザーの全有効権限リストを返す |

### 4.5 リソース可視性の改修

既存のリソースAPIに権限フィルタリングを追加：

- `GET /api/chat/providers` → 現在のユーザーが`provider.list`権限を持つプロバイダーのみ返し、かつ`provider.use`権限のあるモデルのみ表示
- `GET /api/channel` → `channel.list`権限のあるチャネルのみ返す
- クルーズモード起動前 → `yolo.use`権限をチェック

## 5. フロントエンド設計（Plana）

### 5.1 RbacViewの再構築

3つのタブに分割：

#### タブ1: ユーザー管理

- ユーザーリストテーブル：アバター、ユーザー名、メール、ロール（ドロップダウン切替）、グループタグ、状態（アクティブ/無効）、操作
- ユーザー招待ボタン → モーダルを表示（ユーザー名/メール/パスワード入力、ロール選択）
- 行操作：ロール編集、無効/有効化、削除

#### タブ2: グループ管理

- グループリストテーブル：名前、説明、メンバー数、操作
- グループ作成 → モーダルを表示
- グループをクリック → メンバーリストを展開、メンバーの追加/削除が可能

#### タブ3: 権限マトリックス

- 左上で認可モードを選択：グローバル / グループ別 / ユーザー別
- グループまたはユーザーを選択後、権限マトリックステーブルを表示：
  - 行：リソースカテゴリ（プロバイダー、MCP、エージェント、チャネル、クルーズモード...）
  - 列：操作（リスト、作成、変更、削除、使用）
  - セル：3状態切替（✅ 許可 / ❌ 拒否 / ➖ デフォルト継承）
- 特定リソースIDの細かい制御（特定のプロバイダーのみ使用許可など）

### 5.2 ナビゲーション権限制御

- サイドバー項目は現在のユーザー権限に応じて動的に表示/非表示
- ルートガードに権限チェックを追加、権限がない場合は403ページにリダイレクト
- 操作ボタン（「プロバイダー追加」など）を権限に応じて表示/非表示

## 6. 実装手順

### フェーズ1: バックエンド基盤

1. データベースマイグレーションを追加（`rbac_groups`, `rbac_user_groups`, rbac_grantsテーブル + auth_users.roleフィールド）
1. SeaORMエンティティモデルを追加
1. RBAC APIルーティングを実装（users, groups, grants CRUD）
1. 権限チェックミドルウェア/extractorを実装
1. JWT claimsにroleフィールドを追加

### フェーズ2: バックエンド統合

1. 既存のリソースAPI（providers, channelsなど）に権限チェックを追加
1. `/api/rbac/check`と`/api/rbac/my-permissions`を実装
1. aronaのリソースリクエストを権限フィルタリングに適合するよう変更

### フェーズ3: フロントエンドUI

1. aronaのRbacViewを再構築（ユーザー/グループ/権限マトリックスの3タブ）
1. サイドバーとルートの権限ガードを実装
1. arona側で権限に応じた機能の表示/非表示（クルーズモードボタンなど）

## 7. セキュリティ考慮事項

- `admin`ロールの権限はrbac_grantsで上書き不可（ハードコードで通過）
- 権限チェックはミドルウェア層で統一的に実行し、ビジネスコードの手動チェックに依存しない
- 機密操作（ユーザー削除、権限変更）は監査ログを記録
- JWTにはroleのみを含め、具体的な権限は毎回DBからリアルタイムクエリ（権限変更後にトークンが未更新になるのを回避）
