+++
id = "commit-convention-gitmoji"
title = "提交格式预设：gitmoji"
kind = "system"
+++

# Commit Convention — gitmoji（默认）

## 格式

```text
<gitmoji> <Capitalized English summary ending with period.>
```

## 规则

1. **必须以 gitmoji 开头**：gitmoji.dev 规范集（✨ 🐛 🔧 ♻️ 🔥 📝 🎨 ✅ 🚀 🌐 ⬆️ 🎉 📦 等）。
2. **英文一句话**：大写开头、句号结尾。
3. **禁止冒号前缀**：不得写成 `fix:` / `feat:` / `xxx(scope):`（CI lint 规则 7 拒绝）。
4. **禁止中文摘要**（easy-hydro-* 两仓豁免）。
5. 禁止 "Merge branch xxx"（用 squash merge）。
6. 详细说明放正文（空行 + bullets），不进摘要行。

## 示例

```text
✅ Add regression tests for the modbus write path.
🐛 Fix stale CSRF cookie path assertion.
♻️ Refactor the sensor poller to per-station concurrent tasks.
```

## 反例

```text
feat(scope): add feature        # ❌ 冒号前缀
chore: auto-fix                 # ❌ 冒号前缀
修复一个问题                     # ❌ 中文 + 无 gitmoji
```
