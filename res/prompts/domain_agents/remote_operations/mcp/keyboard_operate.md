+++
name = "keyboard_operate"
agent = "remote_operations"

[description]
en = "Perform keyboard operations on a remote device (HMI interaction)"
zh-Hans = "在远程设备上执行键盘操作（HMI交互）"
zh-Hant = "在遠程設備上執行鍵盤操作（HMI交互）"
ja = "リモートデバイスでキーボード操作（HMIインタラクション）"
ko = "원격 장치에서 키보드 조작(HMI 상호작용)"
fr = "Effectuer des opérations de clavier sur un appareil distant (interaction HMI)"
es = "Realizar operaciones de teclado en un dispositivo remoto (interacción HMI)"
ru = "Операции клавиатурой на удалённом устройстве (взаимодействие с HMI)"
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
