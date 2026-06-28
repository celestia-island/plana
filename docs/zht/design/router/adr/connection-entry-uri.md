# 連線條目 URI 方案

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

連線目錄需要一種統一的方式來表示不同的協定連線（SSH、VNC、RDP、序列埠、Docker）。URI 方案提供了人類可讀且可序列化的描述符。

## 決策

使用協定特定的 URI 方案：

- `ssh://user@host:port`
- `vnc://host:5900`
- `rdp://host:3389`
- `serial:///dev/ttyUSB0?baud=9600`
- `docker:///var/run/docker.sock?container=name`

`ConnectionEntry` 將 URI 解析為帶有 scheme、host、port、username、path 和查詢參數的型別化結構體。目錄是一個包含 `ConnectionCategory` 節點的樹狀結構，節點中包含條目。

## 後果

### 正面

- 熟悉的 URI 格式；易於序列化；支援複製貼上分享連線。

### 負面

- 某些協定無法乾淨地對應到 URI（例如 Kubernetes 上下文）。
- 查詢字串參數是非結構化的。
