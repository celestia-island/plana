+++
name = "rag_db_write"
agent = "aporia"

[description]
en = "Write text blocks and their vector embeddings to the RAG database"
+++

# rag_db_write

Write a text block and its embedding into ApoRia's current in-memory vector-document store.

## Parameters

- **content** (required, string, separate-call): The document content text to store. Provide via `rag_db_write.content("...")` in a follow-up call. Must not be empty — an empty string triggers a failure.
- **embedding** (optional, array of number): The embedding vector for the content, as an array of float32 values. If omitted, the tool stores a zero vector of 1536 dimensions.

## Returns

### On Success

```text
Operation successful

status: "success"
id: "123e4567-e89b-12d3-a456-426614174000"
embedding_dimensions: 1536
content: "The stored document text..."
```

### On Failure

```text
Operation failed

Error: Content cannot be empty
```

Possible causes:

- `content` is an empty string
- `embedding` is an empty array

## Use Cases

- **Knowledge Ingestion**: Store processed information for later retrieval.
- **Document Indexing**: Add new documents to ApoRia's in-memory vector store.
- **Knowledge Base Updates**: Incrementally append or update existing knowledge entries.

## Examples

### Example 1: Write a document with embedding

Invocation:

```text
rag_db_write
  content: "Project Alpha utilizes advanced semantic search capabilities to enhance information retrieval."
  embedding: [0.012, -0.034, 0.056, ..., 0.078]
```

Return:

```text
Operation successful

status: "success"
id: "123e4567-e89b-12d3-a456-426614174000"
embedding_dimensions: 1536
content: "Project Alpha utilizes advanced semantic search capabilities to enhance information retrieval."
```

Description: A document is written with its embedding vector. The returned `id` can be used later with `rag_db_read` or `rag_db_delete`.

### Example 2: Write with default zero embedding

Invocation:

```text
rag_db_write
  content: "The RAG system combines retrieval and generation to provide accurate responses."
```

Return:

```text
Operation successful

status: "success"
id: "234f5678-e89b-12d3-a456-426614174001"
embedding_dimensions: 1536
content: "The RAG system combines retrieval and generation to provide accurate responses."
```

Description: When no embedding is provided, a zero vector of 1536 dimensions is used. This is useful when the embedding will be computed and updated separately.

### Example 3: Failure due to empty content

Invocation:

```text
rag_db_write
  content: ""
  embedding: [0.1, 0.2, 0.3]
```

Return:

```text
Operation failed

Error: Content cannot be empty
```

Description: The tool rejects writes with empty content to prevent blank entries in the knowledge base.

## Important Notes

- **Storage Model**: The current implementation appends documents to an in-memory vector-document list.
- **Embedding Dimensions**: The default path uses a zero vector of 1536 dimensions when no embedding is supplied.
- **Content Required**: An empty `content` string is always rejected. Ensure meaningful text is provided.
- **ID Generation**: A unique document ID (UUID) is automatically generated and returned on success. Store this ID if you need to reference or delete the document later.
- **Zero Vector Default**: If no embedding is supplied, a zero vector is stored. You may need to update it later with an actual embedding for meaningful similarity search.
