+++
name = "exec_on_remote"
agent = "remote_operations"

[description]
en = "Execute a command on a remote device"
zh-Hans = "在远程设备上执行命令"
zh-Hant = "在遠程設備上執行命令"
ja = "リモートデバイスでコマンドを実行"
ko = "원격 장치에서 명령 실행"
fr = "Exécuter une commande sur un appareil distant"
es = "Ejecutar un comando en un dispositivo remoto"
ru = "Выполнить команду на удалённом устройстве"
+++

# exec_on_remote

通过已建立的远程连接在目标设备上执行 shell 命令。

## Parameters

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| remote_id | string | ✓ | 远程连接 ID（由 connect_remote_via_ssh 返回） |
| command | string | ✓ | 要执行的命令 |

## Returns

```json
{
  "remote_id": "ssh-0192a3b4-...",
  "command": "ls /tmp",
  "exit_code": 0,
  "stdout": "file1\nfile2",
  "stderr": "",
  "duration_ms": 150
}
```

## Examples

```json
{ "remote_id": "ssh-0192a3b4-...", "command": "cat /etc/os-release" }
```

## Notes

- 必须先通过 `connect_remote_via_ssh` 建立连接
- 命令在远程设备的默认 shell 中执行
