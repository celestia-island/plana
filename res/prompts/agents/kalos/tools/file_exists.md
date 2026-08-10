+++
name = "file_exists"

[description]
en = "Check whether a specified path exists"
+++

# file_exists

Checks whether a file or directory exists at the specified path. Returns the path and a boolean indicating existence. This is useful for validating paths before performing read, write, or edit operations.

## Parameters

- **path** (required, string): The absolute file or directory path to check.

## Returns

### On Success

Returns `{ ok: true, data: { path: string, exists: boolean }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Check if a configuration file exists

```text
path: "/etc/app/config.json"
```

Returns `Exists: true` if the file is present, `Exists: false` otherwise.

### Example 2: Check if a directory exists

```text
path: "/home/user/project"
```

Returns the directory path with its existence status.

### Example 3: Check before writing

```text
path: "/home/user/output.csv"
```

Use to verify whether a file already exists before writing, avoiding accidental overwrites.

## Important Notes

- The path must be an absolute path.
- This tool checks both files and directories.
- It never returns an error; it always returns a boolean result.
