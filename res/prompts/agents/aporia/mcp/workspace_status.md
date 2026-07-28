+++
name = "workspace_status"
agent = "aporia"

[description]
en = "Return current status of the workspace RAG index"
zh-Hans = "返回工作区RAG索引的当前状态"
zh-Hant = "返回工作區RAG索引的目前狀態"
ja = "ワークスペースRAGインデックスの現在のステータスを返す"
ko = "워크스페이스 RAG 인덱스의 현재 상태 반환"
fr = "Retourner l'état actuel de l'index RAG de l'espace de travail"
es = "Devolver el estado actual del índice RAG del espacio de trabajo"
ru = "Вернуть текущее состояние индекса RAG рабочего пространства"
+++

# workspace_status

## Description

Returns the current status of the workspace RAG index, including whether indexing has been performed, the number of indexed files and chunks, the last indexing timestamp, and the configured embedding model. Useful for verifying that the index is up-to-date before performing searches.

## Parameters

None.

## Returns

### On Success

```text
Workspace RAG index status

Status: <ready | not_indexed | indexing_in_progress>
Workspace root: <path>
Last indexed: <ISO 8601 timestamp or "never">
Embedding model: <model_name>

Statistics:
  Total files indexed: <number>
  Total chunks: <number>
  Index size: <size in MB>
  Supported file types: <list of extensions>
```

### On Failure

```text
Status check failed

Error: <error message>
```

## Examples

### Example 1: Indexed workspace

Invocation:

```text
workspace_status
```

Return:

```text
Workspace RAG index status

Status: ready
Workspace root: /home/user/project
Last indexed: 2024-03-10T14:22:00Z
Embedding model: text-embedding-3-small

Statistics:
  Total files indexed: 231
  Total chunks: 1892
  Index size: 12.4 MB
  Supported file types: .md, .txt, .py, .js, .ts, .json, .yaml, .toml
```

### Example 2: Not yet indexed

Invocation:

```text
workspace_status
```

Return:

```text
Workspace RAG index status

Status: not_indexed
Workspace root: (none)
Last indexed: never
Embedding model: text-embedding-3-small

Statistics:
  Total files indexed: 0
  Total chunks: 0
  Index size: 0 MB
```

## Important Notes

- **No parameters**: This tool takes no arguments. It always reports on the currently active workspace index.
- **Indexing status**: A status of `indexing_in_progress` means a `workspace_index` operation is currently running. Avoid starting another index operation simultaneously.
- **Health checks**: Use this tool to verify index readiness before running automated search pipelines.
