+++
name = "mouse_operate"
agent = "remote_operations"

[description]
en = "Perform mouse operations on a remote device (HMI interaction)"
+++

# mouse_operate

向远程工业设备的 HMI 面板发送鼠标操作指令。

## Parameters

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| remote_id | string | ✓ | - | 远程连接 ID |
| action | string | ✓ | "click" | 操作类型: click, double_click, down, up, move, scroll, drag |
| x | integer | | 0 | X 坐标 |
| y | integer | | 0 | Y 坐标 |
| button | string | | "left" | 鼠标按键: left, right, middle |

## Returns

```json
{
  "remote_id": "ssh-0192a3b4-...",
  "action": "click",
  "x": 500,
  "y": 300,
  "button": "left",
  "success": true
}
```

## Examples

```json
{ "remote_id": "ssh-0192a3b4-...", "action": "click", "x": 500, "y": 300 }
```

## Notes

- 常用于操作工业触摸屏 HMI 面板
- 坐标系以屏幕左上角为原点
