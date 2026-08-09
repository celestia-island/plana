+++
id = "commit-convention-config"
title = "提交格式配置"
kind = "system"
+++

# Commit Convention — Active Configuration

> **这是"前台"配置**：修改本文件的 `active` 字段即可切换整个提示词体系使用的提交格式规则。
> 优先级（高→低）：**用户任务内明确指定 > 本配置 active > 仓库分析推荐 > 默认 gitmoji**。
> 用户还可以用自然语言直接描述想要的格式（见 README §5"自定义描述"），
> 例如"末尾不要句号"、"中文也行"、"用 semantic commit"。

```toml
# 当前启用的预设。可选值：gitmoji（默认）/ conventional / plain / custom
active = "gitmoji"

# 可用的预设列表（每个预设对应本目录下的同名 .md 文件）
presets = ["gitmoji", "conventional", "plain"]

# 备注（可选）：为什么选这套。会被分析 skill 与用户看到。
note = "Org default — consistent with the CI commit-msg linter."
```

## 前台可改的字段

| 字段 | 含义 | 示例 |
|------|------|------|
| `active` | 当前使用的预设 | `active = "conventional"` |
| `note` | 选择理由（可选） | `note = "客户仓库用语义化提交"` |

## 覆盖优先级（从高到低）

1. **用户任务内明确指定** — 任务描述包含"提交格式/commit message/commit convention"
   相关指令时，以用户指令为准（例如"用 conventional"、"消息末尾不要句号"）。
2. **项目级覆盖（per-repo）** — 仓库本地配置（AGENTS.md / .opencode/ 等）可指定
   `commit-convention = "<preset>"` 覆盖全局（见 README §8）。
3. **本配置 `active`** — 全局前台配置。
4. **仓库分析推荐** — 初次接入已有仓库时，`analyze_commit_convention` skill
   分析历史提交后给出推荐（见 README §4）。
5. **默认 gitmoji** — 以上均未指定时的兜底。
