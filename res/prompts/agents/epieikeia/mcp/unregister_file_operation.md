+++
name = "unregister_file_operation"
agent = "epieikeia"

[description]
en = "Unregister a previously registered file operation observer"
zhs = "取消注册已注册的文件操作观察者"
+++

## Parameters

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `file_path` | string | yes | The file path to unregister from |
| `agent_type` | string | yes | The agent type to unregister |

## Returns

Returns the tool result as a JSON object with `ok` (boolean) and `data` fields.
