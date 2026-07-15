+++
name = "screenshot"
agent = "web_automation"

[description]
en = "Capture a screenshot of the current page or a specified element"
zhs = "截取当前页面或指定元素的屏幕截图"
zht = "擷取目前頁面或指定元素的螢幕截圖"
ja = "現在のページまたは指定された要素のスクリーンショットを撮る"
ko = "현재 페이지 또는 지정된 요소의 스크린샷 캡처"
fr = "Capturer une capture d'écran de la page actuelle ou d'un élément spécifié"
es = "Capturar una captura de pantalla de la página actual o un elemento especificado"
ru = "Сделать снимок экрана текущей страницы или указанного элемента"
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
