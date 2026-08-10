+++
name = "exec_on_container"
agent = "neikos"

[description]
en = "Execute a command inside a container"
+++

# exec_on_container

Executes a shell command inside a running Docker container. Supports two targeting modes: by `container_id` (the unique container identifier) or by `target_badge` (a human-readable alias like `#017`). Exactly one of these must be provided. The command runs in the container's default shell and returns stdout, stderr, and the exit code.

## Parameters

- **command** (required, string): The shell command to execute inside the container.
- **`container_id`** (optional, string): The unique identifier of the target container. Use this or `target_badge`, not both.
- **`target_badge`** (optional, string): The badge alias of the target container (e.g., `#017`). Use this or `container_id`, not both.

> **Note**: Exactly one of `container_id` or `target_badge` is required. Providing neither or both will result in an error.

## Returns

### On Success

Returns `{ ok: true, data: { container_id: string, exit_code: number, stdout: string, stderr: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Execute via container_id

```text
command: "ls -la /app"
container_id: "a1b2c3d4e5f6"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "exit_code": 0,
  "stdout": "total 32\ndrwxr-xr-x  5 root root 4096 Jan  1 00:00 .\n...",
  "stderr": ""
}
```

### Example 2: Execute via target_badge

```text
command: "python -c \"print('hello')\""
target_badge: "#017"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "exit_code": 0,
  "stdout": "hello\n",
  "stderr": ""
}
```

### Example 3: Command fails inside container

```text
command: "cat /nonexistent"
container_id: "a1b2c3d4e5f6"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "exit_code": 1,
  "stdout": "",
  "stderr": "cat: /nonexistent: No such file or directory\n"
}
```

## Important Notes

- The container must be in a running state; commands cannot be executed on stopped containers.
- Exactly one of `container_id` or `target_badge` must be provided. Supplying neither or both is an error.
- The command runs as the container's default user (typically `root`).
- Long-running commands may time out depending on the agent's execution limits.
