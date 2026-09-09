# PLANA 中继扩展——服务档案之上的链式通信

状态：**生效**（核心实现于 `plana-rpc-client::relay` 与 npm 包
`@celestia-island/plana-rpc-client`；Tauri 适配与标准指令电池在
`plana-tauri`。英文权威版：[relay-profile.md](../../en/rpc/relay-profile.md)）

本文档是对[服务档案](service-profile.md)的**扩展**，绝非替代。服务档案
定义的是"一跳"：客户端直连服务端。中继扩展定义的是当通信双方必须
**经由**对方时怎么办：webview 经由宿主进程、宿主经由网关、网关经由下
一个中继。链可以任意长；本档案就是每一跳都要实现的那个单跳契约。

## 1. 拓扑

链 A₀ → A₁ → … → Aₙ（n ≥ 2）。角色按位置定义：

- **边缘端**（A₀）：UI/客户端——webview、egui 界面、CLI。只说纯
  JSON-RPC，且**如同直连一般**寻址远端（反向代理错觉）。
- **中继**（任意中间 Aᵢ）：宿主进程——Tauri 后台、egui 宿主、daemon、
  网关。每个中继独立路由每一帧：本地 / 转发 / 拒绝。
- **上游**（Aₙ）：一个服务档案服务端——**或另一个中继**。正是这一替
  代让四端、五端乃至 N 端成立：扩展只标准化"跳"，链由跳自然组合。

## 2. 两个通道

**边缘通道**（任意相邻两方）：恰有两个传输函数——
`bridge_call(method, params, id?, relay?)`（请求-响应）与唯一的通知
通道 `bridge_event {method, params}`。帧为规范 JSON-RPC 2.0
（`plana-jsonrpc`），边缘与上游形状一致，一帧从头到尾可检查。通道刻意
与传输无关：Tauri invoke/事件、egui 通道、stdio、进程内直连都只是同
一契约的适配器（Rust 核心 `EdgeTransport`，JS 侧同名接口）。

**上游通道**（中继 → 服务端）：服务档案原文——WS 为主 + HTTP 回退、
心跳、关闭码。中继扩展不新增任何服务端语法。

## 3. 路由——每跳三态

入站帧按方法名的前导点分命名空间路由：

1. **本地**——`relay` 命名空间（保留）或应用自挂的本地命名空间（如
   `flasher.*` 特权磁盘 I/O，永不上转发路径）。
2. **转发**——**最长前缀白名单**命中，把命名空间映射到经
   `relay.conn.open` 打开的命名端点；同前缀以最后一次编辑为准。
3. **拒绝**——未列出的命名空间一律 `-32601`，中继**绝不猜**端点。

两者合起来即扩展之名的由来：反向代理（边缘原样看到服务端的方法表
面）+ 正向代理（中继按自身白名单与代理策略出站）。

## 4. 保留的 `relay.*` 命名空间

结构性不可转发：路由表拒绝一切以 `relay` 为首段的条目，因此无论配置
如何被改写，系统控制方法都拉不上服务器连接。预标准化表面（按需挂
载，后挂覆盖先挂）：

| 方法 | 用途 |
|---|---|
| `relay.net.set_proxy` | 出站代理——`scheme`（`http`/`https`/`socks5`）、`host`、可选 `username`/`password`；空 host = 显式直连。 |
| `relay.net.configure_entrypoint` | 标准登录握手入口 `{url}`——公开前门；内层转发是服务端的事。 |
| `relay.conn.open` | 打开/复用到某端点的 WebSocket 或 HTTP 长轮询连接；返回转发映射所用的句柄。 |
| `relay.fwd.map` | 增加一条白名单 `{namespacePrefix, endpoint}`。 |
| `relay.fwd.list` | 列出端点与白名单。 |
| `relay.window.minimize` / `relay.window.maximize_toggle` / `relay.window.close` | 宿主窗口控制（任意宿主框架可实现）。 |
| `relay.window.state` | 窗口标签与最大化态，供界面外框同步。 |

这也解释了应用首启界面为何只有"入口 + 代理"两项：其余一切连接事实
都由这两个原生物派生。

## 5. 多跳信封

- 端到端关联：JSON-RPC `id` 原样穿透所有跳。
- 防环：扩展成员 `relay: {hops, via?}` 计数跳数；每个转发的中继将其加
  一，并对已达上限（默认 4）的帧报错拒转。`via` 仅作诊断轨迹，绝不承
  载逻辑。
- 没有任何一跳知道自己在链上的绝对位置——每跳套用同一契约。

## 6. 代理策略

每跳一个决策：中继设置覆盖 → `PLANA_PROXY` / `PLANA_PROXY_SCHEME` /
`PLANA_PROXY_USERNAME` / `PLANA_PROXY_PASSWORD` 环境族 → **直连**。
ambient 的 `HTTP(S)_PROXY` 永不生效：流量策略是显式配置。该跳上所有拨
号器（WS 与 HTTP 一致）必须消费同一决策。

## 7. 包落点

- 协议核心（框架无关）：crates.io 的 `plana-rpc-client::relay` 与
  npm `@celestia-island/plana-rpc-client` 的对应模块——随两包既有发
  布管线走。
- Tauri 适配 + 标准电池（`bridge_call` 命令、事件通道、含窗口控制在
  内的预挂载处理器）：`plana-tauri`（crates.io；默认 `bridge`
  feature，纯电池宿主可关）。
- Tauri 之外的框架适配器（egui、stdio）随各自宿主实现 `EdgeTransport`
  与电池钩子。

## 8. 一致性要点

- 中继必须对每个请求恰好应答一次（id 关联）。
- 中继必须拒绝而非猜测（不可路由帧回 `-32601`）。
- 任何配置下中继都不得转发 `relay.*` 方法。
- 转发中继必须执行跳数上限。
- 边缘信封可携带 `relay` 扩展成员；服务档案的服务端忽略未知成员，上
  游的严格性不受影响。
