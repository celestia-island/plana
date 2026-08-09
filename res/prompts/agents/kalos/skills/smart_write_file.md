+++
name = "smart_write_file"
agent = "kalos"

[[next_action]]
agent = "hubris"
name = "plan_execute"

[description]
en = "Safe file writing with backup, diff preview, and conflict prevention"
zh-Hans = "安全文件写入：备份、差异预览、冲突预防"
zh-Hant = "安全檔案寫入：備份、差異預覽、衝突預防"
ja = "バックアップ、差分プレビュー、競合防止を備えた安全なファイル書き込み"
ko = "백업, 차이 미리보기, 충돌 방지를 갖춘 안전한 파일 쓰기"
fr = "Écriture de fichier sécurisée avec sauvegarde, aperçu des différences et prévention des conflits"
es = "Escritura de archivo segura con copia de seguridad, vista previa de diferencias y prevención de conflictos"
ru = "Безопасная запись файлов с резервным копированием, просмотром различий и предотвращением конфликтов"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_edit"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_exists"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_list"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "cosmos"
tool_name = "exec"

[features]
location = "cosmos"
execution_mode = "write"
+++

# smart_write_file

This skill is the **only gateway for writing and editing files** in the workspace.

## PATH WARNING

All file paths MUST use `/workspace/` prefix. Do NOT use the host path from the workspace URI (e.g. `/opt/...`). The container's workspace is at `/workspace`.

## CRITICAL: ONE EXEC CALL PER FILE

Pre-flight + write + verify MUST happen in a **single** `exec` call. Do NOT split across multiple calls — the system detects missing `file_write` if it's not in the same exec.

**NEVER end an exec with just `file_exists()` — always include `file_write()`.**

## Single Exec Pattern

```javascript
exec({ code: "import { file_exists, file_write, file_read } from 'kalos';
const exists = await file_exists({ path: '/workspace/TARGET_FILE' });
if (!exists.exists) {
  const r = await file_write({ path: '/workspace/TARGET_FILE', content: '...' });
  console.log('WROTE:', JSON.stringify(r));
} else {
  const current = await file_read({ path: '/workspace/TARGET_FILE' });
  const r = await file_write({ path: '/workspace/TARGET_FILE', content: '...' });
  console.log('UPDATED:', JSON.stringify(r));
}
const verify = await file_read({ path: '/workspace/TARGET_FILE' });
console.log('VERIFIED:', verify?.content?.length, 'bytes');" })
```

## Critical Rules

- **ALL paths must use `/workspace/` prefix**.
- **`file_write`() MUST be in EVERY exec call** — do not end with just `file_exists`().
- **ONE exec call per file** — combine check, write, verify in one call.
- **Never write** to `.git/`, system directories, or paths outside `/workspace`.
- **Report actual results** using `__vars` syntax.
- **SECRET HYGIENE — HARD RULE**: Before writing any file, scan the content for real
  credentials (passwords, private keys, tokens, API keys), internal IPs (`192.168.x`,
  `10.x`, `172.16-31.x`), and internal paths (`/mnt/...`). If found, **refuse to write**
  and replace with environment variable references or placeholders (see
  `@system/repo-hygiene`). Example IPs must use RFC 5737 documentation addresses.

> IEPL-first execution rules: @system/iepl-first
> Repository hygiene hard rules: @system/repo-hygiene
