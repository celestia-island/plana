+++
name = "container_create"

[description]
en = "Create a new container."
zh-Hans = "创建新容器。"
zh-Hant = "建立新容器。"
ja = "新しいコンテナを作成する。"
ko = "새 컨테이너를 생성합니다."
fr = "Créer un nouveau conteneur."
es = "Crear un nuevo contenedor."
ru = "Создание нового контейнера."
+++

# container_create

Creates a new Docker container from a specified image with configurable environment variables, port mappings, volume mounts, and network settings. The container is created but not started — use `container_start` to bring it online after creation. This is the primary tool for provisioning isolated workspaces managed by the neikos agent.

## Parameters

- **name** (required): The container name. Must be unique within the Docker host. Used for identification and referencing in subsequent operations.
- **image** (required): The Docker image to use as the base (e.g., `"ubuntu:22.04"`, `"node:20"`). Must not be empty — an empty string triggers an immediate failure.
- **env** (optional): Environment variables to set inside the container, provided as a key-value string map. For example: `{"NODE_ENV": "production", "PORT": "3000"}`. Defaults to an empty object.
- **ports** (optional): Port mappings between the host and container, expressed as an array of strings in `"HOST_PORT:CONTAINER_PORT"` format. For example: `["8080:80", "443:443"]`. Defaults to an empty array.
- **volumes** (optional): Volume mappings for persistent storage, each specified as an object with the following fields:
  - **`host_path`** (required): The absolute path on the host machine.
  - **`container_path`** (required): The absolute path inside the container.
  - **`read_only`** (optional, default `false`): Whether the container has read-only access to the volume.
- **network** (optional): The Docker network to attach the container to. Defaults to `"entelecheia-network"`.

## Returns

### On Success

Returns a JSON object confirming the created container:

- **`container_id`**: The unique Docker container identifier.
- **image**: The image used to create the container.
- **name**: The assigned container name.
- **network**: The network the container is connected to.
- **status**: The initial status (typically `"created"`).

### On Failure

Returns a JSON object with:

- **error**: A descriptive error message. The most common failure is passing an empty `image` parameter.

## Examples

### Example 1: Minimal Container Creation

Invocation:

```text
container_create name="my-app" image="ubuntu:22.04"
```

Returns:

```json
{
  "container_id": "a1b2c3d4e5f6",
  "image": "ubuntu:22.04",
  "name": "my-app",
  "network": "entelecheia-network",
  "status": "created"
}
```

### Example 2: Container with Environment Variables and Port Mapping

Invocation:

```json
container_create name="web-server" image="node:20" env={"PORT":"3000","NODE_ENV":"production"} ports=["3000:3000"]
```

Returns:

```json
{
  "container_id": "f7e8d9c0b1a2",
  "image": "node:20",
  "name": "web-server",
  "network": "entelecheia-network",
  "status": "created"
}
```

### Example 3: Container with Volume Mounts and Custom Network

Invocation:

```json
container_create name="data-processor" image="python:3.12" volumes=[{"host_path":"/data/input","container_path":"/input"},{"host_path":"/data/output","container_path":"/output","read_only":false}] network="custom-bridge"
```

Returns:

```json
{
  "container_id": "3k4l5m6n7o8p",
  "image": "python:3.12",
  "name": "data-processor",
  "network": "custom-bridge",
  "status": "created"
}
```

## Important Notes

- The container is created in a stopped state. Call `container_start` with the returned `container_id` to start it.
- The `image` parameter must be non-empty. Passing an empty string is the most common cause of creation failure.
- Volume `host_path` values should be absolute paths to ensure consistent behavior across environments.
- Containers default to the `"entelecheia-network"` network. Only override the `network` parameter if you need isolation on a separate Docker network.
