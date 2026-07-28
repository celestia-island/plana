+++
name = "container_fork"

[description]
en = "Fork an existing container to create a copy"
zhs = "复制现有容器以创建副本"
zht = "複製現有容器以建立副本"
ja = "既存のコンテナをフォークしてコピーを作成"
ko = "기존 컨테이너를 포크하여 복사본 생성"
fr = "Forker un conteneur existant pour en créer une copie"
es = "Bifurcar un conteneur existante para crear una copia"
ru = "Форкнуть существующий контейнер для создания копии"
+++

# container_fork

Creates a derived copy of an existing container by forking it into a new container instance. The fork inherits the parent container's filesystem state and configuration, enabling branching workflows where multiple containers diverge from a shared base. Each fork tracks its branch level relative to the original container, forming a lineage tree. Optionally, a namespace volume can be mounted to isolate fork-specific data.

## Parameters

- **`container_id`** (required): The unique identifier of the parent container to fork from. This container serves as the base for the new derived container.
- **name** (optional): A suffix appended to the branch name. The final branch name follows the format `cosmos-<primary_binding_id_or_uuid8>-<name>`. If no binding ID exists, the first 8 characters of the container UUID are used.
- **`namespace_volume`** (optional): A host filesystem path to mount as a namespace volume inside the forked container. This provides isolated storage for the fork, keeping its data separate from the parent.

## Returns

### On Success

Returns `{ ok: true, data: { original_container: string, forked_container: string, forked_image: string, workspace_path: string, status: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Basic Fork with Auto-generated Name

Invocation:

```text
container_fork container_id="a1b2c3d4e5f6"
```

Returns:

```json
{
  "original_container_id": "a1b2c3d4e5f6",
  "new_container_id": "m3n4o5p6q7r8",
  "branch_level": 1
}
```

### Example 2: Fork with Custom Name and Namespace Volume

Invocation:

```text
container_fork container_id="a1b2c3d4e5f6" name="experiment-a" namespace_volume="/data/forks/experiment-a"
```

Returns:

```json
{
  "original_container_id": "a1b2c3d4e5f6",
  "new_container_id": "s9t0u1v2w3x4",
  "branch_level": 1,
  "branch_name": "cosmos-550e8400-experiment-a"
}
```

### Example 3: Fork of a Fork (Nested Branching)

Invocation:

```text
container_fork container_id="m3n4o5p6q7r8" name="sub-experiment"
```

Returns:

```json
{
  "original_container_id": "m3n4o5p6q7r8",
  "new_container_id": "y5z6a7b8c9d0",
  "branch_level": 2
}
```

## Important Notes

- Forking creates a new container — the parent container remains unmodified.
- The `branch_level` increments with each fork in the chain. Deep nesting may increase resource consumption.
- The `namespace_volume` parameter is useful for giving each fork its own isolated data directory, preventing data collisions between branches.
- Forked containers are created in a stopped state. Use `container_start` to bring them online.
- The fork inherits the parent's image, environment variables, port mappings, and network configuration. These cannot be overridden at fork time — modify them after creation using `exec` if needed.

## Branch Naming Convention

Forked containers receive a branch name that uses the container's **binding ID** (stable across Scepter restarts), not the runtime ID (`#xxx`).

Format: `cosmos-<primary_binding_id_or_uuid8>-<name>`

Priority:

1. If the container has a binding ID (e.g., `@github#123`), use it
1. Otherwise, fall back to the first 8 characters of the container UUID

Examples:

- `cosmos-@github#123-fix-auth-bug` — container bound to GitHub Issue #123
- `cosmos-@gitlab#234-feat-multimodal` — container bound to GitLab Issue #234
- `cosmos-550e8400-refactor-sync` — no external binding, UUID prefix used

Note: The runtime ID (`#xxx`) is NOT used in branch names because it is reassigned after Scepter restarts.
