+++
name = "keypress"
agent = "web_automation"

[description]
en = "Simulate keyboard key press operations within a browser page"
zh-Hans = "在浏览器页面中模拟键盘按键操作"
zh-Hant = "在瀏覽器頁面中模擬鍵盤按鍵操作"
ja = "ブラウザページ内でキーボードキー操作をシミュレートする"
ko = "브라우저 페이지 내에서 키보드 키 입력 작업 시뮬레이션"
fr = "Simuler les opérations de pression de touches dans une page de navigateur"
es = "Simular operaciones de pulsación de teclas dentro de la página del navegador"
ru = "Имитировать операции нажатия клавиш на странице браузера"
+++

# keypress

## Description

Simulates a keyboard key press within the browser page. Sends the specified key to the currently focused element, or to the page if no element is focused. Supports special keys such as `Enter`, `Tab`, `Escape`, `ArrowDown`, and character input.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier (obtained from `create`)
- **key** (string, required): The key to press. Use character keys like `"a"`, `"1"`, or special key names like `"Enter"`, `"Tab"`, `"Escape"`, `"ArrowDown"`, `"Backspace"`, `"Delete"`

## Returns

### Success

```text
Key pressed

Browser ID: browser_abc123
Key: Enter
```

### Failure

```text
Key press failed

Browser ID: browser_abc123
Error: Invalid key name
Key: InvalidKey
```

## Examples

### Example 1: Press Enter to submit

```text
browser_id: "browser_abc123"
key: "Enter"
```

### Example 2: Press Escape to close a modal

```text
browser_id: "browser_abc123"
key: "Escape"
```

### Example 3: Type a character

```text
browser_id: "browser_abc123"
key: "a"
```

## Important Notes

- **Focus**: The key event is sent to whichever element currently has focus. Use `mouse_click` on an input field first to focus it
- **Special keys**: Valid special keys include `Enter`, `Tab`, `Escape`, `Backspace`, `Delete`, `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `Home`, `End`, `PageUp`, `PageDown`, `F1`-`F12`
- **Modifier keys**: For shortcuts like Ctrl+C, use `execute_script` with `document.execCommand` or the Keyboard API
- **Sequential input**: To type a string, call this tool once per character, or use `execute_script` to set the value directly
