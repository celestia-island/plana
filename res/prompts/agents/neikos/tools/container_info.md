+++
name = "container_info"

[description]
en = "Get container information and status"
+++

# container_info

Retrieves detailed information and current status for a specific Docker container. Returns the container's identity, runtime configuration, and operational state including its name, image, creation timestamp, port mappings, and environment variables. Use this tool to inspect a container before performing lifecycle operations or to diagnose issues.

## Parameters

- **`container_id`** (required): The unique identifier of the container to query. This is the `container_id` returned by `container_create`, `container_list`, or a previous `container_info` call.

## Returns

### On Success

Returns `{ ok: true, data: { container_id: string, name: string, image: string, status: string, running: boolean, exit_code: number | null, ip_address: string, started_at: string, ports: string[], env: string[] }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Inspect a Running Container

Invocation:

```text
container_info container_id="a1b2c3d4e5f6"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "name": "web-server",
  "image": "node:20",
  "status": "running",
  "created_at": "2026-03-15T08:30:00Z",
  "ports": ["3000:3000"],
  "env": {"NODE_ENV": "production", "PORT": "3000"}
}
```

### Example 2: Inspect a Stopped Container

Invocation:

```text
container_info container_id="f7e8d9c0b1a2"
```

Returns:

```json
{
  "container_id": "f7e8d9c0b1a2",
  "name": "data-processor",
  "image": "python:3.12",
  "status": "stopped",
  "created_at": "2026-03-10T14:00:00Z",
  "ports": [],
  "env": {}
}
```

### Example 3: Query Non-existent Container

Invocation:

```text
container_info container_id="does-not-exist"
```

Returns:

```json
{
  "error": "Container does-not-exist not found"
}
```

## Important Notes

- Use `container_info` before performing stop, remove, or snapshot operations to verify the container exists and understand its current state.
- Environment variable values may be redacted if they contain sensitive information such as API keys or passwords.
- The `status` field is the authoritative source for determining whether a container is running, stopped, or in an error state.
