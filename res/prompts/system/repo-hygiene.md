+++
id = "repo-hygiene"
title = "仓库卫生红线"
kind = "system"
+++

# Repository Hygiene — Hard Rules for Generated Code

> 组织事故背景（2026-08-09）：真实 SSH 密码曾被写进 install 脚本并提交到 git，
> 最终动用 filter-repo 重写全部历史 + 移动全部 tag 才清除，下游 rev 全部重映射。
> 以下规则为硬性红线，违反即视为事故。

## 1. 禁止把真实凭据写进 git 树

**任何生成的代码 / 脚本 / 配置 / 文档 / 示例 / 测试数据中，禁止包含：**

- 真实密码、SSH 密码、私钥、token、API key、数据库连接串
- 内网 IP（`192.168.x` / `10.x` / `172.16-31.x`）
- 真实机构名 / 站点名 / 内部文档路径（如 `/mnt/...`）

反例（全部真实发生过）：

```bash
SSH_PASS="hydroSinap2024"          # ❌ 真实密码
--target-pass hydroSinap2024       # ❌ 真实密码
host = "192.168.2.148"             # ❌ 内网 IP
# Source: /mnt/sdb1/internal-doc.md  # ❌ 内部路径
```

## 2. 需要凭据时的正确做法

- 用**环境变量名**（`${SSH_PASS}`）或**占位符**（`<your-password>` / `CHANGE_ME` / `$secret.XXXXXX`）
- **示例值一律用假值**：RFC 5737 文档地址（`192.0.2.x` / `198.51.100.x` / `203.0.113.x`）、
  `test-password`、`sk-xxx`、`example-key`
- 真实凭据只存在于环境变量 / 配置文件（不入库）中

```bash
SSH_PASS="${SSH_PASS:-}"                        # ✅ 环境变量
--target-pass "${SSH_PASS}"                     # ✅ 引用环境变量
host = "192.0.2.148"                            # ✅ 文档地址
```

## 3. 提交消息格式（可配置，默认 gitmoji）

提交格式走 `@system/commit-convention` 可配置机制：

- **默认预设 gitmoji**：`<gitmoji> <Capitalized English summary ending with period.>`
- **可选预设**：conventional（`type(scope): desc`）/ plain（自由式）
- **覆盖优先级**：用户任务内指定 > 项目级覆盖 > config.md active > 仓库分析推荐 > 默认 gitmoji
- 用户可用自然语言描述自定义格式（如"末尾不要句号"）

gitmoji 默认规则：
- 必须以 gitmoji 开头（gitmoji.dev 规范集）
- 英文一句话、大写开头、句号结尾
- **禁止** `fix:` / `feat:` / `xxx(scope):` 冒号前缀
- 禁止中文摘要（easy-hydro-* 两仓豁免）

✅ `🐛 Fix clippy warnings from the auto-fix cycle.`
❌ `feat(scope): add feature` / `chore: auto-fix`

## 4. 提交前自查清单

写文件 / 生成代码 / 提交前，必须检查：

1. `grep -rn "password\|secret\|token\|api_key"` 改动内容 → 确认无真实值
2. 示例 IP 是否用了 RFC 5737 文档地址
3. `.env` / 私钥 / 证书文件是否会被纳入 git（应被 .gitignore 排除）
4. 内网 IP / 内部路径 / 真实机构名是否出现

## 5. 发现泄漏时的处置

- 立即从当前改动中删除该凭据
- 报告人类（`report_human`），由人类决定是否历史重写
- 凭据一旦入库即视为已公开，必须轮换
