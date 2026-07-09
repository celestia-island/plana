# arona — 项目状态与计划 (PLAN)

> 本文件由自动化扫描于 **2026-07-04** 生成，记录项目当前状态、近期进展与后续计划。
> 最近一次手动刷新：**2026-07-07**（新增 §6 通信协议结构测试计划）。
> 原有详细计划已保留于文末「既有详细计划（存档）」。

## 1. 项目概述

- **名称**：`arona`
- **简介**：celestia-island 共享协议类型库，含 TypeScript 绑定与构建脚本。
- **远程仓库**：https://github.com/celestia-island/arona.git
- **技术栈**：Rust / Node/TypeScript / just
- **类别**：rust-lib

## 2. 当前状态

- **当前分支**：`dev`
- **工作区**：干净（此前未提交的 `src/protocol/jsonrpc.rs` 改动已随 `8dcfdbf` 入库）
- **最近提交时间**：2026-07-04
- **最近提交**：8dcfdbf style: rustfmt jsonrpc notification fallback
- **分支对比**：`dev` 领先 `master` 127 个提交

## 3. 发布元数据补全（本次刷新完成）

本次刷新补全了面向 crates.io 的发布元数据，已通过 `cargo check` / `cargo clippy --all-features -- -D warnings` / `cargo test --lib`（651 项通过）/ `cargo fmt --check`：

- `README.md`：新增官方 docs.rs 徽章（`https://docs.rs/arona/badge.svg`）。
- `Cargo.toml`：补充 `keywords`、`categories`，新增 `[package.metadata.docs.rs]`（`all-features = true`）。
- 关键词：`json-rpc`、`protocol`、`mcp`、`typescript`、`schema`。
- 分类：`api-bindings`、`data-structures`、`web-programming::websocket`、`network-programming`。

## 4. 近期进展（最近提交）

- refactor: audit and clean up aspirational stub types（移除 `ModelCategory::Vision` / `ModelServerKind::MediaPipe`，见 R11）
- style: rustfmt jsonrpc notification fallback
- docs: add PLAN.md current-status snapshot
- feat(model): add ModelCapability enum, extend ModelCategory, add GenerationTier
- docs: simplify description
- chore: normalize dependency versions to caret (^) prefix
- chore: add CI workflow, rust-toolchain pin, relax schemars dep, update README/PLAN

## 5. 后续计划

1. ~~整理并提交当前未改动~~ — 已完成（`src/protocol/jsonrpc.rs` 随 `8dcfdbf` 入库）。
2. ~~完善 `crates.io` 发布元数据（rust-version / metadata / docs.rs badge）~~ — 已完成（`rust-version` 既有 `1.85`，本次补齐 keywords/categories 与 `[package.metadata.docs.rs]`，README 已加 docs.rs badge）。
3. 补充单元/集成测试，保持 `just test` 与 clippy `-D warnings` 通过。详见 §6「通信协议结构测试计划」。
4. 定期刷新本 PLAN.md 以反映最新状态。

---

## 6. 通信协议结构测试计划

> 本计划聚焦 arona 作为协议类型库的核心职责：确保所有通信协议结构（JSON-RPC、WebSocket 握手、MCP 智能体消息、WS 领域消息、HTTP API 类型、领域枚举、模型类型）的序列化/反序列化正确性、协议合规性与 TypeScript 绑定一致性。

### 6.1 当前测试现状总览

| 模块 | 现有测试 | 覆盖状态 |
|------|---------|---------|
| `protocol/jsonrpc.rs` | 25 项 | 🟢 良好 — Id 往返、UUID v7、请求/通知/响应分类、null-id 边界、错误码、build_notification 回退链 |
| `protocol/handshake.rs` | 0 项 | 🔴 无 — 握手协议是安全关键路径 |
| `protocol/base_messages.rs` | 0 项 | 🔴 无 — 心跳/错误/确认基础消息 |
| `mcp/haplotes.rs` | 7 项 | 🟡 部分 — AgentReference、ConflictInfo、Reasoning、Conversation 序列化 |
| `mcp/kalos.rs` | 3 项 | 🟡 部分 — FileReadResult/FileEditResult 与冲突字段 |
| `external_mcp.rs` | 5 项 | 🟢 良好 — TOML 解析覆盖主要场景 |
| `mcp/` 其余 11 个智能体 | 0 项 | 🔴 无 — hubris/neikos/skemma/skopeo/aporia/eleos/epieikeia/orexis/philia/polemos/web_automation |
| `ws/` 全部 16 个子模块 | 0 项 | 🔴 无 — agent_lifecycle/layer2/state_sync/tasks/yolo/auth/industrial/knowledge_base/llm_provider/bridge_network/file_browsing/logs/noa/system_ui/views/workspace |
| `http.rs` (90+ 结构体) | 0 项 | 🔴 无 — REST API 响应类型全部未测试 |
| `model.rs` | 0 项 | 🔴 无 — 模型描述符/能力枚举/推理类型 |
| `enums.rs` (14 个 str_enum!) | 0 项 | 🔴 无 — 领域词汇枚举的 as_str()/Display/From 实现 |
| `identity.rs` | 0 项 | 🟡 低优 — 机器指纹（平台相关，难以单元测试） |
| TypeScript 绑定 (ts-rs) | ~642 项（自动生成） | 🟢 良好 — 自动生成的 ser/de 往返测试 |

### 6.2 测试层次定义

| 层次 | 描述 | 目标 |
|------|------|------|
| **L1 — 序列化往返** | `serde_json::to_value` → `serde_json::from_value` 往返无损 | 每个公开结构体至少 1 个 |
| **L2 — JSON 形状快照** | 验证输出的精确 JSON 结构（字段名、类型、缺失字段处理） | 核心消息类型全覆盖 |
| **L3 — 协议合规** | 验证符合 JSON-RPC 2.0 / WebSocket 握手等外部规范 | 协议边界类型 |
| **L4 — 边界/错误** | 非法输入拒绝、可选字段的 None 处理、默认值行为 | 带 `#[serde(default)]` 或 `Option` 的类型 |
| **L5 — 跨模块集成** | JSON-RPC 消息携带 MCP/WS 参数时整体序列化正确 | 关键消息路径 |
| **L6 — TypeScript 一致性** | 验证 Rust 序列化输出能被 TypeScript 绑定正确反序列化 | 核心双向通信类型 |

### 6.3 分阶段测试计划

---

#### Phase 1 — 安全关键路径 (P0) | 预计工作量: 3-5 天

##### 1.1 WebSocket 握手协议 (`protocol/handshake.rs`)

**理由**: 握手是客户端接入的第一个安全门禁。token 传输、协议版本协商、能力声明的任何序列化错误都将导致连接建立失败或安全降级。

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `ConnectHandshakeParams` 完整往返 | L1, L2 | 全部字段填充；JSON 形状验证（`protocol_version` 默认值 1） |
| `ConnectHandshakeParams` 最小字段 | L4 | 仅 token 填充时其余 Option 字段为 null/absent |
| `ConnectHandshakeParams` 空 capabilities | L4 | `capabilities: []` 正确处理 |
| `ConnectHandshakeParams` 多余未知字段 | L4 | 前端升级后携带新字段，旧版 arona 应忽略（`#[serde(deny_unknown_fields)]` 检查是否误用） |
| `ClientCapability` 枚举全变体序列化 | L1, L2 | FileRelay / Terminal / ScreenCapture / NoaWorkspace 的 JSON 表示 |
| `HandshakeAckParams` 成功/失败 | L1, L2 | `ok: true` vs `ok: false` + error 字符串 |
| `PingParams` 往返 | L1 | timestamp 正整数的 JSON 表示 |
| `ScepterIdentityParams` 往返 | L1 | Uuid 的 JSON 序列化格式（带/不带连字符） |
| `ClientNodeInfo` 完整/最小填充 | L1, L2, L4 | 可选字段 `workspace_root` / `user_id` 为 None 时的行为 |
| `HANDSHAKE_VERSION` 常量值验证 | L3 | 确保版本号语义化且不被意外修改 |

##### 1.2 WebSocket 基础消息 (`protocol/base_messages.rs`)

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `BaseHeartbeatParams` 往返 | L1, L2 | timestamp 字段；心跳消息不含 `method` 外字段时的处理 |
| `BaseErrorParams` 往返 | L1, L2 | code + message 字符串；验证 JSON 不含多余字段 |
| `BaseAckParams` 往返 | L1, L2 | message_id 字符串；确认消息的精确 JSON 形状 |

##### 1.3 JSON-RPC 核心补充 (`protocol/jsonrpc.rs`)

现有 25 个测试已覆盖良好，补充以下边界场景：

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| Batch 请求数组（空数组） | L3 | JSON-RPC 2.0 规定空 batch 应返回空数组 |
| Batch 请求数组（全是通知） | L3 | 全部为 Notification 的 batch 不应返回任何响应 |
| Batch 请求数组（混合） | L3 | 同时包含 Request + Notification 时各自的路由行为 |
| `JsonRpcError` 标准错误码验证 | L3 | -32700/-32600/-32601/-32602/-32603 的 message 不为空 |
| `JsonRpcError` 自定义错误码验证 | L3 | -32001 到 -32005 的 message 语义正确 |
| `Id::Number` 负数/零/大数 | L4 | 非正整数的 Id 序列化行为 |
| `JsonRpcMessage` 序列化（非反序列化） | L1 | `Serialize` 方向的枚举表示（untagged），确保 Request 写出的 JSON 不含 `result`/`error` |
| 超大 params 负载 | L4 | 嵌套深度/体积较大的 params JSON 能否正常往返（防止栈溢出/截断） |
| `build_notification` / `build_notification_value` with Unicode method | L4 | 非 ASCII 方法名的序列化（R6-R8 修复链应覆盖到） |

---

#### Phase 2 — MCP 智能体协议 (P1) | 预计工作量: 5-8 天

**理由**: MCP 类型是 entelecheia 智能体与 shittim-chest 前端之间具体业务通信的契约。以下按智能体使用频率和复杂度排列。

##### 2.1 核心智能体（高优先级）

###### kalos（文件系统操作）— 已有 3 个测试，需扩展

| 新增测试项 | 层次 |
|------------|------|
| `FileEntry` 文件/目录两种变体 | L1, L2 |
| `FileListResult` 空列表/多条目 | L1, L4 |
| `FileReadResult` 含注释/不含注释 | L1 |
| `FileWriteResult` 往返 | L1 |
| `FileEditResult` conflicts 为 None/空 Vec | L4 |
| `FileTreeEntry` 递归嵌套结构 | L1, L2 |
| `FileTreeListResult` 深层目录树 | L1, L4 |
| `Annotation` 创建/解析 | L1 |
| `ListAnnotationsResult` 往返 | L1 |
| 所有 `*Params` 结构体的可选字段省略 | L4 |

###### haplotes（智能体协作/LLM）— 已有 7 个测试，需扩展

| 新增测试项 | 层次 |
|------------|------|
| `FileAnchor` 完整/最小化 | L1, L2 |
| `ConversationContext` 含多条消息 | L1 |
| `AgentConversation` 含 `ConversationStatus` 各变体 | L1 |
| `ChatMessage` 往返 | L1 |
| `LlmProviderCallParams` 多 Provider 切换 | L1, L4 |
| `SubscribeTriggerParams` 往返 | L1 |
| `FileLineRange` 边界值（start ≥ end?） | L4 |

###### neikos（容器管理）

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `ContainerInfoResult` 完整字段 | L1, L2 | ContainerStatus 各变体映射 |
| `ContainerListItem` / `ContainerListResult` | L1, L4 | 空列表/多容器 |
| `ContainerCreateResult` / `ContainerStartResult` / `ContainerStopResult` | L1 | 操作结果往返 |
| `ContainerSnapshotResult` | L1 | 快照数据 |
| `VolumeInfo` 往返 | L1 |
| `ExecResult` stdout/stderr/exit_code | L1, L2 |
| `GitPushResult` 成功/失败 | L1 |
| `ToolchainProfileInfo` / `ToolchainListResult` | L1 |
| 所有 `*Params` 结构体最少字段构造 | L4 |

###### skemma（脚本/远程执行/工业协议）

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `ScriptExecResult` stdout/stderr/exit | L1, L2 |
| `RemoteConnectionInfo` / `ListRemotesResult` | L1 |
| `ConnectRemoteResult` / `DisconnectRemoteResult` | L1 |
| `ScreenshotResult` base64 数据 | L1, L4 |
| `ModbusReadResult` / `ModbusWriteResult` | L1, L2 | 工业协议数据形状 |
| `SignalNormalizeResult` | L1 |
| `SignalStats` 数值边界 | L4 |

###### orexis（安全/合规/审计）— 安全关键

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `CheckResultItem` 通过/失败 | L1 |
| `SensitivityRule` 匹配模式 | L1 |
| `AuditAlignmentResult` / `AuditLegalityResult` | L1, L2 |
| `AuditFinding` 严重级别 | L1 |
| `ComplianceSummary` / `ComplianceReportToolResult` | L1 |
| `ComplianceRule` 往返 | L1 |

##### 2.2 辅助智能体（中等优先级）

###### hubris（Todo 管理）

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `TodoTreeNode` 嵌套子节点 | L1, L2 | 递归树结构 |
| `TodoListItem` / `TodoListResult` | L1 |
| `TodoTreeListResult` 深层嵌套 | L1, L4 |
| `TodoClearDryRunResult` / `TodoClearResult` | L1 |
| `ListTodoParams.normalize()` 行为 | L3 | 参数规范化逻辑 |
| `ReportParams` / `ReportHumanParams` | L1 |

###### skopeo（目标/轨道/任务）

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `GoalEntry` / `GoalCreateResult` / `GoalUpdateResult` | L1 |
| `TrackEntry` / `TrackCreateResult` | L1 |
| `GoalTaskEntry` / `GoalTaskCreateResult` | L1 |
| `AlignmentCheckResult` | L1 |
| `GoalStatus` / `TrackStatus` / `GoalTaskStatus` 变体序列化 | L1 |

###### aporia（RAG/知识/分析）

| 测试项 | 层次 |
|--------|------|
| `RagDbWriteResult` / `RagDocResult` / `RagDbReadResult` | L1 |
| `RagDbDeleteResult` / `RagDbStatsResult` | L1 |
| `WorkspaceSearchDoc` / `WorkspaceSearchResult` | L1 |
| `CorrelationInfo` / `Hypothesis` / `CausalReasonResult` | L1 |
| `AnomalyInfo` / `AnomalyResult` | L1 |

###### eleos（Web 搜索）

| 测试项 | 层次 |
|--------|------|
| `WebSearchItem` / `WebSearchResult` | L1 |
| `WebFetchResult` markdown/text 内容 | L1 |
| `RemoteRefEntry` / `QueryRemoteRefsResult` | L1 |

###### epieikeia（触发器/跨智能体投递）

| 测试项 | 层次 |
|--------|------|
| `TriggerEntry` / `TriggerListResult` | L1 |
| `TaskEntry` / `TaskListResult` | L1 |
| `DeliverMessageParams` / `DeliverMessageResult` | L1 |
| `InjectUserPromptParams` / `InjectedPromptView` | L1 |
| `NotifyFileOperationToolResult` | L1 |

###### philia（智能体注册/记忆/时序）

| 测试项 | 层次 |
|--------|------|
| `AgentRegistryEntry` / `AgentRegistryListResult` | L1 |
| `McpToolDetail` / `SkillDetail` | L1 |
| `MemoryStoreResult` / `MemoryQueryResult` | L1 |
| `MemorySubgraphResult` 图结构 | L1 |
| `TimeseriesPointResult` / `TimeseriesQueryResult` | L1 |

###### polemos（网络节点/设备发现）

| 测试项 | 层次 |
|--------|------|
| `NodeInfo` / `NodeDiscoverResult` | L1 |
| `ProtocolProbeResult` / `ProtocolProbeResponse` | L1 |
| `DeviceRegisterRangeResult` / `DeviceCapability` | L1 |
| `KneeJerkTest` / `AdaptiveProbeResult` | L1 |
| `Phase1Result` / `Phase2Result` | L1 |

###### web_automation（浏览器自动化）

| 测试项 | 层次 |
|--------|------|
| `BrowserInstanceInfo` / `BrowserListResult` | L1 |
| `BrowserNavigateResult` / `BrowserScreenshotResult` | L1 |
| `BrowserConsoleLogEntry` / `BrowserConsoleLogsResult` | L1 |
| `BrowserNetworkEntry` / `BrowserNetworkLogsResult` | L1 |
| `BrowserScriptResult` / `BrowserRecordResult` | L1 |

---

#### Phase 3 — WebSocket 领域消息 (P1) | 预计工作量: 4-6 天

##### 3.1 智能体生命周期 (`ws/agent/`)

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `AgentStreamingChunkParams` 各 StreamSegment 变体 | L1, L2 | Text/Thinking/DeepThinking/McpCall/McpResult |
| `AgentResponseParams` 完整/最小 | L1, L4 |
| `AgentReportParams` 各 ReportType 变体 | L1, L2 | ReportType 枚举与 JSON 字符串映射 |
| `AgentReportReplyParams` | L1 |
| `OrchestrationStatusParams` | L1 |
| `McpToolResultParams` 成功/失败 | L1, L2 |
| `TuiAgentInfo` / `AgentListResponseParams` | L1 |

##### 3.2 状态同步 (`ws/agent/state_sync.rs`)

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `GlobalSnapshotParams` / `GlobalSnapshotData` | L1, L2 | 全量快照的 JSON 体积与结构 |
| `ContainerSnapshotParams` / `ContainerSnapshotData` | L1 |
| `TasksSnapshotParams` / `TasksSnapshotData` | L1 |
| `ContainerInfo` 各 ContainerStatus 变体 | L1 |
| `TaskInfo` 各 TaskStatus 变体 | L1 |

##### 3.3 YOLO 模式 (`ws/agent/yolo.rs`)

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `YoloStartResponseParams` / `YoloStopResponseParams` | L1 |
| `YoloTaskResult` / `YoloTaskStatus` 各 YoloTaskTier | L1 |
| `YoloStatusResponseParams` / `YoloConfigResponseParams` | L1 |
| `YoloTierConfig` 各层配置 | L1, L2 |

##### 3.4 Layer2 (`ws/agent/layer2.rs`)

| 测试项 | 层次 |
|--------|------|
| `Layer2AgentInfo` / `Layer2AgentListResponseParams` | L1 |
| `Layer2McpToolInfo` / `Layer2AgentMcpResponseParams` | L1 |
| `Layer2SkillInfo` / `Layer2AgentSkillsResponseParams` | L1 |
| `CustomAgentInfo` / `CustomAgentListResponseParams` | L1 |

##### 3.5 服务消息 (`ws/services/`)

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `AuthLoginResponseParams` / `AuthRegisterResponseParams` | L1, L2 | token 格式、用户信息 |
| `IndustrialSensorReading` / `IndustrialAlarmEvent` | L1, L2 | 工业遥测数据的精确 JSON 形状 |
| `IndustrialAlarmLevel` 枚举映射 | L1 | 告警级别与数值对应 |
| `IndustrialDiscoveryProgress` | L1 | 设备发现阶段推进 |
| `WriteApprovalRequest` / `WriteApprovalResponseParams` | L1 | 安全写操作的审批流程 |
| `IndustrialTelemetryBatch` / `IndustrialTopologyParams` | L1 |
| `KnowledgeBaseInfo` / `ListKnowledgeBasesResponseParams` | L1 |
| `ConfiguredProviderInfo` / `ConfiguredProvidersListParams` | L1 |
| `EntrypointConfigInfo` / `ProviderCapabilitiesInfo` | L1 |
| `UsagePeriodData` / `UsagePeriodResponseParams` 空/有数据 | L1, L4 |
| `ModelFsInfo` / `ProviderFsInfo` | L1 |

##### 3.6 UI 消息 (`ws/ui/`)

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `DashboardLayout` / `DashboardLayoutPushParams` | L1 |
| `ViewInstance` / `ViewLayout` | L1 |
| `ViewKind` 所有变体 | L1 |
| `FileTreeParams` / `FileReadParams` | L1 |
| `HostMetrics` / `BridgeNetworkParams` | L1 |
| `WorkspaceNode` / `WorkspaceGitStatus` | L1 |
| `ServerLogEntryParams` / `ContainerLogEntryParams` | L1 |
| `NoaEvent` / `NoaHandshakeResponseParams` | L1 |
| `NoaAuthRequestParams` / `NoaAuthResponseParams` | L1 |
| `WorkspaceStatusParams` | L1 |
| `PolemosDeviceInfo` / `PolemosDeviceListParams` | L1 |

---

#### Phase 4 — HTTP API 类型 (P2) | 预计工作量: 3-5 天

`src/http.rs` 包含 90+ 个面向 shittim-chest REST API 的响应结构体。所有这些类型仅派生 `Serialize + TS`（用于前端类型生成），无需 `Deserialize`（后端负责构造，前端负责消费）。

##### 4.1 测试策略

由于这些类型是单向的（后端序列化 → 前端反序列化），L1 往返测试不适用。改为：

- **L2 — JSON 形状快照**: 每个结构体至少验证一个具例的 JSON 输出
- **L4 — 边界处理**: 可选字段 `None` 时 `skip_serializing_if` 正确省略；空 Vec 输出 `[]` 而非 `null`

##### 4.2 分批覆盖

**Batch A — 核心响应（高优先级）**:
`HealthResponse`, `HealthDetailed`, `ConnectionStatus`, `StatusResponse`, `ErrorResponse`, `OkResponse`, `OkIdResponse`, `IdResponse`, `CreatedResponse`, `DeletedResponse`, `OkMessageResponse`

**Batch B — 用户/RBAC**:
`RbacUser`, `RbacUsersResponse`, `RbacGroup`, `RbacGroupsResponse`, `MyPermissions`, `PermissionsResponse`, `OAuthProvider`, `UserProfileResponse`, `UserPreferences`, `UserTierInfo`, `TierDefinition`, `TierListResponse`, `UpdateUserTierPayload`

**Batch C — 智能体/技能/工具**:
`AgentItem`, `AgentTool`, `AgentConfig`, `AgentContainer`, `AgentListResponse`（如有）, `SkillItem`, `SkillParameterItem`, `ToolItem`

**Batch D — Provider/模型/用量**:
`ProviderPublic`, `ModelInfo`, `VendorInfo`, `ValidateKeyResponse`, `TokenUsageResponse`, `UsageDataResponse`, `UsageEntry`, `UsageModelEntry`, `UsageDayEntry`

**Batch E — 系统/工作区**:
`SystemInfoResponse`, `SystemInfoAgents`, `SystemInfoResources`, `SystemInfoConnections`, `SystemInfoDatabase`, `ProxySystemInfo`, `WorkspaceItem`, `AliasRegistryEntry`, `WorkspaceResolveResponse`, `ProjectItem`

**Batch F — 场景/3D**:
`SceneConfigItem`, `SceneGround`, `SceneLighting`, `SceneGrid`, `SceneCamera`, `SceneCameraBookmark`, `SceneVec3`, `SceneBloom`

**Batch G — 通道/Webhook/设备**:
`ChannelListItem`, `ChannelListResponse`, `ChannelConfigDetail`, `ChannelConfigResponse`, `ChannelMessageItem`, `ChannelMessageListResponse`, `WebhookItem`, `WebhookListResponse`, `WebhookDeliveryItem`, `DeliveryListResponse`, `DeviceResponse`, `IpWhitelistResponse`

**Batch H — 文件/配额/会话**:
`FileListingResponse`, `FileEntry`, `SessionCreateResponse`, `ReadinessResponse`, `SetupCheckResponse`, `ResourceQuota`, `ResourceQuotaListResponse`, `ResourceUsageSummary`, `ResourceUsageResponse`

---

#### Phase 5 — 领域枚举与模型类型 (P2) | 预计工作量: 1-2 天

##### 5.1 str_enum! 宏枚举 (`enums.rs`)

14 个由 `str_enum!` 宏生成的枚举需要验证：

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| 每个枚举的 `as_str()` 返回预期字符串 | L3 | 确保 str 常量与 serde rename 一致 |
| 每个枚举的 `Display` 实现输出 as_str() | L3 | 与 `as_str()` 等价 |
| 每个枚举的 `From<Enum> for String` | L3 | 与 `as_str()` 等价 |
| serde 序列化使用 as_str() 值 | L1, L2 | 非默认 Enum 序列化格式 |
| serde 反序列化从字符串解析 | L1 | 往返一致性 |

覆盖枚举: `FileOpStatus`, `FileType`, `ContainerOpStatus`, `ConsultationStatus`, `WebSearchEngine`, `ScriptLanguage`, `ObservationType`, `FileOperationType`, `ConversationStatus`, `ConversationMessageType`, `AnnotationType`, `GoalStatus`, `TrackStatus`, `GoalTaskStatus`, `ConnectionType`

##### 5.2 模型类型 (`model.rs`)

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `ModelCategory` 各变体 serde 字符串映射 | L1 | 与 provider-registry 的 TOML 配置一致 |
| `ModelCapability` 各变体 | L1 |
| `GenerationTier` 各变体 | L1 |
| `ModelBackend` 各变体 | L1 |
| `ModelServerStatus` / `ModelServerKind` / `ModelServerAction` | L1 |
| `ModelDescriptor` 完整填充 | L1, L2 |
| `ModelDescriptor` 可选字段省略 | L4 | `dimension`/`size_bytes`/`hardware_requirements` 等为空时 |
| `HardwareRequirements` 各字段可选 | L4 |
| `ModelServerInfo` 往返 | L1 |
| `ModelInferenceRequest` / `ModelInferenceResult` | L1 |

##### 5.3 核心枚举补充 (`lib.rs`)

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| `Agent::all()` 返回 13 个变体且互不重复 | L3 | 防止新增变体漏加 |
| `Agent` 的 serde 变体名与 MCP 模块名对应 | L3 | 确保 `"skopeo"` 映射到 `Agent::SkoPeo` |
| `AgentStatus` / `WorkStatus` / `RequestState` 各变体 | L1 |
| `ReportType::is_query()` / `is_error()` / `is_pending()` / `is_terminal()` | L3 | 分类逻辑穷尽所有变体 |
| `AgentErrorCode` 20 个变体及其 `is_*_error()` 分类 | L3 | 分类逻辑穷尽且互斥 |
| `StreamSegment` 5 个变体（含嵌套 McpCall/McpResult） | L1, L2 |
| `StructuredAgentError` 序列化格式 | L1, L2 | code + detail + context HashMap |

---

#### Phase 6 — 跨模块集成测试 (P2) | 预计工作量: 2-3 天

##### 6.1 JSON-RPC 携带 MCP 参数

```
JsonRpcRequest {
    method: "mcp.call",
    params: Some(serde_json::to_value(Kalos::FileReadParams { ... })?),
}
```

验证：
- `FileReadParams` 正确嵌套在 `params` 字段中
- 反序列化能恢复完整的 `JsonRpcRequest`，且 `params` 可二次解析为 `FileReadParams`
- 对 haplotes/neikos/skemma/orexis 的核心请求类型重复此测试

##### 6.2 JSON-RPC 携带 WS 消息参数

```
JsonRpcNotification {
    method: "ws.agent.streaming",
    params: Some(serde_json::to_value(AgentStreamingChunkParams { ... })?),
}
```

验证：
- `AgentStreamingChunkParams` 各 StreamSegment 变体的嵌套序列化
- `AgentResponseParams` / `AgentReportParams` / `McpToolResultParams` 类似测试

##### 6.3 通知构建管道

```
// 从 MCP 类型构造 JSON-RPC 通知字符串
let params = serde_json::to_value(&neikos::ContainerCreateResult { ... })?;
let json_str = build_notification("neikos.container.created", Some(params));
// 验证生成的字符串能被解析为有效的 JsonRpcNotification
```

验证 `build_notification` / `build_notification_value` 对真实 MCP/WS 类型的兼容性。

##### 6.4 TypeScript 绑定一致性

| 测试项 | 层次 | 覆盖要点 |
|--------|------|---------|
| Rust 序列化输出 → TypeScript 类型定义匹配 | L6 | 选取 5-10 个核心消息类型，用 Rust 生成 JSON，对照 `bindings/` 中 TS 类型定义验证字段对应 |
| ts-rs `#[ts(export_to)]` 路径正确性 | L3 | 所有 `#[ts(export_to = "...")]` 路径实际存在于 `bindings/` |
| `bindings/index.ts` 导出完整性 | L3 | 每个 `bindings/` 子模块在 index.ts 中有对应的 re-export |

---

### 6.4 测试基础设施建议

#### 测试组织

```
src/
  protocol/
    jsonrpc.rs          (保留现有 #[cfg(test)] mod tests)
    handshake_tests.rs  (新建 — 独立测试模块)
  mcp/
    kalos_tests.rs      (新建 — 将现有 kalos.rs 内测试迁移至此)
    haplotes_tests.rs   (新建 — 同上)
    neikos_tests.rs     (新建)
    ...                 (每个智能体一个独立测试文件)
  ws/
    agent/
      agent_lifecycle_tests.rs  (新建)
      state_sync_tests.rs       (新建)
      ...
  http_tests.rs         (新建 — HTTP API 类型的 JSON 快照测试)
  model_tests.rs        (新建 — 模型类型的序列化测试)
  enums_tests.rs        (新建 — str_enum! 宏枚举的 Display/as_str 测试)
  integration_tests.rs  (新建 — 跨模块集成测试)

tests/
  integration/          (新建 — 集成测试目录)
    jsonrpc_roundtrip.rs
    ts_binding_sync.rs
    snapshots/          (insta 快照文件)
```

#### 测试工具建议

- **`insta`** (`cargo add --dev insta`): JSON 快照测试，自动检测结构变化，适合 HTTP API 类型和 WS 消息类型的 L2 形状验证
- **`serde_json::json!`** 宏: 构造预期 JSON 进行精确比对
- **`pretty_assertions`** (`cargo add --dev pretty-assertions`): 大结构体 diff 可读性
- **测试辅助宏**: 编写一个 `roundtrip_test!` 宏减少样板代码:

```rust
macro_rules! roundtrip_test {
    ($name:ident, $ty:ty, $val:expr) => {
        #[test]
        fn $name() {
            let original: $ty = $val;
            let json = serde_json::to_value(&original).expect("serialize");
            let back: $ty = serde_json::from_value(json).expect("deserialize");
            assert_eq!(original, back);
        }
    };
}
```

#### CI 集成

- 新增 `just test-protocol` 配方: 仅运行协议结构测试（快速反馈）
- 在现有 GitHub Actions `ci.yml` 中添加 `--all-features` 测试矩阵
- 添加 `cargo test --doc` 确保文档内示例代码可编译

---

### 6.5 测试覆盖目标

| 阶段 | 完成标志 | 预计新增测试数 |
|------|---------|--------------|
| Phase 1 (P0) | 握手协议 + 基础消息 + JSON-RPC 补充全部 L1-L4 | +30 |
| Phase 2 (P1) | 13 个 MCP 智能体核心类型 L1 + 重点类型 L2 | +120 |
| Phase 3 (P1) | 16 个 WS 领域模块核心类型 L1 + 关键类型 L2 | +100 |
| Phase 4 (P2) | 90+ HTTP API 类型快照测试 (Batch A-D 优先) | +90 |
| Phase 5 (P2) | 14 个 str_enum! 枚举 + model.rs + lib.rs 核心枚举 | +40 |
| Phase 6 (P2) | 集成测试 + TS 绑定一致性 | +20 |
| **总计** | | **~400 项新增测试** |

达标后 `cargo test --all-features` 预计从 **651** → **1050+** 项。

---

## Strengths (for reference)
- Excellent documentation: 8 language translations, architecture/design/guides/meta docs
- TypeScript bindings auto-generated via `ts-rs` — full-stack type safety
- Clean separation: domain vocab enums, WS message params, HTTP API types, JSON-RPC core
- Consistent serde rename conventions with backward-compatible `Option` fields
- 642 auto-generated TS binding ser/de tests + hand-written unit tests for JSON-RPC and TOML parsing
- `JsonSchema` derive on all core enum/struct types for schema-aware consumers

---

## 既有详细计划（存档）

# arona — Issues & Action Plan

Generated 2026-06-30 from deep code audit. Updated 2026-07-02 (R3). Updated 2026-07-03 (R4). Updated 2026-07-03 (R5). Updated 2026-07-03 (R6). Updated 2026-07-03 (R7). Updated 2026-07-03 (R8). Updated 2026-07-03 (R9). Updated 2026-07-03 (R10).

arona is the shared protocol crate (v0.1.0) that glues entelecheia and shittim-chest via JSON-RPC types and TypeScript bindings. It also serves as the ecosystem documentation hub.

## Resolved (R4)

### 1. `PROTOCOL_VERSION` constant is unused — RESOLVED (R4)
- Defined in `lib.rs` but never referenced. Retained as a canonical declaration of the platform's advertised protocol version.

### 2. Schemars coverage gaps — RESOLVED (R4)
- Added `JsonSchema` derive to: `AgentErrorCode`, `StreamSegment`, `LlmStream`, `RouteInfo`, `StructuredAgentError`, `PeriodType`, `EmbeddingModel`, `YoloTaskTier`.
- Updated `examples/schema_dump.rs` with all types that carry `JsonSchema`.

### 3. Agent enum aspirational variants removed — RESOLVED (R4)
- Removed `ClassicSoftwareEngineering`, `WebUiPanel`, `IndustrialIoT`, `RemoteOperations` — none had corresponding MCP type modules or agent implementations.
- Agent enum now has 13 variants, all backed by `mcp/` modules.

### 4. Handshake version negotiation — RESOLVED (R3)
- `ConnectHandshakeParams` carries `protocol_version: u32` (default v1), satisfying the version-gating requirement.

## Resolved (R5)

### 5. All types are data-only — logic divergence risk — RESOLVED (R5)
- Added shared validation/categorization impl blocks on core types:
  - `Agent::all()` — returns all 13 variants for iteration
  - `ReportType::is_query()`, `is_error()`, `is_pending()`, `is_terminal()` — wire-level classification both sides can replicate
  - `AgentErrorCode::is_llm_error()`, `is_cosmos_error()`, `is_chain_error()`, `is_skill_error()`, `is_model_selection_error()` — error categorization for routing/retry logic
- These functions serve as documentation contracts for behavior that both entelecheia and shittim-chest must agree on.

### 6. JSON-RPC `Id` UUID v7 format agreement — RESOLVED (R5)
- Added doc comment on `Id::new_uuid()` stating UUID v7 is the canonical wire format for the platform.
- Added `Id::new_uuid_v4()` as an opt-in alternative for consumers that prefer random UUIDs.
- TypeScript consumers should use a UUID v7 library for format parity or treat IDs as opaque sortable strings.
- Existing test `id_new_uuid_is_v7_shaped_string` verifies the v7 format.

## Resolved (R6)

### 7. `ExternalToolInfo` missing serialization traits — RESOLVED (R6)
- `ExternalToolInfo` (external_mcp.rs) is a public type representing tool info from external MCP servers, but lacked `Serialize`, `Deserialize`, and `PartialEq` derives.
- Added all three derives so consumers can serialize, deserialize, and compare tool info from external MCP servers.

### 8. `build_notification` ultimate fallback produces invalid JSON-RPC — RESOLVED (R6)
- The absolute `.unwrap_or_else` fallback in `build_notification` produced `{"method":""}` — an empty method string, which violates JSON-RPC 2.0 spec (method MUST be a non-empty string).
- Fixed the fallback to use `"internal.fallback"` as a sentinel method name, ensuring all fallback paths produce valid JSON-RPC messages.

## Resolved (R7)

### 9. R6 fix incomplete — `build_notification` and `build_notification_value` first-level fallbacks still produce potentially-invalid JSON-RPC — RESOLVED (R7)
- R6 #8 only fixed the innermost/ultimate `.unwrap_or_else` fallback. The first-level fallback in both `build_notification` and `build_notification_value` still passed through the raw `method` argument, which could be an empty string (`{"method":""}` — violates JSON-RPC 2.0).
- Fixed both first-level fallbacks to use `"internal.fallback"` as the sentinel method name, matching the ultimate fallback.
- Simplified `build_notification` from 3 levels to 2 (now only: struct serialization → stringified safe JSON), since two consecutive serialization fallbacks were redundant.

## Resolved (R8)

### 10. R7 fix still incomplete — `build_notification`/`build_notification_value` happy path emits invalid JSON-RPC with empty method — RESOLVED (R8)
- R7 #9 only guarded the *fallback paths* against empty/whitespace method strings. If the method is empty and serialization *succeeds* (the happy path), the functions still produce `{"method":""}` — violating JSON-RPC 2.0 (method MUST be non-empty).
- Added explicit method validation at the top of both `build_notification` and `build_notification_value`: if `method.trim().is_empty()`, substitute `"internal.fallback"` before constructing the notification. This closes the final gap in the R6→R7→R8 chain.

## Resolved (R9)

### 11. `JsonRpcMessage::deserialize` misclassifies notification with explicit `"id": null` — RESOLVED (R9)
- JSON-RPC 2.0 allows `id` to be `null` (explicit null, not absent). The classifier used `value.get("id").is_some()` which returns `true` for `"id": null` (key exists), routing `{"jsonrpc":"2.0","id":null,"method":"test"}` to Request instead of Notification.
- Since `"id": null` semantically means "no response expected" (notification behavior), this misclassification breaks downstream handling.
- Fixed `has_id` check to use `value.get("id").map(|v| !v.is_null()).unwrap_or(false)` — treats `null` id as "no id".
- Added test `message_classifies_explicit_null_id_as_notification` to guard against regression.

## Resolved (R10)

### 12. R9 fix rejects spec-mandated `"id": null` responses — RESOLVED (R10)
- R9 made `has_id` return `false` for `"id": null`, which is correct for *request/notification* discrimination (null id = no id = notification). But the same `has_id` gated the *response* branch (`(has_result || has_error) && has_id`), so a Response carrying `"id": null` was rejected as "cannot classify".
- JSON-RPC 2.0 §5.1 mandates `"id": null` on error responses to requests whose id could not be detected (Parse error / Invalid Request). Rejecting them is a protocol bug — relevant for external MCP-server interop, where a malformed inbound request yields exactly such a response.
- Orthogonal to R9: R9 governs method-message routing; responses are identified by `result`/`error`. Dropped `&& has_id` from the response condition (the request branch still requires a non-null id, so R9's notification routing is unchanged).
- Added test `message_classifies_null_id_error_response`; all prior JSON-RPC tests (R9 notification, ambiguous-payload priority, unclassifiable rejection) still pass.

## Resolved (R11)

### 13. Vision / MediaPipe aspirational stub types removed — RESOLVED (R11)
- evernight 审计报告 Medium #4 指出 `model.rs` 中 `ModelCategory::Vision` 与 `ModelServerKind::MediaPipe` 被标注为 stub，属于 aspirational 占位类型。
- 审计结论：两者均为「全息 AR / MediaPipe 姿态识别」这一未实现特性预留，无任何实际实现——evernight（唯一部署方）自身的 `ModelServerKind` 仅含 `Ollama`/`WhisperCpp`/`Vllm`，并无 `MediaPipe`；entelecheia / shittim-chest / evernight 均未导入 `arona::model` 的类型（消费方各自持有镜像类型，或仅在 shittim-chest 的 `mock-mode` 开发桩里以裸 JSON 字符串形式出现）。
- 已实现的「视觉」能力（MediaFlow 视觉评审，glm-5v-turbo）由 `ModelCategory::MultiModal` + `ModelCapability::{ImageInput, VideoInput, VisualCritique}` 表达，不在本次清理范围内。
- 处理：从公开 API 中移除 `ModelCategory::Vision` 与 `ModelServerKind::MediaPipe` 两个枚举变体，并同步更新模块级文档表格与 `bindings/model.ts`（`cargo test` 自动再生成）。沿用 R4「移除 Agent 枚举 aspirational 变体」的既定惯例。
- 验证：`cargo check --all-features`、`cargo clippy --all-features -- -D warnings`、`cargo test --all-features`（651 项通过）均通过。
- 关联：evernight 仓库 PLAN.md Medium #4（本仓库内完成，evernight 侧无需改动）。


---

## 维护记录（2026-07-10）

### 待办：许可证元数据不一致（需维护者决策）

仓库根目录的 `LICENSE` 文件是 **SySL 1.0（Synthetic Source License）**，但工作区 `Cargo.toml` 声明的 SPDX 许可证是 **BUSL-1.1**，并通过 `license.workspace = true` 传播到所有子包。这意味着 crates.io 元数据宣传 BUSL-1.1，而实际许可证文本是 SySL —— 对依赖 SPDX 表达的下游用户构成真实冲突。

#### 为什么没有自动修复

SySL 1.0 不是标准 SPDX 标识符，crates.io 不接受 `license = "SySL-1.0"`。生态内的兄弟 crate（如 `hikari`）使用 `license-file = "LICENSE"` 来发布 SySL 许可的 crate。将 arona 改为该形式会改变已发布的许可证元数据，可能需要对发布的 crate 做新的 semver 版本提升，因此留给维护者决定。

#### 建议的解决方案（二选一）

1. **crate 确实是 BUSL-1.1**：用 BUSL-1.1 文本替换根 `LICENSE`，并在需要处单独保留 SySL 文本，同时在 README 添加许可证徽章。
2. **crate 是 SySL**：将 `Cargo.toml` 改为 `license-file = "LICENSE"`（移除 `license = "BUSL-1.1"` 行）并提升 crate 版本，与 `hikari` 保持一致。

### 本次维护已完成

- 修正 README 中文档链接文本（原指向 docs.celestia.world/en/arona，实际位于 guides/platforms）。
