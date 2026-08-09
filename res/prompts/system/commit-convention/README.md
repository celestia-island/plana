+++
id = "commit-convention-readme"
title = "提交格式机制说明"
kind = "system"
+++

# Commit Convention — Mechanism

提交格式规则是**可配置的**，不再硬编码。本目录提供预设模板 + 前台配置 +
仓库分析 + 任务覆盖 + 自定义描述五层机制。

> **通用模板机制（第一个实例）**：本目录是"可配置模板系统"的首个实例。
> 后续其他模板（如 Layer2 的 UI 文案、代码风格、文档骨架等）沿用同一套结构：
> `preset 模板文件 + config.md 前台配置 + 分析/推荐 skill + 任务级覆盖 + 自定义描述`，
> 放在 `res/prompts/system/<template-name>/` 下即可复用本机制。
> 目前仅 commit-convention 可改；未来模板接入时按本 README §7 扩展。

## 1. 预设模板

| 文件 | 预设 | 格式 | 适用 |
|------|------|------|------|
| `gitmoji.md` | **gitmoji（默认）** | `<gitmoji> <Capitalized English summary ending with period.>` | org 默认；CI lint 兼容 |
| `conventional.md` | 语义化提交 | `<type>(<scope>): <description>` | 常见开源仓库；`fix:`/`feat:` 前缀 |
| `plain.md` | 自由式 | 无强制前缀，一句描述 | 仓库无统一格式或用户不关心 |

## 2. 前台配置（config.md）

`active` 字段决定当前使用的预设。**修改 config.md 即全局切换**。
未配置时默认 `gitmoji`。

## 3. 任务内覆盖

用户在任务描述中指定格式时**以用户为准**：

- "提交消息用 conventional 格式" → 用 `conventional` 预设
- "commit messages: gitmoji" → 用 `gitmoji` 预设
- "不要句号" / "末尾不要标点" → 在选中预设基础上移除句号要求

## 4. 已有仓库初次接入：analyze_commit_convention

接入一个**已有提交历史**的仓库时，先运行
`hubris/skills/analyze_commit_convention` 分析 skill：

1. 抽样最近 20~50 条提交（`git log --oneline`）；
2. 统计格式特征：
   - 首 token 是否为 gitmoji（emoji 集合匹配）；
   - 是否匹配 `type(scope):` / `type:` 前缀模式；
   - 是否含句号结尾；是否含中文；是否大小写敏感；
3. 与三个预设模板打分匹配；
4. **报告推荐**（含置信度与依据样本），由用户确认或覆盖；
5. 用户确认后把结果写入 `config.md` 的 `active`。

## 5. 自定义描述（custom）

用户用自然语言描述想要的格式时，agent 将其解析为**显式规则**（覆盖或微调预设）：

| 用户表述 | 解析结果 |
|----------|----------|
| "末尾不要句号" | 移除句号要求 |
| "用 semantic commit" | `conventional` 预设 |
| "中文也可以" | 允许中文摘要 |
| "前缀加个 #123" | 描述后追加 issue 引用 |
| "就写一句话随便点" | `plain` 预设 |

解析出的规则以 `@system/commit-convention/custom` 形式临时生效（不落盘），
或用户要求持久化时写入 config.md 的 note。

## 6. 引用方式

各 skill 需要提交格式时引用：

```text
> Commit message rules: @system/commit-convention/config + @system/commit-convention/<active preset>
```

无配置时兜底引用 `gitmoji.md`。

## 7. 扩展新模板（未来 Layer2 模板接入）

新增一个可配置模板时，遵循本目录结构：

```text
res/prompts/system/<template-name>/
├── README.md      # 机制说明（复制本 README 骨架）
├── config.md      # 前台配置：active + presets + note
├── <preset-1>.md  # 预设模板
└── <preset-2>.md  # 预设模板
```

- 分析/推荐 skill 放在 `res/prompts/agents/<agent>/skills/`（如 `analyze_<topic>`）。
- 引用方式统一为 `@system/<template-name>/config + @system/<template-name>/<active>`。
- 项目级覆盖字段统一放在 config.md（见 config.md 的"项目级覆盖"节）。

## 8. 项目级覆盖（per-repo）

某个仓库需要独立于全局配置时：

- 在仓库本地（如仓库 AGENTS.md 或 `.opencode/` 配置）指定
  `commit-convention = "conventional"` 或指向自定义预设路径；
- agent 处理该仓库时以项目级覆盖 > 全局 config.active > 默认 gitmoji 为准；
- 项目级覆盖无需改 plana 提示词，只在该仓库的本地配置生效。
