+++
id = "commit-convention-conventional"
title = "提交格式预设：conventional"
kind = "system"
+++

# Commit Convention — Conventional Commits

## 格式

```text
<type>(<scope>): <description>
```

## 规则

1. **type 必填**：`feat` / `fix` / `docs` / `style` / `refactor` / `perf` / `test` / `build` / `ci` / `chore` / `revert`。
2. **scope 可选**：影响范围（如 `scope = auth`）。
3. **description**：祈使句、小写开头（或按仓库历史风格）。
4. 结尾句号**默认不加**（conventional 规范原文不带句号）——除非仓库历史一致带句号。
5. 破坏性变更：`feat!:` 或正文加 `BREAKING CHANGE:`。

## 示例

```text
feat(auth): add refresh token rotation
fix(api): return 401 on expired session
docs(readme): document the install flow
```

## 反例

```text
✨ Add feature            # ❌ 用了 gitmoji（本预设不适用）
Add feature               # ❌ 缺 type
```
