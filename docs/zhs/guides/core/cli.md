+++
title = "CLI 使用指南"
description = """`entelecheia-cli` 是 Entelecheia（玄枢）多智能体协作平台的命令行界面。它通过 Unix socket JSON-RPC 与 scepter 服务器通信，提供聊天交互、服务生命周期管理、智能体控制、配置等功能。"""
lang = "zhs"
category = "guides"
subcategory = "core"
+++

# CLI 使用指南

`entelecheia-cli` 是 Entelecheia（玄枢）多智能体协作平台的命令行界面。它通过 Unix socket JSON-RPC 与 scepter 服务器通信，提供聊天交互、服务生命周期管理、智能体控制、配置等功能。

> 说明：CLI 目前尚未达到与 TUI 完全同等的功能。当前状态请参见 [ARCHITECTURE.md](../../ARCHITECTURE.md)。

---

## 目录

- [安装](#安装)
- [基本用法](#基本用法)
- [全局选项](#全局选项)
- [聊天命令](#聊天命令)
- [智能体管理](#智能体管理)
- [服务生命周期](#服务生命周期)
- [配置](#配置)
- [连接上下文](#连接上下文)
- [状态与监控](#状态与监控)
- [订阅（Layer3）](#订阅layer3)
- [运行智能体](#运行智能体)
- [时间线](#时间线)
- [Docker 镜像](#docker-镜像)
- [高级用法](#高级用法)

---

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia

# 构建 CLI 二进制文件
cargo build --package entelecheia-cli

# 或使用 just
just cli
```

二进制文件位于 `target/debug/entelecheia-cli`（debug）或 `target/release/entelecheia-cli`（release）。

### 预构建二进制

预构建的二进制文件可从 [GitHub Releases](https://github.com/celestia-island/entelecheia/releases) 获取。下载适合您平台的压缩包，并将二进制文件放入 `PATH` 中。

---

## 基本用法

```bash
# 显示帮助
entelecheia-cli --help

# 通过技能链发送消息
entelecheia-cli send 解释一下这个项目的架构

# 通过管道发送消息
echo "总结这个文件" | entelecheia-cli send

# 检查系统状态
entelecheia-cli status
```

---

## 全局选项

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | 日志级别（trace、debug、info、warn、error） | `warn` |
| `-d, --daemon` | 后台派发命令后立即退出 | — |
| `-c, --clean` | 清理 Cosmos 容器和 socket 文件 | — |
| `-a, --auto-approve` | 自动批准操作（确保服务器正在运行） | — |
| `-t, --table` | 人类可读表格输出（ANSI 格式） | 默认 |
| `-j, --json` | JSON 输出（机器可读） | — |
| `-r, --raw` | 原始纯文本输出（无格式） | — |
| `--format <FORMAT>` | 输出格式（table、json、raw） | `table` |

输出格式选项：

- `table` — 人类可读的表格输出
- `json` — 机器可读的 JSON 输出

**示例：**

```bash
# 清理容器
entelecheia-cli --clean

# 以 JSON 格式获取状态
entelecheia-cli status --format json

# 调试模式发送消息
entelecheia-cli -l debug send "调试连接问题"

# 后台模式运行 agent（立即返回）
entelecheia-cli -d run my-agent --ci
```

---

## 聊天命令

`chat` 子命令管理与会话智能体系统的对话交互。

### 发送消息

```bash
entelecheia-cli chat send [OPTIONS]
```

| 选项 | 描述 |
| --- | --- |
| `-m, --message <MSG>` | 要发送的消息文本 |
| `--stdin` | 从标准输入读取消息 |
| `-f, --file <PATH>` | 从文件读取消息 |

每次只能使用一个输入源。

**示例：**

```bash
# 直接发送消息
entelecheia-cli chat send -m "你好，你能做什么？"

# 从标准输入
echo "分析 src/main.rs 中的代码" | entelecheia-cli chat send --stdin

# 从文件
entelecheia-cli chat send -f ./prompts/review.txt
```

`chat send` 命令将消息通过**技能链**——协调多个智能体的核心执行管道。执行过程中会通过旋转动画显示进度。

### 聊天历史

```bash
entelecheia-cli chat history [OPTIONS]
```

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `--conversation <ID>` | 按会话 ID 筛选 | — |
| `--agent <TYPE>` | 按智能体类型筛选 | — |
| `--role <ROLE>` | 按角色筛选（user/assistant/system） | — |
| `--from <ISO8601>` | 开始日期时间（ISO 8601） | — |
| `--to <ISO8601>` | 结束日期时间（ISO 8601） | — |
| `--limit <N>` | 返回的最大消息数 | `50` |
| `--offset <N>` | 分页偏移量 | `0` |

**示例：**

```bash
entelecheia-cli chat history --agent ApoRia --limit 20 --from 2026-05-01T00:00:00Z
```

### 最近消息

```bash
entelecheia-cli chat recent [OPTIONS]
```

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `--timeline <ID>` | 按时间线/会话 ID 筛选 | — |
| `--agent <TYPE>` | 按智能体类型筛选 | — |
| `--limit <N>` | 返回的最大消息数 | `20` |

---

## 智能体管理

管理智能体生命周期（列出、启动、停止、重启）。

```bash
entelecheia-cli agent <COMMAND>
```

### 命令

```bash
# 列出所有智能体及其状态
entelecheia-cli agent list

# 按类型启动智能体
entelecheia-cli agent start <AGENT_TYPE>

# 停止正在运行的智能体
entelecheia-cli agent stop <AGENT_TYPE>

# 重启智能体
entelecheia-cli agent restart <AGENT_TYPE>
```

**可用的智能体类型：** ApoRia、EleOs、EpieiKeia、Haplotes、HubRis、Kalos、NeiKos、OreXis、PhiLia、Polemos、SkeMma、SkoPeo。

> 说明：智能体作为库 crate 在 scepter 运行时内部运行，而非独立可执行文件。`agent start` 命令尝试生成一个与智能体名称匹配的二进制文件，这主要适用于智能体被编译为单独二进制文件的情况。实际使用中，智能体通过 scepter 服务器激活。

---

## 服务生命周期

使用 Docker 容器管理 Entelecheia（玄枢）服务栈。

### 初始化服务

```bash
entelecheia-cli init [OPTIONS]
```

设置完整的服务栈：PostgreSQL（含 pgvector）、Docker 注册表、scepter 服务器和 WebUI。创建所需的 Docker 网络并拉取/构建镜像。

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `--prefix <STR>` | 容器名称前缀 | `e-` |
| `--source-build` | 从源码构建镜像而非拉取 | `false` |
| `--webui-port <PORT>` | WebUI 端口 | `3424` |

**示例：**

```bash
entelecheia-cli init --prefix ent- --webui-port 8080
```

### 启动所有服务

```bash
entelecheia-cli serve [OPTIONS]
```

启动所有之前已初始化的容器。需要先执行 `init`。

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `--prefix <STR>` | 容器名称前缀 | `e-` |
| `--webui-port <PORT>` | WebUI 端口 | `3424` |

### 停止所有服务

```bash
entelecheia-cli stop [OPTIONS]
```

按顺序停止所有正在运行的容器：webui → scepter → registry → postgres。

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `--prefix <STR>` | 容器名称前缀 | `e-` |

### 仅启动 WebUI

```bash
entelecheia-cli webui [OPTIONS]
```

仅启动或创建 WebUI 容器。

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `--prefix <STR>` | 容器名称前缀 | `e-` |
| `--webui-port <PORT>` | WebUI 端口 | `3424` |

---

## 配置

查看和验证系统配置。

### 显示配置

```bash
entelecheia-cli config show
```

显示当前配置，包括：

- 数据库 URL 和连接设置
- ApoRia LLM 提供商配置（名称、模型、端点）
- WebSocket 绑定地址
- 日志级别

API 密钥在输出中被遮蔽（显示为 `***`）。

### 验证配置

```bash
entelecheia-cli config validate
```

执行验证检查：

- 数据库 URL 已设置
- 至少配置了一个具有完整设置的 ApoRia 提供商
- WebSocket 绑定地址已设置

返回通过/失败结果，并附带任何问题的详细信息。

**输出示例：**

```text
Validate Configuration:

Validating database configuration...
  [ OK ]  Database URL set

Validating ApoRia LLM configuration...
  [ OK ]  ApoRia providers configured

Validating WebSocket configuration...
  [ OK ]  WebSocket Bind Address set

[ OK ]  Configuration validation passed
```

---

## 连接上下文

`context` 子命令用于管理命名的连接配置文件，允许您在本地（Unix socket）和远程（WebSocket）scepter 服务器之间切换。其使用方式与 Docker 的 `docker context` 命令类似。

### 概念

一个**上下文**是一个命名的配置文件，记录了 CLI 如何连接 scepter 服务器：

- **local** — Unix socket 连接（默认，自动解析为 `/run/.../entelecheia-tui.sock`）
- **remote** — 带 Bearer token 认证的 WebSocket 连接

上下文存储在 `~/.config/entelecheia/contexts/contexts.toml` 中。

### 列出上下文

```bash
entelecheia-cli context list
```

当前活动的上下文以 `*` 标记。

### 显示当前上下文

```bash
entelecheia-cli context show
```

显示活动上下文的类型、socket 路径、WS URL 和描述信息。

### 创建上下文

```bash
# 远程 WebSocket 上下文
entelecheia-cli context create staging \
  --ws-url ws://scepter.example.com:8424/ws \
  --bearer-token <TOKEN> \
  --description "Staging server"

# 额外的本地上下文
entelecheia-cli context create dev --description "Development server"
```

从远程服务器获取 Bearer token：

```bash
# 在服务器机器上
docker exec e-scepter cat /home/entelecheia/.config/entelecheia/scepter.token
```

### 切换上下文

```bash
entelecheia-cli context use staging
# 此后所有命令（send、status、chat 等）都将通过 staging 连接路由
```

### 移除上下文

```bash
entelecheia-cli context remove staging
```

`default` 上下文不可被移除。

### 示例工作流

```bash
# 查看当前上下文
entelecheia-cli context list

# 为预发布服务器创建远程上下文
entelecheia-cli context create staging \
  --ws-url ws://192.168.1.100:8424/ws \
  --bearer-token $(cat /path/to/token)

# 切换到预发布环境
entelecheia-cli context use staging

# 通过远程服务器发送消息
entelecheia-cli send "列出当前待办事项"

# 检查远程服务器状态
entelecheia-cli status

# 切换回本地
entelecheia-cli context use default
```

---

## 状态与监控

### 系统状态

```bash
entelecheia-cli status
```

显示：

- 服务器版本
- 连接状态（socket 状态）
- LLM 提供商摘要
- WebSocket 绑定地址
- 智能体列表及运行/停止状态
- 系统资源（内存使用量、平均负载）

### 状态路径查询

`status` 命令接受类路径参数来查询特定子系统。语法支持按 agent 范围的时间线、聊天历史检查和设备枚举。

```bash
entelecheia-cli status <PATH> [--raw]
```

| 路径语法 | 描述 |
| --- | --- |
| `timeline.#agent[-N]` | 显示某 agent 最近 N 次 skill 调用记录 |
| `timeline.#agent[N][M]` | 显示第 N 次 skill 调用中的第 M 个 MCP/工具调用 |
| `history[-N]` | 显示最近 N 条聊天消息（所有角色） |
| `history[-N].body` | 显示倒数第 N 条消息的正文 |
| `device` | 列出所有 Polemos 识别的边缘设备 |
| `device[N]` | 显示第 N 个 Polemos 设备的详细信息 |

**示例：**

```bash
# Haplotes #001 agent 最近 30 次 skill 调度历史
entelecheia-cli status timeline.#hap_lotes.001[-30]

# 第 3 次 skill 调用的第 2 个 MCP/工具调用
entelecheia-cli status timeline.#hap_lotes.001[3][2]

# 最近 30 条消息
entelecheia-cli status history[-30]

# 倒数第 3 条消息正文（纯文本）
entelecheia-cli status history[-3].body --raw

# 所有 Polemos 设备
entelecheia-cli status device

# 第 3 个 Polemos 设备详情
entelecheia-cli status device[3]
```

> **Shell 注意:** 在 bash/zsh 中，请用单引号包裹含 `[...]` 的路径以防 glob 展开：`entelecheia-cli status 'history[-30]'`。`#` 字符嵌在单词中间时无需转义。在 fish shell 中，以上路径均无需引号。

状态路径查询通过 Unix socket JSON-RPC 与服务器通信。`timeline.*` 和 `history.*` 查询需要服务器正在运行。`device` 查询需要服务器上有 Polemos 工作区注册。

### 查看日志

```bash
entelecheia-cli logs [OPTIONS]
```

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `-a, --agent <NAME>` | 按智能体名称筛选日志 | 所有智能体 |
| `-l, --lines <N>` | 显示的行数（尾部） | `100` |

**示例：**

```bash
# 显示所有智能体日志的最后 200 行
entelecheia-cli logs -l 200

# 显示 ApoRia 日志
entelecheia-cli logs -a ApoRia
```

日志从 `./logs/` 目录读取。每个智能体都有各自的日志文件（`ApoRia.log`、`EleOs.log` 等）。

---

## 订阅（Layer3）

管理 Layer3 智能体订阅——可以安装和运行的外部智能体包。

### 列出订阅

```bash
entelecheia-cli subscribe list
```

显示所有已配置的订阅，包括状态（已安装/待处理）、启用状态、自动更新设置和来源。

### 添加订阅

```bash
entelecheia-cli subscribe add [OPTIONS]
```

| 选项 | 描述 |
| --- | --- |
| `--name <NAME>` | 订阅名称（必需） |
| `--source <SOURCE>` | 来源类型：`official`、`github` 或 `url`（必需） |
| `--repository <REPO>` | GitHub 仓库（用于 github 来源） |
| `--url <URL>` | 直接 URL（用于 url 来源） |
| `--version <VER>` | 版本约束 |
| `--auto-update` | 启用自动更新 |
| `--disabled` | 添加为禁用状态 |

**示例：**

```bash
entelecheia-cli subscribe add --name my-agent --source github --repository user/repo
```

### 移除订阅

```bash
entelecheia-cli subscribe remove <NAME>
```

### 同步订阅

```bash
# 同步所有订阅
entelecheia-cli subscribe sync

# 同步特定订阅
entelecheia-cli subscribe sync --name my-agent
```

### 自动更新

```bash
entelecheia-cli subscribe auto-update
```

更新所有启用了 `auto_update` 的订阅。

---

## 运行智能体

```bash
entelecheia-cli run <AGENT> [OPTIONS]
```

运行 Layer3 智能体脚本。在当前目录中查找 `.amphoreus/<AGENT>/run.py`。首次执行时会运行预检审计。

| 选项 | 描述 |
| --- | --- |
| `--ci` | 启用 CI 模式 |
| `--auto-pr` | 启用自动 PR 模式 |
| `--dry-run` | 试运行（不进行实际更改） |
| `--providers <LIST>` | 逗号分隔的提供商列表 |
| `--output-dir <DIR>` | 输出目录 |

**示例：**

```bash
# 以试运行模式运行 Layer3 智能体
entelecheia-cli run my-agent --dry-run

# 使用指定提供商运行
entelecheia-cli run my-agent --providers openai,anthropic

# CI 模式并自动提交 PR
entelecheia-cli run my-agent --ci --auto-pr

# 后台模式运行（立即返回，子进程后台执行）
entelecheia-cli -d run my-agent --ci --auto-pr
```

### 后台模式（`-d` / `--daemon`）

后台模式标志会使 CLI 以剥离 `--daemon` 参数的方式重新生成一个分离的子进程，并立即返回。子进程继承原始命令并独立运行。之后可使用 `status` 查看进度。

适用于 `run`、`init`、`deploy` 等长时间运行的操作：

```bash
# 后台派发 agent 运行
entelecheia-cli -d run my-agent

# 后台派发服务初始化
entelecheia-cli -d init --prefix prod-

# 稍后查看状态
entelecheia-cli status
entelecheia-cli status history[-5]
```

---

## 时间线

查看会话时间线。

### 列出时间线

```bash
entelecheia-cli timeline list [OPTIONS]
```

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `--agent <TYPE>` | 按智能体类型筛选 | — |
| `--limit <N>` | 最大结果数 | `50` |
| `--offset <N>` | 分页偏移量 | `0` |

### 显示时间线详情

```bash
entelecheia-cli timeline show <CONVERSATION_ID> [OPTIONS]
```

| 选项 | 描述 | 默认值 |
| --- | --- | --- |
| `--include-messages` | 输出中包含消息 | `true` |

---

## Docker 镜像

```bash
entelecheia-cli init-docker-images [OPTIONS]
```

构建或拉取平台所需的 Docker 镜像。

| 选项 | 描述 |
| --- | --- |
| `--source-build` | 从源码构建镜像而非拉取 |
| `--tag <TAG>` | 镜像标签（默认：`latest`） |

**示例：**

```bash
# 从源码构建所有镜像
entelecheia-cli init-docker-images --source-build

# 使用自定义标签拉取
entelecheia-cli init-docker-images --tag v0.2.0
```

管理的镜像：

- `entelecheia` — 编排服务器（含内嵌 cosmos 运行时）
- `pgvector/pgvector` — 带向量扩展的 PostgreSQL

---

## 高级用法

### 用于脚本的 JSON 输出

使用 `--format json` 获取机器可读的输出，可管道传输至 `jq` 或其他工具：

```bash
entelecheia-cli status --format json | jq '.server_version'
entelecheia-cli chat history --format json | jq '.messages[].content'
```

### 链式清理与初始化

```bash
# 完全拆除并重建
entelecheia-cli --clean && entelecheia-cli init --prefix my-
```

### 调试模式

```bash
# 启用 trace 级别日志进行调试
entelecheia-cli -l trace send "测试消息"
```

### 与 TUI 搭配使用

CLI 与 TUI 连接到同一个 scepter 服务器。两者可以同时使用：

- 启动 TUI 进行交互式会话：`cargo run --bin entelecheia-tui`
- 使用 CLI 进行脚本编写、自动化和快速查询

---

## 故障排除

### "No command specified"

运行 `--help` 查看可用命令，或使用 `send "消息"` 快速发送消息。

### "Failed to connect to Docker"

确保 Docker（或 Podman）正在运行：

```bash
docker info
docker run hello-world
```

### "Agent binary not found"

智能体是 scepter 运行时的内部库 crate，而非独立二进制文件。启动 scepter 服务器以激活智能体：

```bash
entelecheia-cli init && entelecheia-cli serve
```

### "No LLM providers configured"

通过环境变量设置 ApoRia 提供商配置。有关提供商设置说明，请参见[构建指南](building.md)。

### "Configuration validation failed"

运行 `entelecheia-cli config validate` 查看哪些检查失败。常见问题：

- 缺少 `DATABASE_URL` 环境变量
- ApoRia 提供商设置不完整（名称、模型、`api_key`）
- 缺少 WebSocket 绑定地址
