+++
name = "file_delete"

[description]
en = "Delete specified files or directories"
zhs = "删除指定的文件或目录"
zht = "刪除指定的檔案或目錄"
ja = "指定されたファイルまたはディレクトリを削除"
ko = "지정된 파일 또는 디렉토리 삭제"
fr = "Supprimer les fichiers ou répertoires spécifiés"
es = "Eliminar archivos o directorios especificados"
ru = "Удалить указанные файлы или каталоги"
+++

# file_delete

Deletes the file at the specified path from the filesystem. Returns the file path on success. This operation is irreversible, so use with caution.

## Parameters

- **path** (required, string): The absolute file path to delete.

## Returns

### On Success

Returns `{ ok: true, data: { path: string, status: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Delete a temporary file

```text
path: "/home/user/temp_output.log"
```

Removes the temporary log file and returns the path.

### Example 2: Delete an old backup

```text
path: "/home/user/backup_2024_old.tar.gz"
```

Removes the specified backup archive.

### Example 3: Delete a non-existent file (failure case)

```text
path: "/home/user/nonexistent.txt"
```

Returns an error since the file does not exist.

## Important Notes

- This operation is permanent and cannot be undone.
- Ensure the path is correct before deleting.
- The tool requires write permissions on the parent directory.
- Consider backing up important data before deletion.
