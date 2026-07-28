+++
name = "rag_db_delete"
agent = "aporia"

[description]
en = "Delete specified knowledge items from the RAG database"
zh-Hans = "从RAG数据库中删除指定的知识条目"
zh-Hant = "從RAG資料庫中刪除指定的知識項目"
ja = "RAGデータベースから指定されたナレッジアイテムを削除する"
ko = "RAG 데이터베이스에서 지정된 지식 항목 삭제"
fr = "Supprimer les éléments de connaissances spécifiés de la base de données RAG"
es = "Eliminar elementos de conocimiento especificados de la base de datos RAG"
ru = "Удалить указанные элементы знаний из базы данных RAG"
+++

# rag_db_delete

Delete a specific knowledge entry from the RAG database by its document ID.

## Description

The `rag_db_delete` tool removes a single document from the RAG database using its unique ID. This is used for knowledge base maintenance — removing outdated, incorrect, or sensitive content. Deletion is permanent and cannot be undone.

## Parameters

- **id** (required, string): The unique document ID to delete. Must not be empty — an empty string triggers a failure.

## Returns

### On Success

```text
Operation successful

id: "123e4567-e89b-12d3-a456-426614174000"
```

### On Failure

```text
Operation failed

Error: Document not found
```

Possible causes:

- `id` is an empty string
- No document exists with the specified ID

## Use Cases

- **Knowledge Base Maintenance**: Remove incorrect or low-quality data from the database.
- **Expired Content Cleanup**: Delete outdated documents that are no longer relevant.
- **Data Compliance**: Remove sensitive or regulated data to meet compliance requirements.

## Examples

### Example 1: Delete an existing document

Invocation:

```text
rag_db_delete
  id: "123e4567-e89b-12d3-a456-426614174000"
```

Return:

```text
Operation successful

id: "123e4567-e89b-12d3-a456-426614174000"
```

Description: The document with the given ID is permanently removed from the RAG database.

### Example 2: Delete a non-existent document

Invocation:

```text
rag_db_delete
  id: "00000000-0000-0000-0000-000000000000"
```

Return:

```text
Operation failed

Error: Document not found
```

Description: Attempting to delete a document that does not exist results in a failure response.

### Example 3: Failure due to empty ID

Invocation:

```text
rag_db_delete
  id: ""
```

Return:

```text
Operation failed

Error: id cannot be empty
```

Description: An empty ID string is always rejected. Provide a valid document ID obtained from `rag_db_write` or `rag_db_read`.

## Important Notes

- **Permanent Deletion**: Deleting a document is irreversible. If you need the content later, ensure you have a backup outside the RAG database.
- **ID Required**: The tool only supports deletion by exact document ID. There is no batch delete or filter-based deletion.
- **ID Source**: Use the `id` returned by `rag_db_write` or found in `rag_db_read` results to identify the document to delete.
