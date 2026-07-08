+++
name = "mouse_click"
agent = "web_automation"

[description]
en = "Simulate mouse click operations within a browser page"
zhs = "在浏览器页面中模拟鼠标点击操作"
zht = "在瀏覽器頁面中模擬滑鼠點擊操作"
ja = "ブラウザページ内でマウスクリック操作をシミュレートする"
ko = "브라우저 페이지 내에서 마우스 클릭 작업 시뮬레이션"
fr = "Simuler les opérations de clic de souris dans une page de navigateur"
es = "Simular operaciones de clic del ratón dentro de una página del navegador"
ru = "Имитировать операции щелчка мыши на странице браузера"
+++

# mouse_click

## Description

Simulates a mouse click on an element identified by a CSS selector within the browser page. Dispatches the full sequence of mouse events (mousedown, mouseup, click) to mimic real user interaction. Use this to click buttons, links, checkboxes, or any clickable element.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier (obtained from `create`)
- **selector** (string, required): CSS selector identifying the element to click (e.g., `"#submit-btn"`, `"a.login-link"`)

## Returns

### Success

```text
Click executed

Browser ID: browser_abc123
Selector: #submit-btn
```

### Failure

```text
Click failed

Browser ID: browser_abc123
Error: Element not found
Selector: #nonexistent-btn
```

## Examples

### Example 1: Click a submit button

```text
browser_id: "browser_abc123"
selector: "#submit-btn"
```

### Example 2: Click a navigation link

```text
browser_id: "browser_abc123"
selector: "a[href='/about']"
```

### Example 3: Click an element by text content

```text
browser_id: "browser_abc123"
selector: "button:has-text('Sign In')"
```

## Important Notes

- **Visibility**: The target element must be visible and not obscured by other elements. Off-screen or hidden elements cannot be clicked
- **Wait for element**: If the element loads asynchronously, use `execute_script` to wait for it before clicking
- **Multiple matches**: If the selector matches multiple elements, the first match is clicked
- **No selector validation**: Invalid CSS selectors will cause an error
