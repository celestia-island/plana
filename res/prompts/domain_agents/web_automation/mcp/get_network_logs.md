+++
name = "get_network_logs"
agent = "web_automation"

[description]
en = "Retrieve browser network request logs"
zh-Hans = "检索浏览器网络请求日志"
zh-Hant = "檢索瀏覽器網路請求日誌"
ja = "ブラウザのネットワークリクエストログを取得する"
ko = "브라우저 네트워크 요청 로그 조회"
fr = "Récupérer les journaux des requêtes réseau du navigateur"
es = "Recuperar los registros de solicitudes de red del navegador"
ru = "Получить журналы сетевых запросов браузера"
+++

# get_network_logs

## Description

Retrieves a log of all HTTP network requests made by the browser page since the last navigation. Each entry includes the request method, URL, status code, and content type. Use this to verify API calls, debug network issues, and inspect XHR/fetch interactions during automation.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier (obtained from `create`)

## Returns

### Success

```text
Network logs retrieved

Browser ID: browser_abc123
Count: 5

GET  https://example.com/             200  text/html
GET  https://example.com/style.css    200  text/css
GET  https://example.com/app.js       200  application/javascript
GET  https://api.example.com/data     200  application/json
POST https://api.example.com/submit   201  application/json
```

### Failure

```text
Network log retrieval failed

Browser ID: browser_abc123
Error: Browser instance not found
```

## Examples

### Example 1: Verify API calls after page load

```text
browser_id: "browser_abc123"
```

### Example 2: Inspect network after button click

```text
browser_id: "browser_abc123"
```

### Example 3: Debug failed resource loading

```text
browser_id: "browser_abc123"
```

## Important Notes

- **Log buffer**: Network logs are captured per navigation. Navigating to a new page clears the previous log
- **Request details**: Each entry includes method, URL, HTTP status code, and content type
- **Failed requests**: Requests that failed to complete (DNS errors, timeouts) appear with an error status instead of an HTTP code
- **WebSocket**: WebSocket upgrade requests are logged, but individual frame data is not included
