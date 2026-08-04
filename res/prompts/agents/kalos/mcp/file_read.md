+++
name = "file_read"

[description]
en = "Read file content"
zh-Hans = "读取文件内容"
zh-Hant = "讀取檔案內容"
ja = "ファイル内容を読み取り"
ko = "파일 내용 읽기"
fr = "Lire le contenu d'un fichier"
es = "Leer el contenido del archivo"
ru = "Читать содержимое файла"
+++

# file_read

Reads the contents of a file at the specified path and returns the file path, size in bytes, and the full text content. This is the primary tool for inspecting file contents on the filesystem.

## Parameters

- **path** (required, string): The absolute file path to read.

## Returns

### On Success

Returns `{ ok: true, data: { path: string, size_bytes: number, content: string }, error: null }`.

Access content via `result.data.content`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Read a configuration file

```text
path: "/home/user/config.json"
```

Returns the full contents of `config.json` including its path and file size.

### Example 2: Read a source code file

```text
path: "/home/user/project/src/main.rs"
```

Returns the full source code with the file path and byte count.

### Example 3: Read a non-existent file

```text
path: "/home/user/missing.txt"
```

Returns an error indicating the file does not exist.

## Important Notes

- The path must be an absolute file path.
- If the file does not exist, the tool returns an error.
- Binary files may produce garbled output.
- Large files are returned in full; consider whether the file size is appropriate before reading.
