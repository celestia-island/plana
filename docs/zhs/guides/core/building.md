+++
title = "构建指南"
description = """- [先决条件](#先决条件)"""
lang = "zhs"
category = "guides"
subcategory = "core"
+++

# 构建指南

---

## 目录

- [先决条件](#先决条件)
- [安装](#安装)
- [配置](#配置)
- [构建](#构建)
- [运行](#运行)
- [数据库管理](#数据库管理)
- [开发环境](#开发环境)
- [部署](#部署)
- [故障排除](#故障排除)
- [运行 Webhook 机器人](#运行-webhook-机器人)

---

## 先决条件

### 系统要求

- **操作系统**: Linux、macOS 或 Windows（需要 Docker CLI）
- **内存**: 最低 8GB，推荐 16GB
- **存储**: 最低 20GB 可用空间
- **CPU**: 推荐 4 核心以上

> 说明（设计意图）
> Windows 侧的核心要求是 Docker CLI 可用，命令可以直接在 PowerShell 或 Windows Terminal 执行。
> 但容器最终仍需要 Linux 运行时来承载：
> 1. 本地方案通常是 Docker Desktop（一般依赖 WSL2 后端）。
> 2. 替代方案是本机仅安装 Docker CLI，并通过 `docker context` 转发到远程 Linux Docker 主机。

### 软件要求

#### 必需软件

- **Docker 或 Podman**（容器运行时环境）

```bash
docker --version
docker compose version
```

请按当前平台使用官方推荐安装方式：

- Linux：安装 Docker Engine、Docker Desktop for Linux，或发行版自带的 Podman
- macOS：安装 Docker Desktop 或 Podman Desktop
- Windows：安装 Docker Desktop 或 Podman Desktop

**重要说明**：

- PostgreSQL 等运行时依赖已包含在容器化环境中
- 但如果要运行 `just` 配方或仓库内辅助脚本，宿主机仍需要安装 Python 3.8+
- 无需在宿主机上单独安装 PostgreSQL
- Windows 下命令可直接在 PowerShell 或 Windows Terminal 中运行，但部署仍要求可用的 Docker/Podman Linux 运行时。本地部署通常意味着使用带 WSL2 后端的 Docker Desktop；也可通过本机 Docker CLI/context 转发到远程 Linux Docker 主机。

- **Rust 1.85+**（仅开发构建需要）

```bash
rustup update stable
```

请按平台使用官方 rustup 安装方式：

- Linux/macOS：访问 <https://rustup.rs>
- Windows：从 <https://rustup.rs> 下载并运行 `rustup-init.exe`，然后执行 `rustup update stable`

#### 推荐软件

- **just**（命令运行器）

```bash
  # 使用 cargo
  cargo install just

  # 使用 brew（macOS）
  brew install just
  ```

- **VS Code** 并安装 rust-analyzer 扩展

---

## 安装

### 步骤 1: 克隆仓库

```bash
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia
```

### 步骤 2: 配置环境变量

```bash
# 在从 .env.example 创建 .env 后编辑配置
nano .env  # 或使用您喜欢的编辑器
```

请使用当前 shell 或文件管理器把 `.env.example` 复制为 `.env`。

POSIX shell：

```bash
cp .env.example .env
```

PowerShell：

```powershell
Copy-Item .env.example .env
```

#### 基本配置

```bash
# 数据库配置（容器内部自动配置）
# DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
# DATABASE_MAX_CONNECTIONS=10

# LLM 快速初始化，启动后导入 ApoRia
# 单个 provider：
# LLM_API_KEY=your-api-key-here
# LLM_BASE_URL=https://api.openai.com/v1
# LLM_MODEL=gpt-4
# 多 provider（分号分隔）：
# LLM_API_KEY=key1;key2
# LLM_BASE_URL=https://api.one/v1;https://api.two/v1
# LLM_PROTOCOL=openai;openai,api-key
# LLM_MODEL_DEEP=model-a1,model-a2;model-b1
# LLM_MODEL_NORMAL=model-a3;model-b2
# LLM_MODEL_BASIC=model-a4;model-b3

# provider 级快捷入口（推荐）
OPENAI_API_KEY=your-api-key-here
# ANTHROPIC_API_KEY=
# DEEPSEEK_API_KEY=
# DASHSCOPE_API_KEY=
# BIGMODEL_API_KEY=
# ZAI_API_KEY=

# WebSocket 配置
WS_BIND_ADDRESS=127.0.0.1:42470
WS_MAX_CONNECTIONS=100
```

#### LLM 环境变量配置说明

> **重要提示**：当前 LLM provider 配置由 ApoRia 统一管理。环境变量只作为启动引导入口，不再是长期配置源。

**工作机制**：

1. 当 TUI 需要自动启动 server 时，会读取通用 `LLM_*` 快速初始化变量，或 `OPENAI_API_KEY` 这类 provider 级变量。多 provider 配置使用分号分隔的平行数组：`LLM_API_KEY`、`LLM_BASE_URL`、`LLM_PROTOCOL`、`LLM_MODEL_DEEP`、`LLM_MODEL_NORMAL`、`LLM_MODEL_BASIC`。编程套餐环境变量（如 `BIGMODEL_API_KEY_CODING_PRO`）也支持分号分隔多个密钥，自动编号 `(#2)`、`(#3)`。自定义 provider 会在括号中显示域名。
1. 在 server 启动前，TUI 会先把首批 provider 配置预写到 `res/prompts/agents/aporia/config.toml`
1. 预写完成后，以 ApoRia 配置和 TUI 的 Models 页面为准
1. 已存在且 API key 非空的 provider 不会被环境变量覆盖

**建议用法**：

- 使用环境变量完成首次引导
- 后续统一通过 Models 页面或 `res/prompts/agents/aporia/config.toml` 维护

### 步骤 3: 启动服务

```bash
# 使用 Docker Compose 启动所有服务
docker compose up -d

# 或者使用 just 命令（如果已安装）
just dev
```

---

## 配置

### LLM 提供商配置

Entelecheia（玄枢） 支持多个 LLM 提供商。配置您首选的提供商：

#### OpenAI

```bash
OPENAI_API_KEY=sk-...
```

#### Anthropic

```bash
ANTHROPIC_API_KEY=sk-ant-...
```

#### 本地 LLM（Ollama）

```bash
# 通过 Models 页面或 res/prompts/agents/aporia/config.toml 配置本地 provider
# endpoint = http://localhost:11434
# model = llama2
```

### Docker 配置

```bash
# Docker socket（通常自动检测）
DOCKER_HOST=unix:///var/run/docker.sock

# 容器设置
CONTAINER_NETWORK=entelecheia-network
CONTAINER_REGISTRY=127.0.0.1:5000
```

---

## 构建

### 开发构建

```bash
# 快速开发构建
just build-dev
```

### 生产构建

```bash
# 优化的发布构建
just build
```

### 构建特定组件

```bash
# 仅构建服务器
cargo build -p scepter

# 仅构建 TUI
cargo build -p entelecheia-tui

# 构建特定代理
cargo build -p haplotes
```

### 构建产物

构建完成后，您将找到：

- **二进制文件**: `target/debug/` 或 `target/release/`
- **Docker 镜像**: 在 `just dev` 期间自动构建

---

## 运行

### 开发模式

```bash
# 启动完整开发环境（包含 TUI）
just dev

# 仅启动服务器（无 TUI）
just dev --no-tui

# 清洁启动（删除所有数据）
just dev-clean
```

### 生产模式

```bash
# 启动服务器
just server

# 启动 TUI 客户端
just tui

# 启动所有代理
just agents-up
```

### 终端兼容性参数

TUI 依赖 ANSI 转义序列、鼠标事件和图像渲染（Sixel/Kitty 协议）。在受限的终端环境中——如 SSH 会话、串口控制台、CI 运行器或旧版终端模拟器——可以使用三个渐进式降级参数：

#### `--no-image-render`

禁用所有图像渲染。其余功能——颜色、鼠标、差异刷新——保持完全正常。

```bash
just tui -- --no-image-render
```

适用场景：终端支持颜色和鼠标，但缺少 Sixel/Kitty 图像协议支持（最常见的情况）。

#### `--no-ansi`

禁用鼠标捕获和特殊按键监听。颜色和差异（部分）屏幕刷新保留。当鼠标事件干扰终端选择、复制粘贴或回滚历史时很有用。

```bash
just tui -- --no-ansi
```

适用场景：需要颜色但鼠标捕获造成问题（终端复用器、`screen`、基础 `tmux` 配置等）。

#### `--no-ansi-pure`

纯单色模式——最激进的降级。禁用所有 ANSI 颜色（全局强制 `Color::Reset`），禁用鼠标捕获，每帧进行全屏重绘。启动画面 Logo 替换为纯 ASCII 艺术字版本。此参数隐含 `--no-ansi`。

```bash
just tui -- --no-ansi-pure
```

适用场景：通过最小化终端支持的 SSH、串口控制台、`docker exec`、CI 环境运行，或任何不能正确处理 ANSI 颜色代码的终端。

#### 参数对比

| 功能 | 默认 | `--no-image-render` | `--no-ansi` | `--no-ansi-pure` |
| --- | --- | --- | --- | --- |
| 颜色 | 完整 | 完整 | 完整 | 禁用 |
| 鼠标捕获 | 是 | 是 | 否 | 否 |
| 图像渲染 | 是 | 否 | 否 | 否 |
| 屏幕刷新 | 差异 | 差异 | 差异 | 全屏重绘 |
| 启动 Logo | ANSI 彩色 | ANSI 彩色 | ANSI 彩色 | 纯 ASCII 艺术字 |

### 服务管理

```bash
# 检查服务状态
just dev-status

# 查看日志
just dev-logs

# 停止服务
just dev-down

# 强制终止所有服务
just dev-kill
```

---

## 数据库管理

### 初始化数据库

```bash
# 创建数据库
just db-create

# 运行迁移
just db-migrate

# 使用种子数据初始化
just db-init
```

### 数据库操作

```bash
# 检查数据库状态
just db-status

# 备份数据库
just db-backup

# 恢复数据库
just db-restore backups/backup_xxx.sql

# 重置数据库（警告：删除所有数据）
just db-reset
```

### 迁移管理

```bash
# 创建新迁移
cargo test -p scepter test_create_migration -- --nocapture --ignored

# 回滚上次迁移
just db-migrate-down
```

---

## 开发环境

### 环境设置

```bash
# 初始化所有依赖
just init

# 检查 Python 依赖

# 格式化代码
just fmt

# 运行代码检查
just clippy
```

### 测试

```bash
# 运行所有测试
just test

# 运行特定类型的测试
just test unit
just test integration
just test e2e
just test llm-providers

# 详细输出
just test verbose
```

### 代码质量

```bash
# 格式化代码
just fmt

# 检查格式
just fmt-check

# 运行 clippy
just clippy

# 类型检查
just check
```

---

## 部署

### Docker 部署

#### 构建镜像

```bash
docker build -t entelecheia:latest .
```

#### 运行容器

```bash
docker run -d --name entelecheia \
  --env-file .env \
  -p 8424:8424 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  entelecheia:latest
```

### Docker Compose 部署

```bash
# 启动所有服务
docker compose up -d

# 查看日志
docker compose logs -f

# 停止服务
docker compose down
```

---

## 故障排除

### 常见问题

#### Docker 权限被拒绝

```bash
# 将用户添加到 docker 组
sudo usermod -aG docker $USER

# 注销并重新登录
```

#### 端口已被占用

```bash
# 检查占用端口的进程
lsof -i :8424

# 终止进程
kill -9 <PID>
```

#### 构建失败

```bash
# 清理构建产物
cargo clean

# 更新依赖
cargo update

# 重新构建
just build
```

#### 容器无法启动

```bash
# 检查 Docker 日志
docker compose logs

# 重新构建容器
docker compose down
docker compose build --no-cache
docker compose up -d
```

### 获取帮助

1. 搜索 [GitHub Issues](https://github.com/celestia-island/entelecheia/issues)
1. 加入我们的[讨论区](https://github.com/celestia-island/entelecheia/discussions)

---

## 运行 Webhook 机器人

Webhook 机器人位于 `plugins/github-webhook/` 下。每个平台都有独立的目录。

### 前置条件

- Python 3.10+（当前机器人）
- Node.js 18+（未来的 TypeScript 迁移）
- 各平台的 bot token（参见 [Webhook 配置指南](webhook-setup.md)）

### 运行单个机器人

```bash
# GitHub
cd plugins/github-webhook/github
pip install -r requirements.txt
python bot.py

# Gitee
cd plugins/github-webhook/gitee
pip install -r requirements.txt
python bot.py

# Discord
cd plugins/github-webhook/discord
pip install -r requirements.txt
python bot.py
```

### 运行所有机器人

```bash
just webhooks-up
```

### 环境变量

复制示例环境文件并进行配置：

```bash
cp plugins/github-webhook/.env.example plugins/github-webhook/.env
```

各平台的具体配置详情请参见 [Webhook 配置指南](webhook-setup.md)。

---

## 下一步

- 阅读[基础指南](fundamentals.md)以了解架构
- 浏览[代理文档](../../agents/)以了解可用的代理

---

**祝您构建愉快！** 🚀
