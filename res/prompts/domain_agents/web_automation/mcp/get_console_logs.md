+++
name = "get_console_logs"
agent = "web_automation"

[description]
en = "Retrieve browser console logs"
zh-Hans = "检索浏览器控制台日志"
zh-Hant = "檢索瀏覽器主控台日誌"
ja = "ブラウザのコンソールログを取得する"
ko = "브라우저 콘솔 로그 조회"
fr = "Récupérer les journaux de la console du navigateur"
es = "Recuperar los registros de la consola del navegador"
ru = "Получить журналы консоли браузера"
+++

# get_console_logs

## Description

Retrieves all console messages logged by the browser page since the last navigation. Includes messages from `console.log`, `console.warn`, `console.error`, and `console.info`, as well as uncaught exceptions. Essential for debugging JavaScript errors and monitoring application behavior during automation.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier (obtained from `create`)

## Returns

### Success

```text
Console logs retrieved

Browser ID: browser_abc123
Count: 3

[LOG] Application started
[WARN] Deprecated API usage detected
[ERROR] Failed to fetch resource: /api/data (404)
```

### Failure

```text
Console log retrieval failed

Browser ID: browser_abc123
Error: Browser instance not found
```

## Examples

### Example 1: Check for errors after navigation

```text
browser_id: "browser_abc123"
```

### Example 2: Inspect logs after form submission

```text
browser_id: "browser_abc123"
```

### Example 3: Debug a failing page load

```text
browser_id: "browser_abc123"
```

## Important Notes

- **Log buffer**: Logs are buffered per navigation. Navigating to a new page clears the previous logs
- **Log levels**: Each entry includes its level (LOG, WARN, ERROR, INFO) for easy filtering
- **Exceptions**: Uncaught JavaScript exceptions appear as ERROR entries with stack traces
- **Timing**: Logs are captured in chronological order with timestamps
