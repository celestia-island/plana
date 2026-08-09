+++
name = "file_write"

[description]
en = "Write or overwrite file content"
zh-Hans = "写入或覆盖文件内容"
zh-Hant = "寫入或覆蓋檔案內容"
ja = "ファイル内容を書き込みまたは上書き"
ko = "파일 내용 쓰기 또는 덮어쓰기"
fr = "Écrire ou écraser le contenu d'un fichier"
es = "Escribir o sobrescribir el contenido del archivo"
ru = "Записать или перезаписать содержимое файла"
+++

# file_write

Writes content to a file at the specified path, creating the file if it does not exist or overwriting it entirely if it does. Returns the file path and the total size written in bytes. This tool performs a full replacement of file contents.

## Parameters

- **path** (required, string): The absolute file path to write to.
- **content** (required, string, separate-call): The content to write. Provide via `file_write.content("...")` in a follow-up call. Defaults to an empty string if not specified.

## Returns

### On Success

Returns `{ ok: true, data: { path: string, size_bytes: number, status: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Create a new file

```text
path: "/home/user/notes.txt"
content: "Meeting notes for today."
```

Creates `notes.txt` and writes the specified content. Returns the path and byte size.

### Example 2: Overwrite an existing configuration file

```text
path: "/etc/app/config.json"
content: "{\"debug\": false, \"port\": 8080}"
```

Completely replaces the content of `config.json` with the new JSON string.

### Example 3: Write an empty file

```text
path: "/home/user/placeholder.txt"
content: ""
```

Creates or overwrites the file with empty content. Returns size of 0 bytes.

## Important Notes

- This tool overwrites the entire file. Use `file_edit` for partial replacements.
- Parent directories must exist; the tool does not create intermediate directories.
- Ensure you have write permissions for the target path.
- When writing content that includes commit-related artifacts (e.g. scripts that
  generate commit messages, hooks, CI lint configs), respect the active commit
  convention: @system/commit-convention/config (+ active preset).
