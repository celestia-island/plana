+++
name = "record"
agent = "web_automation"

[description]
en = "Record a video of the browser session"
zh-Hans = "录制浏览器会话视频"
zh-Hant = "錄製瀏覽器工作階段影片"
ja = "ブラウザセッションの動画を録画する"
ko = "브라우저 세션 비디오 녹화"
fr = "Enregistrer une vidéo de la session de navigation"
es = "Grabar un video de la sesión del navegador"
ru = "Записать видео сеанса браузера"
+++

# record

## Description

Starts or stops a video recording of the browser viewport. Call with `action: "start"` to begin recording, then perform your automated actions, and call again with `action: "stop"` to finalize and save the video. Useful for creating visual test evidence, reproducing bug reports, and demo recordings.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier (obtained from `create`)
- **action** (string, required): The recording action to perform. Must be `"start"` or `"stop"`

## Returns

### Start success

```text
Recording started

Browser ID: browser_abc123
Status: recording
```

### Stop success

```text
Recording stopped

Browser ID: browser_abc123
File: /tmp/recordings/browser_abc123_20240115.webm
Duration: 12.5s
```

### Failure

```text
Recording failed

Browser ID: browser_abc123
Error: Already recording
```

## Examples

### Example 1: Start recording

```text
browser_id: "browser_abc123"
action: "start"
```

### Example 2: Stop recording

```text
browser_id: "browser_abc123"
action: "stop"
```

## Important Notes

- **Action values**: Only `"start"` and `"stop"` are valid. Calling start on an already-recording browser returns an error
- **Stop before close**: Always stop recording before calling `close`. Closing the browser without stopping discards the recording
- **Video format**: Recordings are saved in WebM format by default
- **Performance**: Recording has minimal impact on automation performance but consumes disk space proportional to recording duration
