+++
name = "disconnect_remote"
agent = "remote_operations"

[description]
en = "Disconnect from a remote device"
+++

# disconnect_remote

Disconnects a remote device registered with the SkeMma connection manager. The remote is identified by its UUID (returned by tools like `connect_remote_via_ssh`).

## Parameters

| Name | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `remote_id` | string | yes | — | UUID of the remote connection to disconnect |

## Returns

### On Success

```json
{
  "disconnected": true,
  "remote_id": "0192a3b4-..."
}
```

### On Failure

Returns `{ ok: false, data: null, error: string }` — e.g. invalid UUID format or remote not found.

## Examples

### Example 1: Disconnect a remote

```json
{ "remote_id": "0192a3b4-5678-..." }
```

## Important Notes

- `remote_id` must be a valid UUID v4 string.
- A remote that has already been disconnected returns a failure with message "not found or already disconnected".
