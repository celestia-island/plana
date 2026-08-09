+++
id = "commit-convention-plain"
title = "提交格式预设：plain"
kind = "system"
+++

# Commit Convention — Plain（自由式）

## 格式

无强制前缀，一句清晰的描述。具体风格以**仓库历史提交**为准。

## 规则

1. 无 emoji / type 前缀强制要求。
2. 描述一句话，说清楚"做了什么"。
3. 语言风格跟随仓库历史（英文优先；若仓库历史为中文可中文）。
4. 结尾标点跟随仓库历史风格。
5. 若仓库历史呈现某种一致模式（如 `[area] summary`、`summary #issue`），
   优先复刻该模式而不是自由发挥。

## 示例

```text
Add refresh token rotation
Fix 401 on expired session
Document the install flow
```

## 注意

使用本预设前**必须**先跑 `analyze_commit_convention` 分析仓库历史，
确认确实无统一格式；若历史有明确模式，按历史模式提交。
