+++
name = "web_fetch"
agent = "eleos"

[description]
en = "Fetch web page content from a specified URL"
zh-Hans = "从指定URL获取网页内容"
zh-Hant = "從指定URL擷取網頁內容"
ja = "指定したURLからWebページのコンテンツを取得"
ko = "지정된 URL에서 웹 페이지 콘텐츠 가져오기"
fr = "Récupérer le contenu d'une page web à partir d'une URL spécifiée"
es = "Obtener el contenido de una página web desde una URL especificada"
ru = "Получение содержимого веб-страницы по указанному URL"
+++

# web_fetch

Fetches the content of a web page from a given URL and returns the title, status code, response headers, and a content preview. This tool retrieves raw or parsed content from any publicly accessible HTTP or HTTPS endpoint.

## Parameters

- **url** (required, string): The URL to fetch. Must be a valid HTTP or HTTPS address. An empty URL triggers a failure.

## Returns

### On Success

```text
URL: <url>
Title: <page title>
Status: <HTTP status code>
Headers:
  <header-name>: <header-value>
  ...
Content:
  <page content preview>
```

### On Failure

```text
Fetch failed

URL: <url>
Error: <error message>
```

## Examples

### Example 1: Fetch a Documentation Page

```text
web_fetch
  url: "https://docs.rs/tokio/latest/tokio/"
```

Returns the page title, HTTP status (e.g. 200), response headers, and the rendered content of the Tokio documentation page.

### Example 2: Fetch an API Endpoint

```text
web_fetch
  url: "https://api.github.com/repos/rust-lang/rust"
```

Retrieves the JSON response from the GitHub API, including status code and headers. Useful for inspecting API responses.

### Example 3: Invalid URL

```text
web_fetch
  url: ""
```

Returns a failure response because the URL is empty:

```text
Fetch failed

URL:
Error: URL cannot be empty
```

## Important Notes

- The URL must not be empty; an empty string will always produce a failure.
- Some websites employ anti-bot measures (CAPTCHAs, rate limiting) that may cause fetch failures or return blocked-content pages.
- The content preview may be truncated for very large pages. For full content, consider fetching specific sub-pages.
- Only publicly accessible URLs can be fetched. Authenticated or private endpoints are not supported.
