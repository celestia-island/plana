# PLANA 服务档案——严格 WebSocket JSON-RPC 2.0

状态：**草案**（由 `plana-rpc-server` 实现，一致性测试套件位于
`packages/rpc-server/tests/conformance.rs`。英文权威版：
[service-profile.md](../../en/rpc/service-profile.md)）

本文档定义 celestia 托管服务对外暴露的线上协议档案，第三方可自行重新
实现。协议与 [`plana-rpc-server`] 框架均为开放实现；托管实例（如
`gateway.celestia.world`）是 celestia 自己的部署。

本档案是 JSON-RPC 2.0 的一个**窄方言**：客户端能做的一切就是请求/响应
调用与心跳通知。没有客户端发起的通知处理器、没有批量输入、没有服务端
主动发起的流（已建立连接上由 handler 推送的通知除外）。

## 1. 信封

- 信封为 `plana::jsonrpc`（`packages/jsonrpc/src/types.rs`）规范定义的
  JSON-RPC 2.0——全舰队唯一的规范定义。每帧必须携带
  `"jsonrpc": "2.0"`。
- 每个 WebSocket **text** 帧一个 JSON 对象。二进制帧是协议违规（关闭码
  `1003`）。
- 请求 `id`：对服务端不透明；字符串形式的 UUID v7 是客户端规范格式
  （时间有序、字典序可排）。数字 id 是合法 JSON-RPC，必须原样回显。
- 带 `id` 的请求必须恰好被应答一次，响应回显该 id（成功携带
  `result`；失败携带 `error` 且无 `result`）。
- 对无法解析的帧，错误响应携带 `"id": null`（JSON-RPC 2.0 强制）。

## 2. 传输

| 传输 | 地址 | 说明 |
|---|---|---|
| WebSocket | `GET {path}` 升级 | 主传输；每连接一个逻辑会话 |
| HTTP POST | 同 `{path}` | 退化环境回退（对应 `@celestia-island/plana-rpc-client`）；单请求对象进、单响应对象出；handler 通知被丢弃 |

POST 回退的 HTTP 状态映射：一切已派发结果（含 JSON-RPC 应用错误）均
`200`；无法解析的请求体或非请求对象 `400`；连接认证钩子拒绝时 `401`；
连接数达上限时 `429`。

## 3. 允许的 HTTP 面（其余一切都是 RPC）

仅以下三类豁免于"仅 RPC"规则：

1. **探针**——`GET /api/health`（引擎健康体）、healthz/readyz/ping、网
   络描述。负载均衡器与 malkuth 信息落地页说的是 HTTP，不是 RPC。
2. **凭据转发登录**——OAuth `authorize`/`callback` 浏览器重定向（HTTP
   302 流无法骑在浏览器尚未打开的 WS 上）。回调交给 SPA 一次性票据；
   SPA 打开 socket 并经 RPC（如 `rescue.open_session`）铸造会话。凭据
   除单次票据外绝不出现在 JS 可读的响应体或 URL 中。
3. **特殊协议**——MQTT 等非 HTTP 监听器占用独立端口/路径属显式部署决
   策，不在本档案范围内。

## 4. 方法命名

服务 RPC 使用 `lowercase.dotted` 命名空间（`rescue.open_session`、
`device.register`、`enrollment.mint`）。`Sync.*` / `Base.*` PascalCase
目录属于 entelecheia 工作区同步方言，不用于新服务表面。唯一保留的通知
名是 `Base.Heartbeat` / `Base.HeartbeatAck`（为
`plana-rpc-client` 默认协议的客户端兼容性而保留）。

## 5. 心跳与活性

- 客户端按自身节拍（plana-rpc-client 默认 15s）发送
  `{"jsonrpc":"2.0","method":"Base.Heartbeat"}`（无 id）。
- 服务端在**控制车道**上应答
  `{"jsonrpc":"2.0","method":"Base.HeartbeatAck"}`——一条能越过任何响
  应积压的优先队列，因此饱和的数据车道不会被误读为断连。
- 任何入站帧都会重置空闲计时器。空闲窗口（默认 45s ≈ 3× 节拍）内无
  入站帧 ⇒ 服务端以 `4000` 关闭。

## 6. 关闭码

| 码 | 含义 |
|---|---|
| 1000 | 正常关闭 |
| 1003 | 纯文本档案上的二进制帧 |
| 1008 | 连接级策略违规 |
| 1011 | 服务端意外故障 |
| 4000 | 空闲/心跳超时 |

## 7. 错误码

标准 JSON-RPC（`-32700/-32600/-32601/-32602/-32603`）与 plana 舰队扩
展码（`plana::jsonrpc::error_codes`，特别是 `-32005` AUTH_ERROR）。本档
案新增：

| 码 | 含义 |
|---|---|
| `-32050` | 连接数达上限（HTTP 429 响应体） |
| `-32051` | 派发超时被取消 |

应用错误**应当**在 `error.data.code` 中携带稳定的机器可读字符串（如
`"quota_exhausted"`），与人类可读消息并存。

## 8. 限制、超时与取消

- 最大并发连接：默认 100（与 scepter 对齐）；超额升级在握手前以
  HTTP 429 + `-32050` 拒绝。
- 帧/消息预算：默认 1 MiB / 4 MiB。
- 运行超过超时上限（默认 8s，低于客户端 10s 应答窗口）的 handler 会被
  **取消**并以 `-32051` + `data.stalled=true` 应答。迟到的结果绝不落
  到线上。⇒ **handler 必须可安全取消。**
- 批量输入（JSON 数组）以 `-32600` + `data.reason="batch_not_supported"`
  拒绝（v1）。
- 客户端发来的 `Response` 帧被忽略并告警。

## 9. 认证模型

三层可组合，均可按部署选配：

1. **连接认证**（升级时，拒绝则 HTTP 401）：钩子看到原始
   headers/URI——bearer token、一次性票据、mTLS 身份。产出每连接一份
   不透明上下文。
2. **请求守卫**（派发前，拒绝则 JSON-RPC 错误）：基于连接上下文的方法
   级授权。舰队约定：认证拒绝用 `-32005`。
3. **连接绑定会话**（强交互服务推荐）：会话句柄存于服务端内存、以连
   接为键，绝不序列化给客户端——没有可泄漏的 bearer token，URL 里也没
   有 token。重连 ⇒ 重新认证。长期凭据属于第 1 层，不属于会话句柄。

## 10. 一致性

声称实现本档案的实现须通过
`packages/rpc-server/tests/conformance.rs` 的套件（14 例）：含 UUIDv7
的 id 回显、`-32601`/`-32602` 映射、解析错误保持连接且 `id:null`、批量
拒绝、心跳应答（含控制车道越过繁忙数据车道）、超时取消、空闲关闭
`4000`、升级拒绝 401、守卫拒绝 `-32005` 且不派发、同方法映射表上的
HTTP POST 回退、handler 推送通知。

## 11. 参考实现

- **TypeScript 客户端**——`@celestia-island/plana-rpc-client`（npm）：
  分层传输 WS + HTTP 回退、可插拔心跳（默认 `Base.Heartbeat` 通知模
  式）、401 上的 token 刷新、指数退避重连。
- **Rust 客户端**——`plana-rpc-client`（本工作区）：持久 WS 连接与 id
  关联（UUIDv7）、逐调用超时、心跳看门狗、指数退避重连（含快速失败
  `Failed` 态）、通知订阅、一次性 HTTP POST 回退（`http::post_rpc`）。
- **Rust 服务端**——`plana-rpc-server`（本工作区）：支撑 §10 的框架与
  一致性套件。

## 12. 私有部署（受支持的部署形态）

本档案刻意与端点解耦：**任何实现都不得硬编码官方主机**。每个参考服务
的端点都来自配置，因此完全私有的舰队（纯内网工厂、气隙实验室、本地机
房）可以自行组合：

- 连接认证钩子后面自选的身份源（bearer token、一次性票据、mTLS——
  §9）；
- 任意内网地址上的 `plana-rpc-server`（或任何一致的第三方服务端），
  由纯 TCP 或内部反向代理前置；
- 以 URL 指向它的 `plana-rpc-client` /
  `@celestia-island/plana-rpc-client`——ws://、wss://、http(s)://、裸
  pod 地址都合规。

活参考：celestia 演示注册网关——一台内网 pod 上的 `evernight-gateway`
实例，其 chest-token 校验密钥与演示 chest 对齐，由 nginx 路径车道前
置；烧写器经 `?token=` 传送接入，与托管配置零代码差异。完整自托管配置
面见 `evernight/packages/gateway` 与
`evernight-appliance/packages/flasher/README.md`。
