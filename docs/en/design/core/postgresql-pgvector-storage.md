+++
title = "ADR-003: PostgreSQL + PgVector for Unified Data Storage"
description = """Date: 2026-02"""
lang = "en"
category = "design"
subcategory = "core"
+++

# ADR-003: PostgreSQL + PgVector for Unified Data Storage

**Date**: 2026-02
**Status**: Accepted

## Context

Entelecheia requires a data storage layer that serves **two distinct responsibilities simultaneously**:

1. **Traditional relational data**: User accounts, API credentials, agent sessions, task metadata, audit logs, RBAC policies, container state, timeline events — all structured data that benefits from relational integrity, transactions, and SQL queries.

1. **Vector similarity search**: Embeddings for RAG (Retrieval-Augmented Generation), memory sedimentation, knowledge graph traversal, and document retrieval — data that requires high-dimensional nearest-neighbor search.

Several storage approaches were evaluated:

| Approach | Relational Data | Vector Search | Dual-Responsibility | Maturity |
| --- | --- | --- | --- | --- |
| **PostgreSQL + PgVector** | Full SQL, ACID transactions | HNSW/IVFFlat indexes | Single database, single query language | PostgreSQL: 35+ years; PgVector: stable, widely deployed |
| **Qdrant** | None (vector-only) | Excellent (purpose-built) | Requires separate relational DB | Moderate |
| **Milvus** | None (vector-only) | Excellent | Requires separate relational DB | Moderate |
| **Weaviate** | Limited (built-in CRUD) | Good | Compromised on both | Moderate |
| **SQLite + vector extension** | Full SQL | Experimental | Single file, limited concurrency | Low (vector extension immature) |
| **MongoDB + Atlas Vector Search** | Document store, no SQL | Good | Compromised query model | High (but proprietary vector search) |

## Decision

We chose **PostgreSQL with the PgVector extension** as the unified storage backend.

**Primary reasons:**

1. **Stability and proven reliability.** PostgreSQL is the most battle-tested open-source relational database engine, with 35+ years of development, a mature query optimizer, rock-solid ACID transactions, and a vast ecosystem of tooling. For a system that handles authentication credentials, RBAC policies, and audit logs, this stability is not optional — it is a baseline requirement. Choosing a newer, less-proven database for these workloads would introduce unnecessary operational risk.

1. **Unified dual-purpose storage.** PgVector extends PostgreSQL with vector similarity search (HNSW and IVFFlat indexing) without requiring a separate database. This means:

   - **One database to manage**, one connection pool, one backup strategy, one migration system.
   - **JOIN queries across relational and vector data** — e.g., "find documents similar to this embedding that the user has permission to access" can be expressed as a single SQL query.
   - **Transactional consistency** between metadata updates and embedding inserts.
   - **SQL as the universal query language** — no need for developers to learn a separate vector query DSL.

1. **PgVector is relatively stable for our scale.** While not as optimized as purpose-built vector databases for billion-scale deployments, PgVector handles the Entelecheia workload (agent memories, knowledge documents, RAG contexts) competently. The embedding dimensions (768-3072) and dataset sizes (thousands to low millions of vectors) are well within PgVector's comfort zone.

1. **Team familiarity and ecosystem.** PostgreSQL is the most widely deployed database engine in the world. The team has deep familiarity with SQL, PostgreSQL administration, and the Rust ORM ecosystem (SeaORM, SQLx). Choosing an unfamiliar vector database would require significant learning investment for marginal benefit.

1. **No compromise on SQL compatibility.** Many newer vector databases either don't support SQL at all or support a limited dialect. This would force the application to maintain two separate query models — one for relational data and one for vector search — increasing code complexity and the surface for bugs.

## Consequences

### Positive

- **Single operational surface**: One database to monitor, backup, upgrade, and debug. SeaORM migrations handle both schema and extension setup.
- **Transactional vector operations**: Embedding inserts and metadata updates happen in the same transaction, preventing orphaned or inconsistent data.
- **Full SQL power for combined queries**: Permissions-aware vector search, time-filtered similarity, and multi-table joins are native SQL operations.
- **SeaORM + PgVector integration**: The Rust ecosystem has mature PostgreSQL support. SeaORM entities can include vector columns with distance operators.
- **Production-hardened**: PostgreSQL's replication, point-in-time recovery, connection pooling (PgBouncer), and monitoring (`pg_stat_statements`) are industry-standard.

### Negative

- **Vector search performance ceiling**: For very large embedding collections (100M+ vectors), purpose-built vector databases (Qdrant, Milvus) significantly outperform PgVector. Entelecheia's current scale does not approach this limit, but it is a future consideration.
- **PgVector is an extension, not core**: PgVector must be installed and maintained separately from PostgreSQL. Container images must include the extension (we use `pgvector/pgvector:pg18`). Upgrading PostgreSQL may require extension recompilation.
- **Limited vector index types**: PgVector supports HNSW and IVFFlat. Purpose-built vector databases offer more specialized index types (e.g., DiskANN, ScaNN) that may be more efficient for specific distributions.
- **Resource competition**: Vector indexing (especially HNSW build) consumes CPU and memory that is shared with OLTP workloads on the same PostgreSQL instance. At scale, separating vector workloads to a dedicated replica may become necessary.

### Trade-off Accepted

**Performance ceiling for operational simplicity.** A dual-database architecture (PostgreSQL for relational + Qdrant/Milvus for vectors) would provide better vector search performance at scale. However, it would double the operational complexity, require data synchronization between two systems, and introduce consistency challenges. For the current and near-term scale of Entelecheia (single-user to small-team deployments), the unified PostgreSQL approach is the correct trade-off. If vector search becomes a bottleneck in the future, a read replica with PgVector or a dedicated vector cache layer can be introduced incrementally without changing the application's query model.
