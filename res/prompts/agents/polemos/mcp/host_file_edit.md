+++
name = "host_file_edit"
agent = "polemos"

[description]
en = "Edit a file on the host machine via evernight IPC (find and replace)"
zh-Hans = "通过 evernight IPC 编辑宿主机文件（查找替换）"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | string | yes | Absolute path of the file to edit |
| `old_text` | string | yes | Text to find and replace |
| `new_text` | string | yes | Replacement text |

## Returns

Returns `ok: true` on success.
