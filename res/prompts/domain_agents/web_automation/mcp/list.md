> **Note:** This tool is planned but not yet implemented — no MCP registration exists in the current runtime.

+++
name = "list"
agent = "`web_automation`"

[description]
en = "List all active browser instances"
zh-Hans = "列出所有活动浏览器实例"
zh-Hant = "列出所有活動瀏覽器實例"
ja = "すべてのアクティブなブラウザインスタンスを一覧表示する"
ko = "모든 활성 브라우저 인스턴스 나열"
fr = "Lister toutes les instances de navigateur actives"
es = "Listar todas las instancias de navegador activas"
ru = "Список всех активных экземпляров браузера"
+++

# list

## Description

Lists all currently active browser instances, including their IDs, browser type, and headless status. Use this to track which browsers are open, verify cleanup, or find a browser ID for subsequent operations.

## Parameters

None.

## Returns

### Success

```text
Active browser instances

Count: 2

ID: browser_abc123  Type: chromium  Headless: true
ID: browser_xyz789  Type: chromium  Headless: false
```

### Empty

```text
No active browser instances
```

## Examples

### Example 1: List all browsers

```text
(no parameters needed)
```

## Important Notes

- **No parameters**: This tool takes no arguments and inspects the global browser session state
- **Resource awareness**: Use this to verify that browsers were properly closed and are not leaking resources
- **Cross-session**: Only browsers created within the current session are listed
