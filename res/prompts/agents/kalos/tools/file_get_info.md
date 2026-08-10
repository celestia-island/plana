+++
name = "file_get_info"

[description]
en = "Get file or directory metadata"
+++

# file_get_info

Retrieves metadata for the file or directory at the specified path. Returns the path, type (file or directory), size in bytes, and last modification time as a Unix timestamp. This is useful for inspecting file properties before performing operations.

## Parameters

- **path** (required, string): The absolute file or directory path to inspect.

## Returns

### On Success

Returns `{ ok: true, data: { path: string, file_type: "file" | "directory", size_bytes: number, modified_unix: number }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Get info about a file

```text
path: "/home/user/document.pdf"
```

Returns the type as `file`, size in bytes, and the Unix timestamp of the last modification.

### Example 2: Get info about a directory

```text
path: "/home/user/project"
```

Returns the type as `dir` along with size and modification time.

### Example 3: Get info about a non-existent path (failure case)

```text
path: "/home/user/missing.txt"
```

Returns an error indicating the path does not exist.

## Important Notes

- Works on both files and directories.
- The modification time is returned as a Unix timestamp (seconds since epoch).
- The path must be an absolute path.
