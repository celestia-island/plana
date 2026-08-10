+++
name = "keyboard_operate"
agent = "remote_operations"

[description]
en = "Perform keyboard operations on a remote device (HMI interaction)"
+++

# keyboard_operate

向远程工业设备的 HMI 面板发送键盘操作指令。

## Parameters

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| remote_id | string | ✓ | - | 远程连接 ID |
| action | string | | "type" | 操作类型: type, press, down, up, combo |
| keys | string[] | ✓ | - | 按键序列，如 ["Ctrl", "C"] 或 ["Enter"] |

## Returns

```json
{
  "remote_id": "ssh-0192a3b4-...",
  "action": "type",
  "keys": ["H", "e", "l", "l", "o"],
  "success": true
}
```

## Examples

```json
{ "remote_id": "ssh-0192a3b4-...", "action": "combo", "keys": ["Ctrl", "S"] }
```

## Notes

- `type`：依次输入字符序列
- `combo`：组合键（如 Ctrl+C）
- `press`/`down`/`up`：单键按下与释放
