+++
title = "Entelecheia 安全架构"
description = """> Entelecheia 多 Agent 编排平台的全方位纵深防御模型。"""
lang = "zhs"
category = "design"
subcategory = "core"
+++

# Entelecheia 安全架构

> Entelecheia 多 Agent 编排平台的全方位纵深防御模型。

## 概述

Entelecheia 实现了跨 14 个可独立测试的安全层级的**纵深防御安全架构**——从硬件级容器隔离到 LLM 面对的工具权限门控。与传统 Agent 框架将所有工具直接暴露给 LLM 不同，Entelecheia 的 **Exec-Only 微内核**设计意味着 LLM 仅看到 3 个原始工具（`exec`、`write_to_var`、`write_to_var_json`），而 148 个 MCP 工具通过类型化 IEPL 管线与多层授权进行分发。

## 安全层级索引

| # | 层级 | Crate(s) | 缓解的威胁 |
| --- | --- | --- | --- |
| 1 | Exec-Only 微内核 | `scepter`、`mcp_types` | LLM 无限制工具访问 |
| 2 | 双重授权权限门控 | `security_policy` | 未经授权的 MCP 工具调用 |
| 3 | 信任级别技能授权 | `domain_skills_permissions` | 通过技能链接权提升 |
| 4 | 容器隔离（外层） | `container`（Docker/Podman） | Agent 代码危及宿主机 |
| 5 | OCI 沙箱（内层）| `container_runtime`（Youki/libcontainer）| 容器逃逸 |
| 6 | RBAC 访问控制 | `domain_auth`、shittim-chest `rbac` | 未经授权的 API 访问 |
| 7 | JWT 认证 | shittim-chest `auth`（HS256）| Session 劫持、重放攻击 |
| 8 | API 密钥加密 | `aporia`（AES-256-GCM）| 凭证静态泄漏 |
| 9 | 安全哨兵 | `orexis`（OreXis Agent）| 恶意代码执行、合规违规 |
| 10 | IEPL 类型安全管线 | `iepl`、`iepl_engine`、`skemma` | 通过非类型化工具调用注入 |
| 11 | Provider 注册表白名单 | `config/registries.toml` | 通过不受信任包的供应链攻击 |
| 12 | Prompt 注入防御 | IEPL 沙箱边界 | 通过工具输出的 LLM Prompt 注入 |
| 13 | 速率限制 | shittim-chest `channel/rate_limit` | DoS、资源耗尽 |
| 14 | 审计追踪 | `orexis`、`timeline` | 事后取证、问责 |

---

## 第 1 层：Exec-Only 微内核

**Crates：** `scepter`、`mcp_types`
**设计理念：** 最小化 LLM 攻击面

LLM 在一个 **exec-only 沙箱**中运行，只能调用三个原始操作：

| 工具 | 用途 | 参数 |
| --- | --- | --- |
| `exec` | 执行脚本字符串 | JavaScript 代码（通过 IEPL 从 TypeScript 转译） |
| `write_to_var` | 存储字符串值 | 变量名 + 值 |
| `write_to_var_json` | 存储 JSON 值 | 变量名 + JSON 值 |

所有 148 个 MCP 工具（文件操作、容器管理、设备控制、网页搜索等）**对 LLM 不可见**。当 LLM 的 `exec` 调用 ES 模块导入（如 `import { file_read } from 'kalos'`）时，它们通过 IEPL 管线间接调用。

**威胁模型：** 即使 LLM 通过 Prompt 注入被攻陷，它也无法直接调用 `container_destroy` 或 `ssh_exec` 等危险工具。IEPL 管线在任何工具执行前强制实施类型检查和权限验证。

**实现：** `packages/shared/mcp_types/src/` 定义了微内核 IPC 类型。`packages/cosmos/` 中的 `exec` 处理器通过 Boa 引擎转译并执行脚本，工具调用通过 `skemma` 的 `McpRouter` 路由。

---

## 第 2 层：双重授权权限门控

**Crate：** `security_policy`（5,772 行）

每个 MCP 工具通过**权限级别**枚举声明其访问要求。每个技能（IEPL 脚本）声明其每个工具所需的权限级别。两者必须一致，调用才能继续。

```rust
pub enum PermissionLevel {
    /// 只读操作（file_read、list_dir 等）
    Read,
    /// 工作区内的写入操作（file_write、exec_script）
    Write,
    /// 影响外部系统的操作（ssh_exec、container_deploy）
    System,
    /// 有不可逆后果的操作（container_destroy、device_reboot）
    Destructive,
}
```

**授权流程：**

1. 技能声明："我需要 `System` 级别访问 `ssh_exec`"
1. 工具声明："我要求 `System` 权限"
1. 权限门控检查：`skill_level >= tool_requirement` 且 `skill 被显式授予此工具`
1. 若任一检查失败：调用被阻止、记录并报告给 OreXis 哨兵

**实现：** `packages/shared/security_policy/src/` — 107 个测试注解、4 个 tokio 测试。

---

## 第 3 层：信任级别技能授权

**Crate：** `domain_skills_permissions`（1,776 行）

技能被归类为决定其默认权限范围的**信任级别**：

| 信任级别 | 描述 | 默认权限 |
| --- | --- | --- |
| `Builtin` | 随平台发布 | 完整工具访问 |
| `Verified` | 经维护者审查和签名 | Read + Write |
| `Community` | 用户提交 | 仅 Read |
| `Untrusted` | 动态加载 | 无工具访问（仅 exec） |

每个技能的信任级别在加载时验证并缓存。尝试提升信任级别被记录为安全事件。

---

## 第 4 层：容器隔离（外环）

**Crate：** `container`（5,742 行）

每个 Agent 执行发生在一个 **Docker 或 Podman 容器**内，具有：

- 网络命名空间隔离
- 只读根文件系统（工作区挂载除外）
- 限制系统调用的 Seccomp 配置文件
- 资源限制（CPU、内存、PID 数量）
- 无宿主 Docker 套接字访问

**实现：** `packages/shared/container/src/` — 74 个测试注解、12 个 tokio 测试。支持 Docker（通过 Bollard API）和 Podman。

---

## 第 5 层：OCI 沙箱（内环）

**Crate：** `container_runtime`（3,645 行）

在 Docker 容器内部，Entelecheia 使用 Youki/libcontainer 运行**第二层隔离**——一个无守护进程、无根的 OCI 兼容容器运行时。这提供：

- 无根执行（不可能权限提升）
- 独立于 Docker 的命名空间隔离
- Cgroup v2 强制执行
- Seccomp 过滤器（默认拒绝）

**为什么需要两层？** Docker 提供粗粒度隔离（网络、文件系统）。Youki 提供细粒度系统调用过滤和资源计费。如果 Docker 被攻陷，Youki 沙箱仍然包含 Agent。

---

## 第 6 层：RBAC 访问控制

**Crates：** `domain_auth`（380 行）、shittim-chest `rbac`（1,736 行）

基于角色的访问控制管理所有 API 操作：

- **组：** 用户属于组；组拥有权限授予
- **授予：** 细粒度权限（每种资源类型的 read/write/admin）
- **工作区隔离：** 用户只能访问其所属的工作区
- **跨工作区操作：** 需要显式的管理员授予

---

## 第 7 层：JWT 认证

**模块：** shittim-chest `auth/jwt.rs`（264 行）

- **算法：** HS256（HMAC-SHA256）
- **访问令牌：** 短生命周期（可配置，默认 15 分钟）
- **刷新令牌：** 较长生命周期，使用时轮换
- **基于 Nonce 的 CSRF 保护**用于浏览器客户端
- **认证端点速率限制**（GCRA 算法）

---

## 第 8 层：API 密钥加密

**Crate：** `aporia`（5,802 行）

所有 LLM 提供商 API 密钥使用 **AES-256-GCM** 在静态加密，具有：

- 每次加密操作唯一 Nonce
- 由主密钥（环境配置）派生的密钥
- 使用后内存中明文密钥归零
- 密钥轮换支持

---

## 第 9 层：安全哨兵（OreXis）

**Crate：** `orexis`（5,239 行）——"免疫系统" Agent

OreXis 是一个 Layer-1 Agent，负责：

- **审计代码**以发现安全漏洞和许可证合规问题
- **根据已注册的安全策略检查工具调用**
- **按模式阻止/解除阻止**任何 Agent 的工具
- **监控** Agent 行为的异常模式

MCP 工具（24 个）：`standard_check`、`compliance_report`、`audit_alignment`、`audit_legality`、`agent_integrity`、`security_audit`、`tool_block`、`tool_unblock`、`policy_register`、`policy_list` 等。

---

## 第 10 层：IEPL 类型安全管线

**Crates：** `iepl`（2,670 行）、`iepl_engine`（1,228 行）、`skemma`（7,960 行）

**Entelecheia 插件语言**（IEPL）管线确保 LLM 生成代码与原生工具分发之间的类型安全：

1. LLM 使用 ES 模块导入生成 TypeScript 代码
1. **SWC** 将 TypeScript 转译为 JavaScript（语法验证）
1. **Boa 引擎**在沙箱化上下文中执行 JavaScript
1. ES 模块导入解析为 `__native_dispatch` 调用
1. 每次分发通过具有完整类型检查的 `McpRouter` 路由

**缓解的威胁：** 通过非类型化工具调用的注入攻击（在基于 Python 的 Agent 框架中常见，其中工具 schema 仅在运行时验证）。

---

## 第 11 层：Provider 注册表白名单

**文件：** `configs/registries.toml`（337 行）

Entelecheia 维护跨 15 个生态系统的受信任包注册表**硬编码白名单**：

crates.io、PyPI、npm、Go modules、Docker Hub、Maven Central、NuGet、RubyGems、Hackage、Alpine APK、Debian APT、GitHub、GitLab、`HuggingFace`、PyTorch。

从非白名单注册表导入的任何包在执行前在**容器级别被阻止**。

---

## 第 12 层：Prompt 注入防御

**机制：** IEPL 沙箱边界

LLM 的 `exec` 输出在一个**隔离的 Boa JS 上下文**中执行，无法访问：

- 宿主文件系统
- 网络套接字
- 环境变量
- 其他 Agent 的状态

返回给 LLM 的工具输出被**过滤**——二进制数据 base64 编码，过多输出被截断，工具结果中潜在的 Prompt 注入模式由 OreXis 标记。

---

## 第 13 层：速率限制

**模块：** shittim-chest `channel/rate_limit.rs`（118 行）

使用 **GCRA（通用信元速率算法）**的每用户、每通道速率限制：

- 可配置突发大小和持续速率
- 每用户 DashMap，实现 O(1) 查找
- 超限时自动回退
- API 调用、消息发送和工具调用的独立限制

---

## 第 14 层：审计追踪

**Crates：** `orexis`、`timeline`（3,096 行）

每次工具调用、Agent 决策和安全事件被：

1. 记录在**时间线**中，带完整上下文（Agent 工号、技能名称、参数、结果）
1. 哈希链接到之前的事件以实现防篡改
1. 持久化到 PostgreSQL，具有可配置的保留期
1. 可通过 CLI 查询（`entelecheia-cli trace-chain <badge>`）

---

## 与其他框架的安全对比

| 功能 | Entelecheia | OpenFANG | LangChain | Claude Code |
| --- |  ---  |  ---  |  ---  |  ---  |
| LLM 可见工具 | **3（exec-only）** | 53（全部可见） | 全部可见 | 33（全部可见） |
| 容器隔离 | **双层**（Docker + Youki）| 仅 WASM | 无 | OS 级（Seatbelt/Landlock） |
| 工具权限模型 | **双重授权** | RBAC | 无 | 无 |
| 代码审计 Agent | **OreXis（24 个工具）** | 循环守卫 | 无 | 无 |
| 类型安全分发 | **IEPL 管线** | 直接函数调用 | 直接函数调用 | 直接函数调用 |
| 包白名单 | **15 个注册表** | 无 | 无 | 无 |
| 审计追踪 | 哈希链接时间线 | Merkle 哈希链 | 无 | 无 |

---

## 威胁模型

### 超出范围

- 对宿主机的物理访问
- 被攻陷的 Docker/Podman 守护进程（假设受信任）
- 内核漏洞利用（用户空间隔离可缓解但不可阻止）
- Rust crate 依赖的供应链攻击（部分由 `cargo-deny` 缓解）

### 接受的风险

- Boa JS 引擎漏洞（在容器内沙箱化）
- LLM 提供商中断（无回退执行路径）
- PostgreSQL 数据损坏（由备份缓解，非阻止）

---

## 报告漏洞

漏洞报告流程见 [SECURITY.md](../SECURITY.md)。

## 许可

此安全架构是 Entelecheia 的一部分，根据 [BUSL-1.1](../LICENSE) 许可。
