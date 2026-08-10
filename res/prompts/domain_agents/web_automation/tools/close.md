+++
name = "close"
agent = "web_automation"

[description]
en = "Close a specified browser instance and release resources"
+++

# close

## Description

Closes a browser instance identified by `browser_id` and releases all associated resources (memory, file handles, network connections). Always close browser instances when they are no longer needed to prevent resource leaks.

## Parameters

- **`browser_id`** (string, required): The unique identifier of the browser instance to close (obtained from `create`)

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Browser closed successfully

Browser ID: browser_abc123
Resources released: memory, file handles, network connections
```

### Failure

```text
Browser close failed

Error: Browser instance not found
Browser ID: browser_nonexistent
Message: No active browser with the specified ID.
```

## Examples

### Example 1: Close after testing

```text
browser_id: "browser_abc123"
```

### Example 2: Close after screenshot capture

```text
browser_id: "browser_xyz789"
```

## Important Notes

- **Resource cleanup**: Always close browsers after use. Unclosed instances consume memory and may exhaust available browser slots
- **Session data**: Closing a browser discards all in-memory session data (cookies, `localStorage`, DOM state). Save any needed data before closing
- **Active operations**: If the browser is performing an operation (navigation, recording), it will be interrupted. Stop recordings before closing
- **Invalid IDs**: Passing an already-closed or nonexistent browser ID returns an error but does not affect other instances
