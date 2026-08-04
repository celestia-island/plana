+++
name = "container_list"

[description]
en = "List all containers."
zh-Hans = "列出所有容器。"
zh-Hant = "列出所有容器。"
ja = "すべてのコンテナを一覧表示する。"
ko = "모든 컨테이너를 나열합니다."
fr = "Lister tous les conteneurs."
es = "Listar todos los contenedores."
ru = "Список всех контейнеров."
+++

# container_list

Returns a summary listing of all Docker containers managed by the neikos agent, regardless of their running status. Each entry includes the container name, base image, and current status. This is the discovery tool — use it to find container IDs before performing operations like start, stop, remove, or exec.

## Parameters

This tool takes no parameters.

## Returns

### On Success

Returns `{ ok: true, data: { total_count: number, containers: [{ name: string, image: string, status: string, id: string }] }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: List All Containers

Invocation:

```text
container_list
```

Returns:

```json
[
  {
    "name": "web-server",
    "image": "node:20",
    "status": "running"
  },
  {
    "name": "data-processor",
    "image": "python:3.12",
    "status": "stopped"
  },
  {
    "name": "redis-cache",
    "image": "redis:7",
    "status": "running"
  }
]
```

### Example 2: Empty Environment (No Containers)

Invocation:

```text
container_list
```

Returns:

```json
[]
```

### Example 3: Multiple Containers of the Same Image

Invocation:

```text
container_list
```

Returns:

```json
[
  {
    "name": "worker-1",
    "image": "alpine:3.19",
    "status": "running"
  },
  {
    "name": "worker-2",
    "image": "alpine:3.19",
    "status": "running"
  },
  {
    "name": "worker-3",
    "image": "alpine:3.19",
    "status": "stopped"
  }
]
```

## Important Notes

- The list includes containers in all states (created, running, stopped). Filter by `status` in your logic if you only need running containers.
- This tool does not return full container details — use `container_info` with a specific container ID for comprehensive information.
- An empty array `[]` indicates that no containers have been created in the current environment.
