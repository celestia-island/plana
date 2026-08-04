+++
name = "container_remove"

[description]
en = "Remove a container."
zh-Hans = "删除容器。"
zh-Hant = "刪除容器。"
ja = "コンテナを削除する。"
ko = "컨테이너를 제거합니다."
fr = "Supprimer un conteneur."
es = "Eliminar un contenedor."
ru = "Удаление контейнера."
+++

# container_remove

Permanently removes a Docker container and its filesystem from the host. This operation is irreversible — all data inside the container that is not stored on mounted volumes will be lost. The container should be stopped before removal; attempting to remove a running container may result in an error depending on the Docker daemon configuration.

## Parameters

- **`container_id`** (required): The unique identifier of the container to remove. This is the `container_id` returned by `container_create`, `container_list`, or `container_info`.

## Returns

### On Success

Returns a confirmation message:

- **`container_id`**: The identifier of the removed container.
- **status**: Confirmation of removal (e.g., `"removed"`).

### On Failure

Returns a JSON object with:

- **error**: A descriptive error message. The most common cause is referencing a container that does not exist (returns a `"not found"` error). Attempting to remove a running container may also fail.

## Examples

### Example 1: Remove a Stopped Container

Invocation:

```text
container_remove container_id="a1b2c3d4e5f6"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "status": "removed"
}
```

### Example 2: Attempt to Remove Non-existent Container

Invocation:

```text
container_remove container_id="does-not-exist"
```

Returns:

```json
{
  "error": "Container does-not-exist not found"
}
```

### Example 3: Full Lifecycle — Create, Start, Stop, Remove

Invocation sequence:

```text
container_create name="temp-worker" image="alpine:3.19"
# → container_id: "x1y2z3w4v5"

container_start container_id="x1y2z3w4v5"
# → status: "running"

container_stop container_id="x1y2z3w4v5"
# → status: "stopped"

container_remove container_id="x1y2z3w4v5"
```

Returns:

```json
{
  "container_id": "x1y2z3w4v5",
  "status": "removed"
}
```

## Important Notes

- This operation is **irreversible**. All container-local data (not on mounted volumes) will be permanently deleted.
- Always stop the container with `container_stop` before removing it to ensure clean shutdown.
- Data stored on mounted volumes (configured via the `volumes` parameter in `container_create`) persists after removal.
- Use `container_list` to verify the container ID before removal to avoid accidentally deleting the wrong container.
