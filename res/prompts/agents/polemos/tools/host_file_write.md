+++
name = "host_file_write"
agent = "polemos"

[description]
en = "Write content to a file on the host machine via evernight IPC"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | string | yes | Absolute path of the file to write |
| `content` | string | yes | Content to write to the file |

## Returns

Returns `ok: true` on success.
