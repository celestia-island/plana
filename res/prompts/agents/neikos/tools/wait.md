+++
name = "wait"
agent = "neikos"

[description]
en = "Register a wait timer"
+++

# wait

Pauses orchestration for a specified duration without blocking the event loop. The wait is asynchronous — use `check_wait` with the returned handle to poll for completion.

## Parameters

| Name | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `seconds` | integer | no | 30 | Duration to wait in seconds (clamped 1–600 or `NEIKOS_WAIT_MAX_SECS` env var) |

## Returns

### On Success

```json
{
  "handle": "0192a3b4-...",
  "deadline": "2026-06-06T12:00:30+00:00",
  "seconds": 30,
  "status": "waiting"
}
```

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Wait 5 seconds

```json
{ "seconds": 5 }
```

### Example 2: Default wait (30s)

```json
{}
```

## Important Notes

- The handle returned by `wait` must be passed to `check_wait` to determine when the timer has elapsed.
- The maximum wait is controlled by the `NEIKOS_WAIT_MAX_SECS` environment variable (default 600).
- Minimum wait is clamped to 1 second.
