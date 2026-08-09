+++
name = "connect_remote_via_ssh"
agent = "remote_operations"

[description]
en = "Connect to a remote device via SSH"
zh-Hans = "通过 SSH 连接远程设备"
zh-Hant = "通過 SSH 連接遠程設備"
ja = "SSH経由でリモートデバイスに接続"
ko = "SSH를 통해 원격 장치에 연결"
fr = "Se connecter à un appareil distant via SSH"
es = "Conectar a un dispositivo remoto vía SSH"
ru = "Подключение к удалённому устройству через SSH"
+++

# connect_remote_via_ssh

建立到远程设备的 SSH 连接，注册到 SkeMma 的连接管理器。

## Parameters

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| host | string | ✓ | - | 目标主机地址 |
| port | integer | | 22 | SSH 端口 |
| username | string | | "root" | 登录用户名 |

## Returns

```json
{
  "id": "ssh-0192a3b4-...",
  "host": "198.51.100.100",
  "port": 22,
  "protocol": "ssh",
  "connected": true,
  "message": "SSH connection registered"
}
```

## Examples

```json
{ "host": "198.51.100.100", "port": 22, "username": "admin" }
```

## Notes

- 连接建立后可通过 `exec_on_remote`、`screenshot` 等工具操作
- 返回的 `id` 用于后续工具调用的 `remote_id` 参数
