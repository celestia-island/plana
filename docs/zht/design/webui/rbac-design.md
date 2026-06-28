+++
title = "RBAC 系統詳細設計文件"
description = """為 Shittim Chest 實作完整的基於角色的存取控制系統，支援："""
lang = "zht"
category = "design"
subcategory = "webui"
+++

# RBAC 系統詳細設計文件

## 1. 目標

為 Shittim Chest 實作完整的基於角色的存取控制系統，支援：

- **使用者管理**：管理員可邀請/建立/停用/刪除使用者
- **群組管理**：支援帳號群組，使用者可屬於多個群組
- **細粒度權限**：控制使用者能否新增/修改/使用特定的模型提供者、MCP 工具、Layer3 Agent、IM 頻道等
- **功能開關**：控制使用者能否使用自動巡航模式等進階功能
- **靈活授權模式**：管理員可選擇全域統一設定、各個帳號單獨設定或帳號群組共享

## 2. 核心概念

### 2.1 角色 (Role)

| 角色 | 說明 |
| --- | --- |
| `admin` | 超級管理員，擁有所有權限，可管理 RBAC 本身 |
| `operator` | 運營人員，可管理大部分資源（提供者、頻道、Agent 等） |
| `member` | 普通成員，可使用被授權的資源 |
| `viewer` | 唯讀使用者，僅可檢視，不可修改 |

角色是**預設的**，不提供自訂角色（簡化實作）。每個使用者可以有一個主角色。

### 2.2 權限 (Permission)

權限格式：`<resource>.<action>`

| 類別 | 權限 | 說明 |
| --- | --- | --- |
| **提供者** | `provider.list` | 檢視提供者列表 |
| | `provider.create` | 新增提供者 |
| | `provider.update` | 修改提供者設定 |
| | `provider.delete` | 刪除提供者 |
| | `provider.use` | 使用提供者的模型進行對話 |
| **MCP 工具** | `mcp.list` | 檢視 MCP 工具列表 |
| | `mcp.create` | 註冊 MCP 工具 |
| | `mcp.update` | 修改 MCP 工具設定 |
| | `mcp.delete` | 刪除 MCP 工具 |
| | `mcp.use` | 在對話中使用 MCP 工具 |
| **Agent** | `agent.list` | 檢視 Agent 列表 |
| | `agent.create` | 建立 Agent |
| | `agent.update` | 修改 Agent 設定 |
| | `agent.delete` | 刪除 Agent |
| | `agent.use` | 在分析模式中使用 Agent |
| **IM 頻道** | `channel.list` | 檢視 IM 頻道列表 |
| | `channel.create` | 建立 IM 頻道 |
| | `channel.update` | 修改頻道設定 |
| | `channel.delete` | 刪除頻道 |
| | `channel.use` | 透過頻道收發訊息 |
| **巡航模式** | `yolo.use` | 使用自動巡航模式 |
| **工作區** | `workspace.list` | 檢視工作區 |
| | `workspace.create` | 建立工作區 |
| | `workspace.manage` | 管理工作區（刪除、匯出） |
| **設備** | `device.list` | 檢視遠端設備 |
| | `device.connect` | 連線遠端設備 |
| **系統** | `system.read` | 檢視系統設定 |
| | `system.write` | 修改系統設定 |
| | `rbac.manage` | 管理 RBAC（使用者/群組/權限） |
| **OAuth** | `oauth.read` | 檢視 OAuth 設定 |
| | `oauth.write` | 修改 OAuth 設定 |

### 2.3 角色預設權限

| 權限 | admin | operator | member | viewer |
| --- | --- | --- | --- | --- |
| `provider.*` | ✅ | ✅ | `list` + `use` | `list` |
| `mcp.*` | ✅ | ✅ | `list` + `use` | `list` |
| `agent.*` | ✅ | ✅ | `list` + `use` | `list` |
| `channel.*` | ✅ | ✅ | `list` + `use` | `list` |
| `yolo.use` | ✅ | ✅ | ❌ (預設關閉) | ❌ |
| `workspace.*` | ✅ | ✅ | `list` + `create` | `list` |
| `device.*` | ✅ | ✅ | `list` + `connect` | `list` |
| `system.*` | ✅ | ❌ | ❌ | ❌ |
| `rbac.manage` | ✅ | ❌ | ❌ | ❌ |
| `oauth.*` | ✅ | ✅ | ❌ | ❌ |

### 2.4 授權模式

對於提供者、MCP、Agent、頻道等資源，支援三種授權模式：

| 模式 | 說明 | 適用場景 |
| --- | --- | --- |
| **全域設定** | 所有使用者共享相同的權限 | 小團隊、個人使用 |
| **按使用者設定** | 每個使用者有獨立的資源權限 | 需要精細控制的場景 |
| **按群組設定** | 同一群組的使用者共享權限 | 按部門/團隊劃分 |

管理員在「權限矩陣」頁面中選擇授權模式，然後設定具體的允許/拒絕規則。

**優先級**：按使用者設定 > 按群組設定 > 全域設定 > 角色預設權限

## 3. 資料庫 Schema

### 3.1 新增表

#### `rbac_groups` — 使用者群組

```sql
CREATE TABLE rbac_groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `rbac_user_groups` — 使用者-群組關聯

```sql
CREATE TABLE rbac_user_groups (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id   UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);
```

#### `rbac_grants` — 權限授予（統一表）

```sql
CREATE TABLE rbac_grants (
    id           UUID PRIMARY KEY,
    -- 授權目標（三選一）
    scope        VARCHAR(16) NOT NULL, -- 'global' | 'group' | 'user'
    user_id      UUID REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id     UUID REFERENCES rbac_groups(id) ON DELETE CASCADE,
    -- 權限
    permission   VARCHAR(64) NOT NULL, -- e.g. 'provider.use', 'yolo.use'
    resource_id  VARCHAR(128),         -- 可選：限定到特定資源 (provider name, channel id 等)，NULL 表示該類別全部資源
    -- 授權類型
    granted      BOOLEAN NOT NULL DEFAULT TRUE, -- TRUE=允許, FALSE=拒絕
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 約束：scope 和對應的 FK 必須一致
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

### 3.2 修改現有表

#### `auth_users` 增加角色欄位

```sql
ALTER TABLE auth_users ADD COLUMN role VARCHAR(16) NOT NULL DEFAULT 'member';
-- 遷移：is_admin=true 的使用者設為 'admin'
UPDATE auth_users SET role = 'admin' WHERE is_admin = TRUE;
```

保留 `is_admin` 欄位做相容，但新程式碼優先使用 `role`。

### 3.3 權限檢查邏輯（偽程式碼）

```rust
fn has_permission(user, permission, resource_id=None) -> bool {
    // 1. admin 角色直接通過
    if user.role == "admin" { return true; }

    // 2. 收集所有匹配的 grants，按優先級排序
    let grants = [];

    // 2a. 角色預設權限（最低優先級）
    grants.push(role_defaults(user.role, permission));

    // 2b. 全域設定
    grants.extend(query_grants(scope="global", permission, resource_id));

    // 2c. 群組設定（使用者所屬的所有群組）
    for group in user.groups:
        grants.extend(query_grants(scope="group", group_id=group.id, permission, resource_id));

    // 2d. 使用者級別設定（最高優先級）
    grants.extend(query_grants(scope="user", user_id=user.id, permission, resource_id));

    // 3. 按優先級：user > group > global > role_default
    // 同一 scope 內，denied 優先於 granted
    // 有任何 user scope denied → 拒絕
    // 有任何 group scope denied → 拒絕（除非 user scope granted）
    // 最終結果
    resolve_grants(grants)
}
```

## 4. API 設計

### 4.1 使用者管理 (`/api/rbac/users`)

| 方法 | 路徑 | 權限 | 說明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/users` | `rbac.manage` | 列出所有使用者（含角色、群組） |
| POST | `/api/rbac/users` | `rbac.manage` | 邀請使用者（發郵件或建立帳號） |
| PUT | `/api/rbac/users/:id` | `rbac.manage` | 更新使用者角色、啟用/停用 |
| DELETE | `/api/rbac/users/:id` | `rbac.manage` | 刪除使用者 |

### 4.2 群組管理 (`/api/rbac/groups`)

| 方法 | 路徑 | 權限 | 說明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/groups` | `rbac.manage` | 列出所有群組 |
| POST | `/api/rbac/groups` | `rbac.manage` | 建立群組 |
| PUT | `/api/rbac/groups/:id` | `rbac.manage` | 更新群組（名稱、描述） |
| DELETE | `/api/rbac/groups/:id` | `rbac.manage` | 刪除群組 |
| POST | `/api/rbac/groups/:id/members` | `rbac.manage` | 新增成員 |
| DELETE | `/api/rbac/groups/:id/members/:userId` | `rbac.manage` | 移除成員 |

### 4.3 權限管理 (`/api/rbac/grants`)

| 方法 | 路徑 | 權限 | 說明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/grants` | `rbac.manage` | 列出所有權限規則（支援 ?scope=&permission= 過濾） |
| PUT | `/api/rbac/grants` | `rbac.manage` | 批量設定權限（傳入完整規則列表，覆蓋對應 scope 的規則） |
| DELETE | `/api/rbac/grants/:id` | `rbac.manage` | 刪除單條規則 |

### 4.4 權限檢查 (`/api/rbac/check`)

| 方法 | 路徑 | 權限 | 說明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/check?permission=xxx&resource_id=yyy` | (任何認證使用者) | 檢查目前使用者是否有指定權限 |
| GET | `/api/rbac/my-permissions` | (任何認證使用者) | 返回目前使用者的所有有效權限列表 |

### 4.5 資源可見性改造

現有資源 API 需要增加權限過濾：

- `GET /api/chat/providers` → 只返回目前使用者有 `provider.list` 權限的提供者，且只展示有 `provider.use` 權限的模型
- `GET /api/channel` → 只返回有 `channel.list` 權限的頻道
- 巡航模式啟動前 → 檢查 `yolo.use` 權限

## 5. 前端設計（Plana）

### 5.1 RbacView 重構

分為三個 Tab：

#### Tab 1: 使用者管理

- 使用者列表表格：頭像、使用者名稱、信箱、角色（下拉切換）、群組標籤、狀態（活躍/停用）、操作
- 邀請使用者按鈕 → 彈出 Modal（輸入使用者名稱/信箱/密碼、選擇角色）
- 行操作：編輯角色、停用/啟用、刪除

#### Tab 2: 群組管理

- 群組列表表格：名稱、描述、成員數、操作
- 建立群組 → 彈出 Modal
- 點擊群組 → 展開成員列表，可新增/移除成員

#### Tab 3: 權限矩陣

- 左上角選擇授權模式：全域 / 按群組 / 按使用者
- 選擇群組或使用者後，顯示權限矩陣表格：
  - 行：資源類別（提供者、MCP、Agent、頻道、巡航模式...）
  - 列：操作（列表、建立、修改、刪除、使用）
  - 儲存格：三態切換（✅ 允許 / ❌ 拒絕 / ➖ 繼承預設）
- 特定資源 ID 的精細化控制（如只允許使用某個提供者）

### 5.2 導覽權限控制

- 側邊欄項目根據目前使用者權限動態顯示/隱藏
- 路由守衛增加權限檢查，無權限時跳轉到 403 頁面
- 操作按鈕（如「新增提供者」）根據權限顯示/隱藏

## 6. 實作步驟

### Phase 1: 後端基礎

1. 新增資料庫遷移（`rbac_groups`, `rbac_user_groups`, `rbac_grants` 表 + auth_users.role 欄位）
1. 新增 SeaORM 實體模型
1. 實作 RBAC API 路由（users, groups, grants CRUD）
1. 實作權限檢查中介層/extractor
1. 在 JWT claims 中加入 role 欄位

### Phase 2: 後端整合

1. 在現有資源 API（providers, channels 等）中加入權限檢查
1. 實作 `/api/rbac/check` 和 `/api/rbac/my-permissions`
1. 修改 arona 的資源請求以適配權限過濾

### Phase 3: 前端 UI

1. 重構 arona 的 RbacView（使用者/群組/權限矩陣三個 Tab）
1. 實作側邊欄和路由的權限守衛
1. arona 端根據權限隱藏/停用功能（如巡航模式按鈕）

## 7. 安全考量

- `admin` 角色的權限不可被 `rbac_grants` 覆蓋（硬編碼放行）
- 權限檢查在中介層層級統一執行，不依賴業務程式碼手動檢查
- 敏感操作（刪除使用者、修改權限）記錄審計日誌
- JWT 中只包含 role，具體權限每次從 DB 即時查詢（避免權限變更後 token 未更新）
