+++
name = "analyze_commit_convention"
agent = "hubris"
kind = "skill"
description = "Analyze git commit messages against the configured convention presets and suggest corrections."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
location = "cosmos"
execution_mode = "read"
+++

# analyze_commit_convention

初次接入**已有提交历史**的仓库时，分析其提交格式并推荐最合适的预设规则。
用于回答"这个仓库的提交该用什么格式"。

## 何时使用

- 仓库是既有项目（非从零创建），提交历史 > 20 条；
- 用户未在任务中明确指定提交格式；
- 需要为 `@system/commit-convention/config` 的 `active` 字段做决策。

## 流程

### 1. 抽样

```bash
git log --oneline -50
```

（不足 50 条则全量；合并提交 `Merge ...` 跳过不计）

### 2. 特征统计

对每条提交（排除 Merge）统计：

| 特征 | 判定 |
|------|------|
| gitmoji 开头 | 首 token 命中 gitmoji.dev emoji 集合 |
| type 前缀 | 匹配 `^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.*\))?:` |
| 句号结尾 | 末字符为 `.` |
| 中文 | 含 CJK 字符 |
| 大写开头 | 首字母大写 |
| 统一模式 | 多数提交符合同一模式（阈值 ≥ 70%） |

### 3. 匹配预设

| 特征组合 | 推荐 |
|----------|------|
| gitmoji 开头 ≥ 70% | `gitmoji` |
| type 前缀 ≥ 70% | `conventional` |
| 无明显统一模式 | `plain`（并提示用户确认） |
| 混合/不确定 | 报告样本，请用户决定 |

### 4. 报告

```text
## Commit Convention Analysis

- Sample: 50 commits (3 merge skipped)
- gitmoji: 42/47 (89%)
- type-prefix: 0/47
- period-ending: 30/47
- CJK: 0/47

**Recommendation: gitmoji** (89% match, org default)
Confidence: high
```

### 5. 确认与写入

- 调用 `report_human` 给出推荐，由用户确认；
- 用户确认后，将结果写入 `@system/commit-convention/config.md` 的 `active` 字段
  （如用户要求持久化）；
- 用户也可直接指定其他预设或自定义描述（见 README §3/§5）。

## 注意

- 不要修改仓库现有提交历史；只做分析。
- 分析结果仅用于后续**新提交**的格式。
- 用户任务内已明确指定格式时，跳过本 skill。
