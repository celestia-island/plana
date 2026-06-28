+++
title = "TIA Portal 前置准备 —— 接入 evernight"
description = """如何在 TIA Portal 中对 S7-1200/1500 PLC 做一次性配置，使 evernight 能够连接、自组网、读写设备，且此后无需任何人工干预。"""
lang = "zhs"
category = "guides"
subcategory = "router"
+++

# TIA Portal 前置准备 —— 接入 evernight

> **目标**：在 TIA Portal 中对西门子 S7-1200/1500 PLC 做**一次性**配置，使
> evernight 能够连接、自组网、读写设备，且**此后无需任何人工干预**。这只是一
> 次 CPU 属性配置 —— 你永远不必改动梯形图/SCL 程序逻辑。

evernight 通过**两条通道**连西门子 PLC，根据你的 PLC 暴露情况二选一：

| 通道 | 端口 | 访问方式 | 需要 TIA 准备 | 适用场景 |
|------|------|----------|--------------|----------|
| **S7comm** | 102 | M / I / Q / DB 的裸字节读写 | PUT/GET + 非优化 DB | 老系统、精简、无 OPC UA 授权 |
| **OPC UA** | 4840 | 符号化、自描述 | 启用内置 server | **推荐** —— 自动发现、优化 DB 也可访问 |

如果能开 OPC UA，优先用它：evernight 会**自动浏览**整棵符号地址空间，零人工录
符号。

---

## 路径 A —— S7comm（裸寄存器访问）

### A.1 开启 PUT/GET 通信

S7-1200/1500 默认禁止外部 S7 读写。

1. 在 **TIA Portal** 打开工程。
2. 在设备/网络视图里**点选 CPU**。
3. 属性 → **保护与安全（Protection & Security）→ 连接机制（Connection mechanisms）**。
4. 勾选 **"允许来自远程对象的 PUT/GET 通信（Permit access with PUT/GET communication from remote partner）"**。
5. 将硬件配置下载到 CPU。

### A.2 把目标 DB 改为非优化

优化的块访问（S7-1200/1500 默认）没有固定字节偏移，绝对地址读会失败。对每个
evernight 要读写的 DB：

1. 右键 DB → **属性（Properties）**。
2. 取消勾选 **"优化的块访问（Optimized block access）"**。
3. 重新编译并下载。

> M 区标志位和 I/Q 过程映像始终是字节寻址的，无需改动。这一步只针对 DB。

### A.3 用 evernight 连接

```
s7://192.168.1.10:102?rack=0&slot=1
```

- S7-1200/1500：`rack=0, slot=1`
- S7-300：`slot=2`

代码里自组网：

```rust
use evernight::protocol::auto_provision;

let profile = auto_provision("192.168.1.10").await?;
// profile.data_blocks / profile.db_structures 现在描述了每个可读 DB
```

---

## 路径 B —— OPC UA（推荐）

### B.1 固件与授权前提

- CPU 固件 **V2.0+**（OPC UA 方法需 V2.5+）。
- 与 CPU 档位匹配的 **SIMATIC OPC UA 运行系统许可证**（在 CPU 属性 →
  **运行系统许可证（Runtime licenses）→ OPC UA** 处分配，合规所需）。

### B.2 启用 OPC UA server

1. 在网络/设备视图里**点选 CPU**。
2. 属性 → **OPC UA → 常规（General）**：填一个 server 名称。
3. 属性 → **OPC UA → 服务器（Server）**：勾选 **"激活 OPC UA 服务器（Activate OPC UA server）"**。
4. 把 server 分配给客户端要访问的 **PROFINET 接口**。

### B.3 暴露符号变量

- **OPC UA → 服务器 → 服务器接口（Server interface）**：选 **"标准 SIMATIC 服
  务器接口（Standard SIMATIC server interface）"**，这样每个符号变量/DB（含优
  化 DB）都会自动发布。（自定义接口可手工挑选变量。）

### B.4 认证与安全

- **OPC UA → 服务器 → 用户认证（User authentication）**：匿名（受信任局域网）
  或用户名/密码。
- **OPC UA → 服务器 → 安全（Security）**：选策略。首次连接用 `None` 最简单；
  生产用 `Sign & Encrypt`。
- 在固件 **V3.1+ 且 TIA V19+** 上，把 **"OPC UA 服务器访问（OPC UA server
  access）"** 功能角色/运行权限授予连接用户。

### B.5 信任客户端证书

OPC UA 客户端连接时出示 X.509 证书；PLC 会隔离未知证书。evernight 首次连接后：

1. TIA Portal → **CPU → 证书（Certificates）**（在线），**或**
2. PLC **Web 服务器** → "与 OPC UA 客户端的通信"，**或**
3. CPU 显示屏的证书管理器。

然后**接受/信任** evernight 的客户端证书。

### B.6（可选）导出 OPC UA NodeSet XML

NodeSet 文件是所有变量的离线地图，便于无实时连接时预规划：

1. CPU 属性 → **OPC UA → 服务器 → 导出（Export）**。
2. 点 **"导出 OPC UA XML 文件（Export OPC UA XML file）"**，保存
   `*.Opc.Ua.NodeSet2.xml`。

### B.7 下载

下载**硬件配置**。这些是 CPU 属性，不是程序逻辑 —— 你的梯形图/SCL 代码原封不动。

### B.8 用 evernight 连接

端点 URL：

```
opc.tcp://192.168.1.10:4840
```

evernight 作为 OPC UA 客户端连上，**浏览**整棵符号树，按名字读写 —— 零人工录
符号，优化 DB 照样可访问。

---

## 验证连通性（零风险探测）

驱动输出前，用只读探测确认通道存活：

```bash
# 102 端口是否在讲 S7comm？
evernight probe 192.168.1.10 --ports 102

# 4840 端口 OPC UA server 是否起来？
evernight probe 192.168.1.10 --ports 4840
```

二者都是被动握手 —— 不读不写任何东西。

---

## 安全边界

- **绝不要**把安全联锁（急停、限位、过载）放到 S7/OPC UA 链路上。让它们留在
  PLC 扫描里。网络一断绝不能让安全功能失效。
- evernight 控制适合**慢速**负载（阀门、状态机、模式切换）。S7/OPC UA 往返延
  迟约 10–50 ms —— 监督级够用，运动/伺服太慢。
- 优先写**命令 M 位**，由现有 PLC 逻辑去执行（你抢占触发源），而不是直接写 Q
  输出。

---

## 故障排查

| 现象 | 可能原因 | 处理 |
|------|----------|------|
| S7 连接被拒 / 无 COTP 确认 | PUT/GET 未开；rack/slot 错；防火墙 | A.1；确认 `rack=0 slot=1`（1200/1500） |
| DB 读返回 "optimized" / InvalidAddress | 优化块访问开着 | A.2 —— 取消优化访问，重新编译 |
| OPC UA 端点不可达 | server 未激活；未下载；缺授权 | B.2 / B.7 / B.1 |
| OPC UA 连上后又被拒 | 客户端证书未信任 | B.5 |
| 浏览返回空 | 标准 SIMATIC 接口未启用 | B.3 |

---

## 参考

- [启用 OPC UA server（S7-1500）— STEP 7 V20 文档](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/enabling-the-opc-ua-server-s7-1500-s7-1500t)
- [访问 OPC UA server（端点 URL / 4840 端口）](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/access-to-the-opc-ua-server-s7-1500-s7-1500t)
- [导出 OPC UA XML 文件（NodeSet）](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/accessing-opc-ua-server-data-s7-1500-s7-1500t/export-opc-ua-xml-file-s7-1500-s7-1500t)
