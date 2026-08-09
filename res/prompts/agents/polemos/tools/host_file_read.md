+++
name = "host_file_read"
agent = "polemos"

[description]
en = "Read a file from the host machine via evernight IPC"
zh-Hans = "通过 evernight IPC 读取宿主机文件"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | string | yes | Absolute path of the file to read |

## Returns

Returns the file content as a string.
