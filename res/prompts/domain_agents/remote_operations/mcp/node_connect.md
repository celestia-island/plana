+++
name = "node_connect"
agent = "remote_operations"

[description]
en = "Connect to a specified node."
zh-Hans = "连接到指定节点。"
zh-Hant = "連線到指定節點。"
ja = "指定されたノードに接続する。"
ko = "지정된 노드에 연결합니다."
fr = "Se connecter à un nœud spécifié."
es = "Conectarse a un nodo especificado."
ru = "Подключение к указанному узлу."
+++

# node_connect

## Description

Establishes a connection to a remote node identified by its node ID and network address. Once connected, subsequent operations such as command execution and file transfer can be performed on the target node. Supports customizable port configuration and credential-based authentication.

## Parameters

- **`node_id`** (string, required): Unique identifier for the target node. Used to reference the node in subsequent operations
- **address** (string, required): Network address of the node (hostname or IP address)
- **port** (number, optional): Network port to connect on. Default: `22`
- **credentials** (object, optional): Authentication credentials for the node. Supported fields depend on the node's authentication method (e.g. `username`, `password`, `key_path`). Default: none

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Connected to node successfully

node_id: "node-001"
address: "198.51.100.50"
port: 22
status: "connected"
os: "Ubuntu 22.04"
uptime: "15 days, 3:20"
connected_at: "2024-01-15T10:30:00.000Z"
```

### Failure

```text
Connection failed

node_id: "node-001"
address: "198.51.100.50"
port: 22
Error: Connection refused
Message: No service is listening on the specified address and port.
```

## Examples

### Example 1: Basic SSH connection

```text
node_id: "node-001"
address: "198.51.100.50"
```

### Example 2: Custom port with credentials

```text
node_id: "node-002"
address: "192.0.2.100"
port: 2222
credentials: r#"{username: admin, key_path: /home/user/.ssh/id_rsa}"#
```

### Example 3: Reconnect to a known node

```text
node_id: "prod-server-03"
address: "prod.example.com"
port: 22
credentials: r#"{username: deploy, password: ***}"#
```

## Important Notes

- **Connection lifecycle**: Connections are managed by the PoleMos agent and remain active until explicitly closed or timed out. Avoid opening redundant connections to the same node
- **Credential security**: Avoid passing passwords as plain text in parameters. Prefer key-based authentication where possible
- **Firewall rules**: Ensure the target node's firewall allows inbound traffic on the specified port
- **Node ID consistency**: Use the same `node_id` across operations to ensure the agent routes commands to the correct connection
