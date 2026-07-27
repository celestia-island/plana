+++
name = "container_stop"

[description]
en = "Stop a running container."
zhs = "停止运行中的容器。"
zht = "停止運行中的容器。"
ja = "実行中のコンテナを停止する。"
ko = "실행 중인 컨테이너를 중지합니다."
fr = "Arrêter un conteneur en cours d'exécution."
es = "Detener un contenedor en ejecución."
ru = "Остановка запущенного контейнера."
+++

# container_stop

Gracefully stops a running Docker container, sending a SIGTERM signal followed by a SIGKILL after the grace period expires. The container is not removed — it can be restarted with `container_start` or permanently deleted with `container_remove`. Use this tool to safely shut down containers without losing their filesystem state.

## Parameters

- **`container_id`** (required): The unique identifier of the container to stop. This is the `container_id` returned by `container_create`, `container_list`, or `container_info`.

## Returns

### On Success

Returns `{ ok: true, data: { container_id: string, status: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Stop a Running Container

Invocation:

```text
container_stop container_id="a1b2c3d4e5f6"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "status": "stopped"
}
```

### Example 2: Stop Before Removal

Invocation:

```text
container_stop container_id="f7e8d9c0b1a2"
```

Returns:

```json
{
  "container_id": "f7e8d9c0b1a2",
  "status": "stopped"
}
```

### Example 3: Attempt to Stop Already-Stopped Container

Invocation:

```text
container_stop container_id="a1b2c3d4e5f6"
```

Returns:

```json
{
  "error": "Container a1b2c3d4e5f6 is not running"
}
```

## Important Notes

- The container is stopped but not removed. Use `container_remove` to permanently delete it.
- Stopped containers retain their filesystem and configuration. They can be restarted with `container_start`.
- Always stop a container before removing it to ensure clean shutdown of internal processes.
