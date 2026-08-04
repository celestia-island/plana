+++
name = "container_start"

[description]
en = "Start a stopped container."
zh-Hans = "启动已停止的容器。"
zh-Hant = "啟動已停止的容器。"
ja = "停止したコンテナを起動する。"
ko = "중지된 컨테이너를 시작합니다."
fr = "Démarrer un conteneur arrêté."
es = "Iniciar un contenedor detenido."
ru = "Запуск остановленного контейнера."
+++

# container_start

Starts a previously created or stopped Docker container, bringing it to a running state. The container must exist and be in a non-running state. Use `container_info` to verify the current status before attempting to start.

## Parameters

- **`container_id`** (required): The unique identifier of the container to start. This is the `container_id` returned by `container_create` or `container_list`.

## Returns

### On Success

Returns `{ ok: true, data: { container_id: string, status: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Start a Recently Created Container

Invocation:

```text
container_start container_id="a1b2c3d4e5f6"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "status": "running"
}
```

### Example 2: Start After Stop (Restart)

Invocation:

```text
container_start container_id="f7e8d9c0b1a2"
```

Returns:

```json
{
  "container_id": "f7e8d9c0b1a2",
  "status": "running"
}
```

### Example 3: Attempt to Start Non-existent Container

Invocation:

```text
container_start container_id="does-not-exist"
```

Returns:

```json
{
  "error": "Container does-not-exist not found"
}
```

## Important Notes

- Ensure the container has been created (via `container_create`) before starting it.
- Starting an already-running container may return an error or be a no-op depending on the Docker daemon behavior.
- If the container fails to start, check the container logs via `exec` or Docker CLI for diagnostic information.
