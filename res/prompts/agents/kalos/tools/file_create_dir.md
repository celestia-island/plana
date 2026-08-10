+++
name = "file_create_dir"

[description]
en = "Create new directories with recursive support"
+++

# file_create_dir

Creates a new directory at the specified path, including all parent directories as needed (recursive creation). Returns the directory path on success. This is equivalent to `mkdir -p` on Unix systems.

## Parameters

- **path** (required, string): The absolute directory path to create. Parent directories are created automatically.

## Returns

### On Success

Returns `{ ok: true, data: { path: string, status: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Create a single directory

```text
path: "/home/user/new_folder"
```

Creates the directory `new_folder` inside `/home/user/`.

### Example 2: Create nested directories

```text
path: "/home/user/project/src/components"
```

Creates the entire directory hierarchy: `project/`, `src/`, and `components/`, including any intermediate directories that do not yet exist.

### Example 3: Create a directory that already exists

```text
path: "/home/user/existing_folder"
```

Returns success if the directory already exists (idempotent operation).

## Important Notes

- Parent directories are created automatically if they do not exist.
- The operation is idempotent: creating a directory that already exists does not return an error.
- The path must be an absolute path.
