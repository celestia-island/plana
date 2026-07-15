+++
name = "file_list"

[description]
en = "List files and subdirectories in a directory"
zhs = "列出目录中的文件和子目录"
zht = "列出目錄中的檔案和子目錄"
ja = "ディレクトリ内のファイルとサブディレクトリを一覧表示"
ko = "디렉토리의 파일 및 하위 디렉토리 나열"
fr = "Lister les fichiers et sous-répertoires d'un répertoire"
es = "Listar archivos y subdirectorios de un directorio"
ru = "Список файлов и подкаталогов в каталоге"
+++

# file_list

Lists all files and subdirectories within the specified directory. Returns the directory path, total item count, and each item labeled as either a file or directory. Defaults to the current directory (`.`) if no path is provided.

## Parameters

- **path** (optional, string): The directory path to list. Defaults to `"."` (current directory).

## Returns

### On Success

Returns `{ ok: true, data: { path: string, total_count: number, items: [{ name: string, type: "file" | "directory" }] }, error: null }`.

Access items via `result.data.items` (array of `{ name, type }`).

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: List current directory

```text
path: "."
```

Lists all files and subdirectories in the current working directory with type labels.

### Example 2: List a project directory

```text
path: "/home/user/project"
```

Returns all items in the project directory, each labeled as `[file]` or `[dir]`.

### Example 3: List a non-existent directory (failure case)

```text
path: "/home/user/nonexistent_dir"
```

Returns an error since the directory does not exist.

## Important Notes

- By default, only the immediate children of the directory are listed (non-recursive).
- Pass `recursive: true` to list all descendants recursively.
- Each item is labeled with its type: `[file]` or `[dir]`.
- The path must point to a directory, not a file.
