+++
name = "navigate"
agent = "web_automation"

[description]
en = "Navigate the browser to a specified URL"
zhs = "将浏览器导航到指定URL"
zht = "將瀏覽器導航至指定URL"
ja = "ブラウザを指定したURLにナビゲートする"
ko = "브라우저를 지정된 URL로 이동"
fr = "Naviguer le navigateur vers une URL spécifiée"
es = "Navegar el navegador a una URL especificada"
ru = "Перейти в браузере по указанному URL"
+++

# navigate

## Description

Navigates a browser instance to the specified URL. Waits for the page to reach the requested load state before returning. Returns the final URL (after any redirects) and the page title.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier (obtained from `create`)
- **url** (string, required): The URL to navigate to (e.g., `"https://example.com"`)

## Returns

### Success

```text
Navigation completed

Browser ID: browser_abc123
Final URL: https://example.com
Title: Example Domain
Status: 200
```

### Failure

```text
Navigation failed

Browser ID: browser_abc123
Error: net::ERR_NAME_NOT_RESOLVED
URL: https://nonexistent.invalid
```

## Examples

### Example 1: Navigate to a webpage

```text
browser_id: "browser_abc123"
url: "https://example.com"
```

### Example 2: Navigate to an API endpoint

```text
browser_id: "browser_abc123"
url: "https://api.example.com/docs"
```

### Example 3: Navigate to a local dev server

```text
browser_id: "browser_abc123"
url: "http://localhost:3000"
```

## Important Notes

- **Redirects**: The returned URL reflects the final destination after any HTTP or JavaScript redirects
- **Relative URLs**: Always use absolute URLs including the scheme (`http://` or `https://`)
- **Invalid URLs**: Malformed or unreachable URLs return an error without changing the current page
- **Wait behavior**: Navigation waits for the `load` event by default. Pages with heavy async content may need additional waits via `execute_script`
