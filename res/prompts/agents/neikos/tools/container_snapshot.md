+++
name = "container_snapshot"

[description]
en = "Create a snapshot of a container"
zh-Hans = "创建容器快照"
zh-Hant = "建立容器快照"
ja = "コンテナのスナップショットを作成"
ko = "컨테이너 스냅샷 생성"
fr = "Créer un instantané d'un conteneur"
es = "Crear una instantánea de un contenedor"
ru = "Создать снимок контейнера"
+++

# container_snapshot

Creates a point-in-time snapshot of a container's filesystem and configuration state. Snapshots capture the full container state at the moment of invocation, enabling rollback, cloning, or archival workflows. The container does not need to be stopped to take a snapshot, though stopping it first ensures a consistent state.

## Parameters

- **`container_id`** (required): The unique identifier of the container to snapshot. This is the `container_id` returned by `container_create`, `container_list`, or `container_info`.

## Returns

### On Success

Returns a JSON object with the snapshot details:

- **`snapshot_id`**: The unique identifier assigned to the newly created snapshot.
- **`container_id`**: The identifier of the source container.

### On Failure

Returns a JSON object with:

- **error**: A descriptive error message. Common causes include the container not existing or insufficient disk space on the host.

## Examples

### Example 1: Snapshot a Running Container

Invocation:

```text
container_snapshot container_id="a1b2c3d4e5f6"
```

Returns:

```json
{
  "snapshot_id": "snap-abc123def456",
  "container_id": "a1b2c3d4e5f6"
}
```

### Example 2: Snapshot Before a Risky Operation

Invocation:

```text
container_snapshot container_id="f7e8d9c0b1a2"
```

Returns:

```json
{
  "snapshot_id": "snap-789ghi012jkl",
  "container_id": "f7e8d9c0b1a2"
}
```

### Example 3: Snapshot Non-existent Container

Invocation:

```text
container_snapshot container_id="does-not-exist"
```

Returns:

```json
{
  "error": "Container does-not-exist not found"
}
```

## Important Notes

- For the most consistent snapshot, stop the container before snapshotting. Snapshots of running containers may capture an inconsistent filesystem state.
- Snapshots consume disk space proportional to the container's filesystem size. Monitor host storage when creating multiple snapshots.
- The returned `snapshot_id` can be used for rollback or cloning operations in downstream tooling.
- Snapshotting does not modify or interrupt the source container.
