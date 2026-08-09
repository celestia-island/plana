+++
name = "execute_script"
agent = "web_automation"

[description]
en = "Execute JavaScript code within a browser page"
zh-Hans = "在浏览器页面中执行JavaScript代码"
zh-Hant = "在瀏覽器頁面中執行JavaScript程式碼"
ja = "ブラウザページ内でJavaScriptコードを実行する"
ko = "브라우저 페이지 내에서 JavaScript 코드 실행"
fr = "Exécuter du code JavaScript dans une page de navigateur"
es = "Ejecutar código JavaScript dentro de una página del navegador"
ru = "Выполнить код JavaScript на странице браузера"
+++

# execute_script

## Description

Executes arbitrary JavaScript code within the context of the currently loaded browser page. The script runs in the page's main frame and has access to the full DOM and `window` object. Use this to extract data, manipulate the DOM, or trigger JavaScript events that cannot be accessed through standard browser automation tools.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier
- **script** (string, required, separate-call): JavaScript code to execute. Provide via `execute_script.script("...")` in a follow-up call. Use `return` to return a value from the script
- **args** (array, optional): Arguments to pass into the script. Accessed via the `arguments` array inside the script

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Script executed successfully

Browser ID: browser_abc123
Result: "Example Domain"
Return type: string
```

### Failure

```text
Script execution failed

Browser ID: browser_abc123
Error: ReferenceError: undefinedVar is not defined
Line: 1, Column: 8
```

## Examples

### Example 1: Get page title

```text
browser_id: "browser_abc123"
script: "return document.title"
```

### Example 2: Extract data from the DOM

```text
browser_id: "browser_abc123"
script: r#"return Array.from(document.querySelectorAll('.item')).map(el => el.textContent)"#
```

### Example 3: Scroll to bottom and wait

```text
browser_id: "browser_abc123"
script: r#"window.scrollTo(0, document.body.scrollHeight); return document.body.scrollHeight"#
```

## Important Notes

- **Return values**: Only JSON-serializable values can be returned (strings, numbers, booleans, arrays, plain objects). DOM elements and functions cannot be returned
- **Execution context**: The script runs in the page context with full access to `window`, `document`, and global variables
- **Async support**: Promises are not automatically awaited. Wrap async operations in a synchronous pattern or use `await` inside an async IIFE
- **Security**: Scripts execute with the page's origin permissions. Cross-origin restrictions apply
- **Timeout**: Long-running scripts may time out. Keep scripts short and efficient
