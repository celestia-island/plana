+++
name = "host_file_read"
agent = "polemos"

[description]
en = "Read a file from the host machine via evernight IPC"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | string | yes | Absolute path of the file to read |

## Returns

Returns the file content as a string.
