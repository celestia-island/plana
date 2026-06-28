+++
title = "Entelecheia 生产部署检查清单"
description = """> 将 Entelecheia 部署到生产的 12 步检查清单。"""
lang = "zhs"
category = "design"
subcategory = "core"
+++

# Entelecheia 生产部署检查清单

> 将 Entelecheia 部署到生产的 12 步检查清单。

## 部署前

- [ ] **1. 选择数据库模式**
  - 嵌入式 pglite：单二进制，无需外部数据库。适用于 <50 个并发 Agent。
  - PostgreSQL：推荐用于生产。设置 `DATABASE_URL`。

  ```bash
  # 嵌入式模式
  docker run -d -p 8080:8080 -v data:/data entelecheia:latest

  # PostgreSQL 模式
  docker-compose up -d
  ```

- [ ] **2. 配置用户身份**

  ```bash
  export ENTELECHEIA_USER_UUID=$(uuidgen)
  ```

此 UUID 是工作区所有者身份。所有 Agent 操作的范围限定于此。

- [ ] **3. 设置 LLM 提供商**

  ```bash
  entelecheia-cli config set-provider openai --api-key sk-...
  entelecheia-cli config set-provider anthropic --api-key sk-ant-...
  ```

API 密钥通过 Aporia Agent 以 AES-256-GCM 加密存储。

- [ ] **4. 配置容器运行时**
  - Docker（默认）：`--container-backend docker`
  - Youki（无根 OCI）：`--container-backend youki`
  - 验证 seccomp 配置文件：`configs/seccomp/`

- [ ] **5. 审查安全策略**

  ```bash
  # 列出已注册的安全策略
  entelecheia-cli security policy-list

  # 审查 OreXis 哨兵配置
  entelecheia-cli config show orexis
  ```

## 部署

- [ ] **6. 构建或拉取镜像**

  ```bash
  # 从源码构建
  docker build -t entelecheia:latest .

  # 或使用发布版
  curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
  ```

- [ ] **7. 启动服务**

  ```bash
  # 使用 Docker Compose（推荐）
  docker-compose up -d

  # 或独立运行
  docker run -d --name entelecheia \
    -p 8080:8080 \
    -v entelecheia-data:/data \
    -e ENTELECHEIA_USER_UUID=$ENTELECHEIA_USER_UUID \
    --restart unless-stopped \
    entelecheia:latest
  ```

- [ ] **8. 验证健康状态**

  ```bash
  entelecheia-cli status
  curl http://localhost:8080/health
  ```

- [ ] **9. 初始化 Agent 的 Docker 镜像**

  ```bash
  entelecheia-cli init-docker-images
  ```

为每个 Layer-1 Agent 构建用于隔离执行的容器镜像。

## 部署后

- [ ] **10. 设置监控**

  ```bash
  # 启用追踪
  export RUST_LOG=info,entelecheia=debug

  # 检查时间线是否有问题
  entelecheia-cli timeline list --agent orexis
  ```

- [ ] **11. 配置备份**
  - 嵌入式模式：备份 `/data` 目录
  - PostgreSQL：`pg_dump` 或 WAL 归档
  - 时间线审计日志：定期导出

- [ ] **12. 负载测试**

  ```bash
  # 发送测试消息
  entelecheia-cli send "你好，验证系统是否正常运行"

  # 检查 Agent 状态
  entelecheia-cli agent list

  # 验证审计追踪
  entelecheia-cli trace-chain demiurge.001
  ```

## 安全加固（推荐）

| 检查项 | 命令 |
| --- | --- |
| 验证环境变量中无密钥泄漏 | `env \| grep -i key` |
| 审查 RBAC 组 | `entelecheia-cli security rbac-list` |
| 检查速率限制 | `entelecheia-cli config show channel.rate_limit` |
| 验证容器隔离 | `docker inspect entelecheia \| grep SecurityOpt` |
| 审查 OreXis 审计日志 | `entelecheia-cli logs --agent orexis --lines 100` |

## 故障排除

| 症状 | 诊断 |
| --- | --- |
| Agent 无响应 | `entelecheia-cli status` → 检查 scepter 是否在运行 |
| LLM 调用失败 | 检查 API 密钥：`entelecheia-cli config show providers` |
| 容器错误 | `docker logs entelecheia` → 查找 Youki/Docker 错误 |
| 数据库问题 | 检查 `DATABASE_URL` 或 pglite 数据目录权限 |
| 工具权限被拒绝 | `entelecheia-cli security policy-list` → 审查被拒绝的调用 |
