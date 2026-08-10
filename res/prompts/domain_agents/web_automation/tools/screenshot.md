+++
name = "screenshot"
agent = "web_automation"

[description]
en = "Capture a screenshot of the current page or a specified element"
+++

# screenshot

## Description

Captures a screenshot of the current browser viewport, or of a specific element if a CSS selector is provided. Returns the image as base64-encoded data. Useful for visual verification, error reporting, and debugging automated browser sessions.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier (obtained from `create`)
- **selector** (string, optional): CSS selector to capture a specific element instead of the full viewport (e.g., `"#header"`, `".main-content"`)

## Returns

### Success

```text
Screenshot captured

Browser ID: browser_abc123
Format: png
Size: 1920x1080
Image data: <base64-encoded PNG>
```

### Failure

```text
Screenshot failed

Browser ID: browser_abc123
Error: Element not found
Selector: #nonexistent-element
```

## Examples

### Example 1: Full page screenshot

```text
browser_id: "browser_abc123"
```

### Example 2: Capture a specific element

```text
browser_id: "browser_abc123"
selector: "#login-form"
```

### Example 3: Screenshot after navigation

```text
browser_id: "browser_abc123"
selector: ".dashboard-container"
```

## Important Notes

- **No selector**: When `selector` is omitted, the entire viewport is captured
- **Element screenshots**: When a selector is provided, only the bounding box of the matched element is captured
- **Missing elements**: If the selector matches no elements, an error is returned
- **Large images**: Base64 output can be large for high-resolution viewports. Consider capturing specific elements to reduce size
