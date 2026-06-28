+++
title = "RBAC System Design"
description = "Comprehensive role-based access control system for Shittim Chest"
lang = "en"
category = "design"
subcategory = "webui"
+++

# RBAC System Design

## 1. Objectives

Implement a comprehensive role-based access control system for Shittim Chest, supporting:

- **User management**: Admins can invite/create/disable/delete users
- **Group management**: Support for account groups; users may belong to multiple groups
- **Fine-grained permissions**: Control whether users can add/modify/use specific model providers, MCP tools, Layer3 agents, IM channels, etc.
- **Feature toggles**: Control whether users can use advanced features such as cruise control mode
- **Flexible authorization modes**: Admins can choose global uniform config, per-user config, or group-shared config

## 2. Core Concepts

### 2.1 Roles

| Role | Description |
| --- | --- |
| `admin` | Super administrator with all permissions; can manage RBAC itself |
| `operator` | Operations staff; can manage most resources (providers, channels, agents, etc.) |
| `member` | Regular member; can use authorized resources |
| `viewer` | Read-only user; can view but not modify |

Roles are **preset** — custom roles are not supported (to simplify implementation). Each user has one primary role.

### 2.2 Permissions

Permission format: `<resource>.<action>`

| Category | Permission | Description |
| --- | --- | --- |
| **Providers** | `provider.list` | View provider list |
| | `provider.create` | Add a provider |
| | `provider.update` | Modify provider configuration |
| | `provider.delete` | Delete a provider |
| | `provider.use` | Use a provider's models for chat |
| **MCP Tools** | `mcp.list` | View MCP tool list |
| | `mcp.create` | Register an MCP tool |
| | `mcp.update` | Modify MCP tool configuration |
| | `mcp.delete` | Delete an MCP tool |
| | `mcp.use` | Use MCP tools in conversations |
| **Agents** | `agent.list` | View agent list |
| | `agent.create` | Create an agent |
| | `agent.update` | Modify agent configuration |
| | `agent.delete` | Delete an agent |
| | `agent.use` | Use agents in analysis mode |
| **IM Channels** | `channel.list` | View IM channel list |
| | `channel.create` | Create an IM channel |
| | `channel.update` | Modify channel configuration |
| | `channel.delete` | Delete a channel |
| | `channel.use` | Send/receive messages through a channel |
| **Cruise Mode** | `yolo.use` | Use autonomous cruise control mode |
| **Workspaces** | `workspace.list` | View workspaces |
| | `workspace.create` | Create a workspace |
| | `workspace.manage` | Manage workspaces (delete, export) |
| **Devices** | `device.list` | View remote devices |
| | `device.connect` | Connect to a remote device |
| **System** | `system.read` | View system settings |
| | `system.write` | Modify system settings |
| | `rbac.manage` | Manage RBAC (users/groups/permissions) |
| **OAuth** | `oauth.read` | View OAuth configuration |
| | `oauth.write` | Modify OAuth configuration |

### 2.3 Default Role Permissions

| Permission | admin | operator | member | viewer |
| --- | --- | --- | --- | --- |
| `provider.*` | ✅ | ✅ | `list` + `use` | `list` |
| `mcp.*` | ✅ | ✅ | `list` + `use` | `list` |
| `agent.*` | ✅ | ✅ | `list` + `use` | `list` |
| `channel.*` | ✅ | ✅ | `list` + `use` | `list` |
| `yolo.use` | ✅ | ✅ | ❌ (off by default) | ❌ |
| `workspace.*` | ✅ | ✅ | `list` + `create` | `list` |
| `device.*` | ✅ | ✅ | `list` + `connect` | `list` |
| `system.*` | ✅ | ❌ | ❌ | ❌ |
| `rbac.manage` | ✅ | ❌ | ❌ | ❌ |
| `oauth.*` | ✅ | ✅ | ❌ | ❌ |

### 2.4 Authorization Modes

For resources such as providers, MCP, agents, channels, etc., three authorization modes are supported:

| Mode | Description | Use case |
| --- | --- | --- |
| **Global config** | All users share the same permissions | Small teams, personal use |
| **Per-user config** | Each user has independent resource permissions | Scenarios requiring fine-grained control |
| **Per-group config** | Users in the same group share permissions | Department/team-based partitioning |

The admin selects an authorization mode on the "Permission Matrix" page, then configures specific allow/deny rules.

**Priority**: Per-user > Per-group > Global > Role defaults

## 3. Database Schema

### 3.1 New Tables

#### `rbac_groups` — User groups

```sql
CREATE TABLE rbac_groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `rbac_user_groups` — User-group association

```sql
CREATE TABLE rbac_user_groups (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id   UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);
```

#### `rbac_grants` — Permission grants (unified table)

```sql
CREATE TABLE rbac_grants (
    id           UUID PRIMARY KEY,
    -- Grant target (exactly one)
    scope        VARCHAR(16) NOT NULL, -- 'global' | 'group' | 'user'
    user_id      UUID REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id     UUID REFERENCES rbac_groups(id) ON DELETE CASCADE,
    -- Permission
    permission   VARCHAR(64) NOT NULL, -- e.g. 'provider.use', 'yolo.use'
    resource_id  VARCHAR(128),         -- Optional: restrict to a specific resource (provider name, channel id, etc.); NULL means all resources in the category
    -- Grant type
    granted      BOOLEAN NOT NULL DEFAULT TRUE, -- TRUE=allow, FALSE=deny
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Constraint: scope and corresponding FK must be consistent
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

### 3.2 Modifying Existing Tables

#### Add role field to `auth_users`

```sql
ALTER TABLE auth_users ADD COLUMN role VARCHAR(16) NOT NULL DEFAULT 'member';
-- Migration: set users with is_admin=true to 'admin'
UPDATE auth_users SET role = 'admin' WHERE is_admin = TRUE;
```

The `is_admin` field is kept for backward compatibility, but new code uses `role` first.

### 3.3 Permission Check Logic (pseudocode)

```rust
fn has_permission(user, permission, resource_id=None) -> bool {
    // 1. admin role always passes
    if user.role == "admin" { return true; }

    // 2. Collect all matching grants, sorted by priority
    let grants = [];

    // 2a. Role defaults (lowest priority)
    grants.push(role_defaults(user.role, permission));

    // 2b. Global config
    grants.extend(query_grants(scope="global", permission, resource_id));

    // 2c. Group config (all groups the user belongs to)
    for group in user.groups:
        grants.extend(query_grants(scope="group", group_id=group.id, permission, resource_id));

    // 2d. User-level config (highest priority)
    grants.extend(query_grants(scope="user", user_id=user.id, permission, resource_id));

    // 3. Priority: user > group > global > role_default
    // Within the same scope, denied takes precedence over granted
    // Any user-scope denied → deny
    // Any group-scope denied → deny (unless user-scope granted)
    // Final result
    resolve_grants(grants)
}
```

## 4. API Design

### 4.1 User Management (`/api/rbac/users`)

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/api/rbac/users` | `rbac.manage` | List all users (with role, groups) |
| POST | `/api/rbac/users` | `rbac.manage` | Invite a user (send email or create account) |
| PUT | `/api/rbac/users/:id` | `rbac.manage` | Update user role, enable/disable |
| DELETE | `/api/rbac/users/:id` | `rbac.manage` | Delete a user |

### 4.2 Group Management (`/api/rbac/groups`)

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/api/rbac/groups` | `rbac.manage` | List all groups |
| POST | `/api/rbac/groups` | `rbac.manage` | Create a group |
| PUT | `/api/rbac/groups/:id` | `rbac.manage` | Update group (name, description) |
| DELETE | `/api/rbac/groups/:id` | `rbac.manage` | Delete a group |
| POST | `/api/rbac/groups/:id/members` | `rbac.manage` | Add a member |
| DELETE | `/api/rbac/groups/:id/members/:userId` | `rbac.manage` | Remove a member |

### 4.3 Permission Management (`/api/rbac/grants`)

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/api/rbac/grants` | `rbac.manage` | List all permission rules (supports ?scope=&permission= filtering) |
| PUT | `/api/rbac/grants` | `rbac.manage` | Batch set permissions (provide full rule list, overwrites the corresponding scope's rules) |
| DELETE | `/api/rbac/grants/:id` | `rbac.manage` | Delete a single rule |

### 4.4 Permission Check (`/api/rbac/check`)

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/api/rbac/check?permission=xxx&resource_id=yyy` | (any authenticated user) | Check whether the current user has the specified permission |
| GET | `/api/rbac/my-permissions` | (any authenticated user) | Return the current user's full list of effective permissions |

### 4.5 Resource Visibility Refactor

Existing resource APIs need permission filtering:

- `GET /api/chat/providers` → only return providers the current user has `provider.list` permission for, and only show models with `provider.use` permission
- `GET /api/channel` → only return channels with `channel.list` permission
- Before starting cruise mode → check `yolo.use` permission

## 5. Frontend Design

### 5.1 RbacView Refactor

Split into three tabs:

#### Tab 1: User Management

- User list table: avatar, username, email, role (dropdown switch), group tags, status (active/disabled), actions
- Invite user button → opens modal (enter username/email/password, select role)
- Row actions: edit role, disable/enable, delete

#### Tab 2: Group Management

- Group list table: name, description, member count, actions
- Create group → opens modal
- Click a group → expand member list, can add/remove members

#### Tab 3: Permission Matrix

- Top-left: select authorization mode: global / per-group / per-user
- After selecting a group or user, display the permission matrix table:
  - Rows: resource categories (providers, MCP, agents, channels, cruise mode...)
  - Columns: actions (list, create, modify, delete, use)
  - Cells: tri-state toggle (✅ allow / ❌ deny / ➖ inherit default)
- Fine-grained control for specific resource IDs (e.g., only allow a specific provider)

### 5.2 Navigation Permission Control

- Sidebar items dynamically show/hide based on the current user's permissions
- Route guards add permission checks; redirect to 403 page when unauthorized
- Action buttons (e.g., "Add Provider") show/hide based on permissions

## 6. Implementation Steps

### Phase 1: Backend Foundation

1. Add database migration (`rbac_groups`, `rbac_user_groups`, `rbac_grants` tables + auth_users.role field)
1. Add SeaORM entity models
1. Implement RBAC API routes (users, groups, grants CRUD)
1. Implement permission check middleware/extractor
1. Add role field to JWT claims

### Phase 2: Backend Integration

1. Add permission checks to existing resource APIs (providers, channels, etc.)
1. Implement `/api/rbac/check` and `/api/rbac/my-permissions`
1. Modify arona's resource requests to accommodate permission filtering

### Phase 3: Frontend UI

1. Refactor arona's RbacView (user/group/permission-matrix tabs)
1. Implement sidebar and route permission guards
1. Show/hide features based on permissions in arona (e.g., cruise mode button)

## 7. Security Considerations

- The `admin` role's permissions cannot be overridden by `rbac_grants` (hardcoded allow)
- Permission checks are enforced uniformly at the middleware layer, not relying on business code to manually check
- Sensitive operations (deleting users, modifying permissions) are recorded in an audit log
- JWT only contains the role; specific permissions are queried from the DB in real time (to avoid stale tokens after permission changes)
