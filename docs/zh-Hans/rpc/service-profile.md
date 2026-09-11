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
| `-32051` | **保留**：专用于「派发被取消」的码。当前实现以 `-32603` + `data.stalled=true` 应答（见 §8），客户端须按 `data.stalled` 判定，而非本码 |
| `-32052` | 延迟操作 id 未知：从未签发，或被服务端保留上限淘汰（因此本码可能在该 id 窗口**尚未**到期时出现） |
| `-32053` | 延迟操作 id 已签发但其有效期窗口已过 |
| `-32054` | 延迟操作注册表已达上限（pending 数满） |
| `-32055` | worker 接受 `ops.cancel` 并放弃了该操作（延迟结果错误码） |

应用错误**应当**在 `error.data.code` 中携带稳定的机器可读字符串（如
`"quota_exhausted"`），与人类可读消息并存。

## 8. 限制、超时与取消

- 最大并发连接：默认 100（与 scepter 对齐）；超额升级在握手前以
  HTTP 429 + `-32050` 拒绝。
- 帧/消息预算：默认 1 MiB / 4 MiB。
- 运行超过超时上限（默认 8s，低于客户端 10s 应答窗口）的 handler 会被
  **取消**并以结构化超时错误应答——当前为 `-32603` + `data.stalled=true`
  （上表的 `-32051` 是为专用码保留的，尚无分支发出，客户端须按
  `data.stalled` 判定）。迟到的结果绝不落到线上。⇒ **handler 必须可安全
  取消。**
- **派发超时上限是「以秒计的活性护栏」，绝不是上游预算。** 它的作用是
  掐掉卡死或失控的派发，而不是限定上游可以跑多久；上游慢也不能靠调大
  它来解决。任何上游可能超过它的 handler——支付/结算请求、供应商回调、
  LLM 调用，以及一切计量型或改状态的调用——**必须**立即以延迟操作引用
  应答（见 §8.1）。改状态的派发被超时取消时后果最严重：上游可能在取消
  **之后**才完成，钱或额度花了却没有结果返回调用方。
- 批量输入（JSON 数组）以 `-32600` + `data.reason="batch_not_supported"`
  拒绝（v1）。

### 8.1 延迟操作

本档案对「不属于自己的上游延迟」的答案：handler 立即应答，调用方稍后取回。

- **立即应答。** 支持延迟的 handler 应答
  `{"op_id": "<不透明随机 id>", "expires_in": <秒>}`。`op_id` 是随机
  UUID——不可猜测、绝非自增计数——因为任何持有它的人都能取回结果。
- **有效期窗口。** `expires_in` 是**自创建起算的剩余**有效期，客户端
  可见 id 的预期区间为 **10–30 分钟**（默认 30）。窗口锚定在创建时刻而
  非结算时刻，因此在窗口内重连的客户端仍能取回结果。
- **取回。** `ops.result {op_id}` → 结果对象
  `{"op_id", "status", "method", "expires_in", "cancel_requested",
  "result"?, "error"?}`，`status` 为 `pending` / `completed` / `failed` 之一；
  进入终态后 `result` / `error` 恰有其一。取回是**非破坏性**的：已结算的
  结果在窗口内可反复取回，因此丢一个响应帧不会让调用方损失结果。从未签发
  的 id 应答 `-32052`，被服务端保留上限提前淘汰的 id 同样应答 `-32052`；
  窗口已过的 id 应答 `-32053`（随后条目即被清除）。
- **取消。** `ops.cancel {op_id}` → `{"op_id", "status",
  "cancel_requested"}`。设计上就是尽力而为：它只记录一个 worker 可观察的
  标志（无需调度器），接受取消的 worker 以 `-32055` 结算该操作。对已结算
  的操作取消是合法应答，`cancel_requested: false`。
- **结算通知。** 当发起连接仍然在线时，数据通道推送
  `ops.settled {op_id, status}`。它是**建议性的，绝不是权威路径**：不携带
  载荷、入队不等待，连接不在或数据车道已满时静默丢弃（慢客户端绝不能把
  已结算该操作的 worker 卡住）。它与携带该 `op_id` 的响应之间**没有顺序保证**
  （立即结算的 worker 可能先于该响应到达），因此客户端必须忽略自己不认识的 id 的
  通知。因此客户端始终以 `ops.result` 兜底。
- **worker 失败也是一份应答。** 返回错误或 panic 的 worker 会把该操作结算为
  `failed`（`-32603`，消息中说明 panic）；延迟操作绝不会因为 worker 死掉
  而永远停在 `pending`。
- **有界性。** 服务端限制 pending 操作数量（达上限应答 `-32054`）并清理
  过期条目，因此注册表与断连客户端都不会无界增长。契约里还有两点必须说明：
  该上限约束的是**条目数而非字节数**（已结算的结果在被取回或过期前一直保留
  其载荷），且该上限是**服务端全局**的——需要按调用方公平性时用请求守卫。
  达到保留上限时会淘汰最旧的已结算条目，这是「未被取回的结果可能在窗口到期
  前消失」的唯一情形（此后该 id 应答 `-32052`）。

两个方法由框架自身提供（`plana-rpc-server` 的内置方法表），WS 与 HTTP
POST 两种传输都可用；服务可以在自己的方法表里覆盖同名方法，或通过请求
守卫拒绝。线格式即 `plana::jsonrpc::deferred` 中的类型；
`plana-rpc-client` 的 `RpcClient::await_op` 在收到 `ops.settled` 通知时
由通知唤醒，否则轮询 `ops.result`，以调用方给定的截止时间为界。
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

延迟操作契约（§8.1）由 `packages/rpc-server/tests/deferred.rs` 覆盖
（12 例：在远小于上游耗时的超时上限下立即应答、结算后可反复取回、从第二条
连接以及经 HTTP 传输取回、结构化 `-32052`/`-32053` 与 `-32602`、建议性
`ops.settled` 通知、过期清理、`-32054` 容量拒绝、`ops.cancel` 标志、服务
覆盖内置取回方法、worker panic、以及作为反向对照的超时守卫本身），外加
`RpcClient::await_op` 自身的用例套件（`packages/rpc-client/tests/deferred.rs`，
12 例，含跨重连取回、「失败结果作为值返回」、不可解析的 `ops.result` 报协议错误、
两个极端截止时间、零截止时间不发起读取、失效客户端快速失败、以及退化轮询节奏
不得变成忙轮询）。

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
