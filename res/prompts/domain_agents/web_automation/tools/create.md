+++
name = "create"
agent = "web_automation"

[description]
en = "Create a new browser instance for automated testing"
+++

# create

## Description

Creates a new browser instance for automated testing and web interaction. Returns a `browser_id` that is used with all other browser tools. Supports headless mode, custom window sizes, user agents, and proxy configuration.

## Parameters

- **headless** (boolean, optional): Run in headless mode (no visible window). Default: `true`
- **`window_size`** (string, optional): Browser window dimensions in `WIDTHxHEIGHT` format (e.g., `"1920x1080"`). Default: `"1280x720"`
- **`user_agent`** (string, optional): Custom User-Agent string for HTTP requests
- **proxy** (string, optional): Proxy server address (e.g., `"http://proxy:8080"`, `"socks5://proxy:1080"`)
- **`browser_type`** (string, optional): Browser engine to use: `chromium`, `firefox`, `webkit`. Default: `"chromium"`
- **`ignore_https_errors`** (boolean, optional): Ignore HTTPS certificate errors. Default: `false`
- **`slow_mo`** (integer, optional): Slow down each browser operation by the specified milliseconds. Useful for debugging. Default: `0`

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Browser instance created

Browser ID: browser_abc123
Type: chromium
Headless: true
Window size: 1920x1080
```

### Failure

```text
Browser creation failed

Error: Browser engine not available
Type: firefox
Message: The Firefox browser engine is not installed.
```

## Examples

### Example 1: Headless browser for scraping

```text
headless: true
window_size: "1920x1080"
```

### Example 2: Visible browser for debugging

```text
headless: false
browser_type: "chromium"
slow_mo: 100
```

### Example 3: Browser with proxy

```text
headless: true
proxy: "http://proxy.example.com:8080"
user_agent: "Mozilla/5.0 (compatible; TestBot/1.0)"
```

## Important Notes

- **Resource limits**: Each browser instance consumes significant memory (typically 100-300 MB). Avoid creating more instances than needed
- **Browser ID**: The returned `browser_id` must be passed to all subsequent browser operations. Store it for the lifetime of the session
- **Cleanup**: Always call `close` when done. Orphaned browsers continue consuming resources
- **Headless mode**: Headless is recommended for automated tasks. Use visible mode only for debugging
- **Proxy support**: SOCKS5 and HTTP proxies are supported. Authentication is passed in the URL: `http://user:pass@proxy:8080`
