+++
name = "screenshot"
agent = "remote_operations"

[description]
en = "Capture a screenshot from a remote device display (HMI)"
zhs = "截取远程设备屏幕（HMI）"
zht = "截取遠程設備螢幕（HMI）"
ja = "リモートデバイスの画面（HMI）をキャプチャ"
ko = "원격 장치 화면(HMI) 캡처"
fr = "Capturer une capture d'écran d'un appareil distant (HMI)"
es = "Capturar pantalla de un dispositivo remoto (HMI)"
ru = "Захват экрана удалённого устройства (HMI)"
+++

# screenshot

截取远程工业设备（HMI/SCADA 操作面板）的当前屏幕画面。

## Parameters

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| remote_id | string | ✓ | - | 远程连接 ID |
| width | integer | | 1920 | 截图宽度 |
| height | integer | | 1080 | 截图高度 |

## Returns

```json
{
  "remote_id": "ssh-0192a3b4-...",
  "width": 1920,
  "height": 1080,
  "format": "png",
  "data_base64": "..."
}
```

## Examples

```json
{ "remote_id": "ssh-0192a3b4-..." }
```

## Notes

- 用于监控工业 HMI 面板状态
- 截图以 base64 编码的 PNG 格式返回
