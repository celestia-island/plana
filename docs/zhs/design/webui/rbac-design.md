+++
title = "RBAC 系统详细设计文档"
description = """为 Shittim Chest 实现完整的基于角色的访问控制系统，支持："""
lang = "zhs"
category = "design"
subcategory = "webui"
+++

# RBAC 系统详细设计文档

## 1. 目标

为 Shittim Chest 实现完整的基于角色的访问控制系统，支持：

- **用户管理**：管理员可邀请/创建/禁用/删除用户
- **群组管理**：支持账号组，用户可属于多个群组
- **细粒度权限**：控制用户能否添加/修改/使用特定的模型提供商、MCP 工具、Layer3 Agent、IM 通道等
- **功能开关**：控制用户能否使用自动巡航模式等高级功能
- **灵活授权模式**：管理员可选择全局统一配置、各个账号单独配置或账号组共享

## 2. 核心概念

### 2.1 角色 (Role)

| 角色 | 说明 |
| --- | --- |
| `admin` | 超级管理员，拥有所有权限，可管理 RBAC 本身 |
| `operator` | 运营人员，可管理大部分资源（提供商、通道、Agent 等） |
| `member` | 普通成员，可使用被授权的资源 |
| `viewer` | 只读用户，仅可查看，不可修改 |

角色是**预设的**，不提供自定义角色（简化实现）。每个用户可以有一个主角色。

### 2.2 权限 (Permission)

权限格式：`<resource>.<action>`

| 类别 | 权限 | 说明 |
| --- | --- | --- |
| **提供商** | `provider.list` | 查看提供商列表 |
| | `provider.create` | 添加提供商 |
| | `provider.update` | 修改提供商配置 |
| | `provider.delete` | 删除提供商 |
| | `provider.use` | 使用提供商的模型进行对话 |
| **MCP 工具** | `mcp.list` | 查看 MCP 工具列表 |
| | `mcp.create` | 注册 MCP 工具 |
| | `mcp.update` | 修改 MCP 工具配置 |
| | `mcp.delete` | 删除 MCP 工具 |
| | `mcp.use` | 在对话中使用 MCP 工具 |
| **Agent** | `agent.list` | 查看 Agent 列表 |
| | `agent.create` | 创建 Agent |
| | `agent.update` | 修改 Agent 配置 |
| | `agent.delete` | 删除 Agent |
| | `agent.use` | 在分析模式中使用 Agent |
| **IM 通道** | `channel.list` | 查看 IM 通道列表 |
| | `channel.create` | 创建 IM 通道 |
| | `channel.update` | 修改通道配置 |
| | `channel.delete` | 删除通道 |
| | `channel.use` | 通过通道收发消息 |
| **巡航模式** | `yolo.use` | 使用自动巡航模式 |
| **工作区** | `workspace.list` | 查看工作区 |
| | `workspace.create` | 创建工作区 |
| | `workspace.manage` | 管理工作区（删除、导出） |
| **设备** | `device.list` | 查看远程设备 |
| | `device.connect` | 连接远程设备 |
| **系统** | `system.read` | 查看系统设置 |
| | `system.write` | 修改系统设置 |
| | `rbac.manage` | 管理 RBAC（用户/群组/权限） |
| **OAuth** | `oauth.read` | 查看 OAuth 配置 |
| | `oauth.write` | 修改 OAuth 配置 |

### 2.3 角色默认权限

| 权限 | admin | operator | member | viewer |
| --- | --- | --- | --- | --- |
| `provider.*` | ✅ | ✅ | `list` + `use` | `list` |
| `mcp.*` | ✅ | ✅ | `list` + `use` | `list` |
| `agent.*` | ✅ | ✅ | `list` + `use` | `list` |
| `channel.*` | ✅ | ✅ | `list` + `use` | `list` |
| `yolo.use` | ✅ | ✅ | ❌ (默认关闭) | ❌ |
| `workspace.*` | ✅ | ✅ | `list` + `create` | `list` |
| `device.*` | ✅ | ✅ | `list` + `connect` | `list` |
| `system.*` | ✅ | ❌ | ❌ | ❌ |
| `rbac.manage` | ✅ | ❌ | ❌ | ❌ |
| `oauth.*` | ✅ | ✅ | ❌ | ❌ |

### 2.4 授权模式

对于提供商、MCP、Agent、通道等资源，支持三种授权模式：

| 模式 | 说明 | 适用场景 |
| --- | --- | --- |
| **全局配置** | 所有用户共享相同的权限 | 小团队、个人使用 |
| **按用户配置** | 每个用户有独立的资源权限 | 需要精细控制的场景 |
| **按群组配置** | 同一群组的用户共享权限 | 按部门/团队划分 |

管理员在「权限矩阵」页面中选择授权模式，然后配置具体的允许/拒绝规则。

**优先级**：按用户配置 > 按群组配置 > 全局配置 > 角色默认权限

## 3. 数据库 Schema

### 3.1 新增表

#### `rbac_groups` — 用户群组

```sql
CREATE TABLE rbac_groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `rbac_user_groups` — 用户-群组关联

```sql
CREATE TABLE rbac_user_groups (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id   UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);
```

#### `rbac_grants` — 权限授予（统一表）

```sql
CREATE TABLE rbac_grants (
    id           UUID PRIMARY KEY,
    -- 授权目标（三选一）
    scope        VARCHAR(16) NOT NULL, -- 'global' | 'group' | 'user'
    user_id      UUID REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id     UUID REFERENCES rbac_groups(id) ON DELETE CASCADE,
    -- 权限
    permission   VARCHAR(64) NOT NULL, -- e.g. 'provider.use', 'yolo.use'
    resource_id  VARCHAR(128),         -- 可选：限定到特定资源 (provider name, channel id 等)，NULL 表示该类别全部资源
    -- 授权类型
    granted      BOOLEAN NOT NULL DEFAULT TRUE, -- TRUE=允许, FALSE=拒绝
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 约束：scope 和对应的 FK 必须一致
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

### 3.2 修改现有表

#### `auth_users` 增加角色字段

```sql
ALTER TABLE auth_users ADD COLUMN role VARCHAR(16) NOT NULL DEFAULT 'member';
-- 迁移：is_admin=true 的用户设为 'admin'
UPDATE auth_users SET role = 'admin' WHERE is_admin = TRUE;
```

保留 `is_admin` 字段做兼容，但新代码优先使用 `role`。

### 3.3 权限检查逻辑（伪代码）

```rust
fn has_permission(user, permission, resource_id=None) -> bool {
    // 1. admin 角色直接通过
    if user.role == "admin" { return true; }

    // 2. 收集所有匹配的 grants，按优先级排序
    let grants = [];

    // 2a. 角色默认权限（最低优先级）
    grants.push(role_defaults(user.role, permission));

    // 2b. 全局配置
    grants.extend(query_grants(scope="global", permission, resource_id));

    // 2c. 群组配置（用户所属的所有群组）
    for group in user.groups:
        grants.extend(query_grants(scope="group", group_id=group.id, permission, resource_id));

    // 2d. 用户级别配置（最高优先级）
    grants.extend(query_grants(scope="user", user_id=user.id, permission, resource_id));

    // 3. 按优先级：user > group > global > role_default
    // 同一 scope 内，denied 优先于 granted
    // 有任何 user scope denied → 拒绝
    // 有任何 group scope denied → 拒绝（除非 user scope granted）
    // 最终结果
    resolve_grants(grants)
}
```

## 4. API 设计

### 4.1 用户管理 (`/api/rbac/users`)

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/users` | `rbac.manage` | 列出所有用户（含角色、群组） |
| POST | `/api/rbac/users` | `rbac.manage` | 邀请用户（发邮件或创建账号） |
| PUT | `/api/rbac/users/:id` | `rbac.manage` | 更新用户角色、启用/禁用 |
| DELETE | `/api/rbac/users/:id` | `rbac.manage` | 删除用户 |

### 4.2 群组管理 (`/api/rbac/groups`)

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/groups` | `rbac.manage` | 列出所有群组 |
| POST | `/api/rbac/groups` | `rbac.manage` | 创建群组 |
| PUT | `/api/rbac/groups/:id` | `rbac.manage` | 更新群组（名称、描述） |
| DELETE | `/api/rbac/groups/:id` | `rbac.manage` | 删除群组 |
| POST | `/api/rbac/groups/:id/members` | `rbac.manage` | 添加成员 |
| DELETE | `/api/rbac/groups/:id/members/:userId` | `rbac.manage` | 移除成员 |

### 4.3 权限管理 (`/api/rbac/grants`)

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/grants` | `rbac.manage` | 列出所有权限规则（支持 ?scope=&permission= 过滤） |
| PUT | `/api/rbac/grants` | `rbac.manage` | 批量设置权限（传入完整规则列表，覆盖对应 scope 的规则） |
| DELETE | `/api/rbac/grants/:id` | `rbac.manage` | 删除单条规则 |

### 4.4 权限检查 (`/api/rbac/check`)

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/api/rbac/check?permission=xxx&resource_id=yyy` | (任何认证用户) | 检查当前用户是否有指定权限 |
| GET | `/api/rbac/my-permissions` | (任何认证用户) | 返回当前用户的所有有效权限列表 |

### 4.5 资源可见性改造

现有资源 API 需要增加权限过滤：

- `GET /api/chat/providers` → 只返回当前用户有 `provider.list` 权限的提供商，且只展示有 `provider.use` 权限的模型
- `GET /api/channel` → 只返回有 `channel.list` 权限的通道
- 巡航模式启动前 → 检查 `yolo.use` 权限

## 5. 前端设计（Plana）

### 5.1 RbacView 重构

分为三个 Tab：

#### Tab 1：用户管理

- 用户列表表格：头像、用户名、邮箱、角色（下拉切换）、群组标签、状态（活跃/禁用）、操作
- 邀请用户按钮 → 弹出 Modal（输入用户名/邮箱/密码、选择角色）
- 行操作：编辑角色、禁用/启用、删除

#### Tab 2：群组管理

- 群组列表表格：名称、描述、成员数、操作
- 创建群组 → 弹出 Modal
- 点击群组 → 展开成员列表，可添加/移除成员

#### Tab 3：权限矩阵

- 左上角选择授权模式：全局 / 按群组 / 按用户
- 选择群组或用户后，显示权限矩阵表格：
  - 行：资源类别（提供商、MCP、Agent、通道、巡航模式...）
  - 列：操作（列表、创建、修改、删除、使用）
  - 单元格：三态切换（✅ 允许 / ❌ 拒绝 / ➖ 继承默认）
- 特定资源 ID 的精细化控制（如只允许使用某个提供商）

### 5.2 导航权限控制

- 侧边栏项目根据当前用户权限动态显示/隐藏
- 路由守卫增加权限检查，无权限时跳转到 403 页面
- 操作按钮（如"添加提供商"）根据权限显示/隐藏

## 6. 实现步骤

### Phase 1：后端基础

1. 新增数据库迁移（`rbac_groups`、`rbac_user_groups`、`rbac_grants` 表 + auth_users.role 字段）
1. 新增 SeaORM 实体模型
1. 实现 RBAC API 路由（users、groups、grants CRUD）
1. 实现权限检查中间件/extractor
1. 在 JWT claims 中加入 role 字段

### Phase 2：后端集成

1. 在现有资源 API（providers、channels 等）中加入权限检查
1. 实现 `/api/rbac/check` 和 `/api/rbac/my-permissions`
1. 修改 arona 的资源请求以适配权限过滤

### Phase 3：前端 UI

1. 重构 arona 的 RbacView（用户/群组/权限矩阵三个 Tab）
1. 实现侧边栏和路由的权限守卫
1. arona 端根据权限隐藏/禁用功能（如巡航模式按钮）

## 7. 安全考虑

- `admin` 角色的权限不可被 `rbac_grants` 覆盖（硬编码放行）
- 权限检查在中间件层统一执行，不依赖业务代码手动检查
- 敏感操作（删除用户、修改权限）记录审计日志
- JWT 中只包含 role，具体权限每次从 DB 实时查询（避免权限变更后 token 未更新）
