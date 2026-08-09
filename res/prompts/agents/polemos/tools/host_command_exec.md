+++
name = "host_command_exec"
agent = "polemos"

[description]
en = "Execute a command on the host machine via evernight IPC"
zh-Hans = "通过 evernight IPC 在宿主机执行命令"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `command` | string | yes | Command to execute |
| `cwd` | string | no | Working directory for the command |

## Returns

Returns command stdout, stderr, and exit code.
