+++
name = "workspace_index"
agent = "aporia"

[description]
en = "Scan and index workspace files into the RAG vector database for semantic retrieval"
zhs = "扫描工作区文件并索引到RAG向量数据库以支持语义检索"
zht = "掃描工作區檔案並索引至RAG向量資料庫以支援語義檢索"
ja = "ワークスペースファイルをスキャンし、RAGベクトルデータベースにインデックスして意味検索を可能にする"
ko = "워크스페이스 파일을 스캔하여 RAG 벡터 데이터베이스에 인덱싱하여 의미 검색 지원"
fr = "Analyser et indexer les fichiers de l'espace de travail dans la base vectorielle RAG"
es = "Escanear e indexar archivos del espacio de trabajo en la base de datos vectorial RAG"
ru = "Сканировать и индексировать файлы рабочего пространства в векторную базу данных RAG"
+++

# workspace_index

## Description

Traverses the workspace directory tree, reads supported file types (source code, markdown, configuration files), generates embeddings for each document chunk, and stores them in the RAG vector database. Supports incremental updates and full rebuilds. After indexing, the files become searchable via `workspace_search`.

## Parameters

- **`workspace_root`** (string, required): Absolute or relative path to the workspace root directory to index.
- **`full_rebuild`** (boolean, optional): If `true`, clears existing index and re-indexes all files from scratch. If `false`, performs incremental indexing of changed files only. Defaults to `true`.

## Returns

### On Success

```text
Workspace indexing complete

Workspace root: <path>
Mode: <full_rebuild | incremental>
Files scanned: <number>
Files indexed: <number>
Chunks created: <number>
Skipped (unsupported): <number>
Errors: <number>
Duration: <seconds>s
```

### On Failure

```text
Workspace indexing failed

Error: <error message>
```

## Examples

### Example 1: Full rebuild

Invocation:

```text
workspace_index
  workspace_root: "/home/user/project"
  full_rebuild: true
```

Return:

```text
Workspace indexing complete

Workspace root: /home/user/project
Mode: full_rebuild
Files scanned: 247
Files indexed: 231
Chunks created: 1892
Skipped (unsupported): 16
Errors: 0
Duration: 34.2s
```

### Example 2: Incremental update

Invocation:

```text
workspace_index
  workspace_root: "/home/user/project"
  full_rebuild: false
```

Return:

```text
Workspace indexing complete

Workspace root: /home/user/project
Mode: incremental
Files scanned: 12
Files indexed: 8
Chunks created: 54
Skipped (unsupported): 0
Errors: 0
Duration: 2.1s
```

## Important Notes

- **Supported file types**: Typically includes `.md`, `.txt`, `.py`, `.js`, `.ts`, `.json`, `.yaml`, `.toml`, and other text-based formats. Binary files are skipped.
- **Full rebuild cost**: A full rebuild re-processes every file. For large workspaces, prefer incremental mode after the initial index.
- **Chunking strategy**: Files are split into overlapping chunks of ~500 tokens with 50-token overlap for optimal retrieval granularity.
- **Embedding model**: Uses the same embedding model configured for the RAG subsystem. Changing the model requires a full rebuild.
- **Concurrency**: Indexing locks the workspace index. Do not run multiple indexing operations simultaneously on the same workspace.
