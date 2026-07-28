+++
name = "file_edit"

[description]
en = "Edit file content with partial replacement"
zh-Hans = "通过部分替换编辑文件内容"
zh-Hant = "透過部分替換編輯檔案內容"
ja = "部分置換によるファイル内容の編集"
ko = "부분 교체로 파일 내용 편집"
fr = "Modifier le contenu d'un fichier avec remplacement partiel"
es = "Editar el contenido del archivo con reemplazo parcial"
ru = "Редактировать содержимое файла с частичной заменой"
+++

# file_edit

Performs a find-and-replace operation on a file by locating the exact `old_content` string and replacing it with `new_content`. This is useful for making targeted edits without rewriting the entire file. The operation fails if the `old_content` string is not found in the file.

## Parameters

- **path** (required, string): The absolute file path to edit.
- **`old_content`** (required, string, separate-call): The exact content to find and replace. Provide via `file_edit.old_content("...")` in a follow-up call. Must match exactly.
- **`new_content`** (required, string, separate-call): The replacement content. Provide via `file_edit.new_content("...")` in a follow-up call.

## Returns

### On Success

Returns `{ ok: true, data: { path: string, status: string, occurrences: number }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Update a configuration value

```text
path: "/home/user/config.json"
old_content: "\"port\": 3000"
new_content: "\"port\": 8080"
```

Finds `"port": 3000` in the file and replaces it with `"port": 8080`.

### Example 2: Fix a typo in source code

```text
path: "/home/user/project/src/main.rs"
old_content: "pritnln!"
new_content: "println!"
```

Replaces the misspelled `pritnln!` macro with the correct `println!`.

### Example 3: Content not found (failure case)

```text
path: "/home/user/config.json"
old_content: "old_setting: true"
new_content: "new_setting: false"
```

If `old_setting: true` does not exist in the file, the operation fails with an error.

## Important Notes

- The `old_content` string must match exactly, including whitespace and line breaks.
- If `old_content` appears multiple times, only the first occurrence is replaced.
- If `old_content` is not found, the file is left unchanged and an error is returned.
- This tool does not support regex; use literal string matching only.
