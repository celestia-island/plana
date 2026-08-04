+++
name = "node_execute"
agent = "remote_operations"

[description]
en = "Execute commands on a specified node."
zh-Hans = "在指定节点上执行命令。"
zh-Hant = "在指定節點上執行命令。"
ja = "指定されたノード上でコマンドを実行する。"
ko = "지정된 노드에서 명령어를 실행합니다."
fr = "Exécuter des commandes sur un nœud spécifié."
es = "Ejecutar comandos en un nodo especificado."
ru = "Выполнение команд на указанном узле."
+++

# node_execute

## Description

Executes a command on a remote node that has been connected via `node_connect`. The command runs in the node's default shell and the tool returns the exit code, standard output, and standard error. Supports configurable execution timeouts to prevent hanging on long-running commands.

## Parameters

- **`node_id`** (string, required): Identifier of the target node. The node must be connected before executing commands
- **command** (string, required): The shell command to execute on the remote node
- **timeout** (number, optional): Maximum execution time in seconds before the command is terminated. Default: `30`

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Command executed successfully

node_id: "node-001"
command: "uptime"
exit_code: 0
stdout: " 10:30:00 up 30 days, 5:20, 2 users, load average: 0.15, 0.10, 0.05"
stderr: ""
```

### Failure

```text
Command execution failed

node_id: "node-001"
command: "cat /nonexistent"
exit_code: 1
stdout: ""
stderr: "cat: /nonexistent: No such file or directory"
```

## Examples

### Example 1: Check system uptime

```text
node_id: "node-001"
command: "uptime"
```

### Example 2: Run a build with a long timeout

```text
node_id: "node-002"
command: "cd /var/www/app && npm run build"
timeout: 120
```

### Example 3: Query disk usage

```text
node_id: "node-003"
command: "df -h"
timeout: 10
```

## Important Notes

- **Connection prerequisite**: The target node must already be connected via `node_connect` before executing commands. Invoking `node_execute` on a disconnected node returns a connection error
- **Timeout behavior**: If a command exceeds the specified timeout, it is forcefully terminated and a timeout error is returned. Set appropriate timeouts for long-running operations
- **Non-interactive**: Commands run non-interactively. Interactive prompts (e.g. password prompts) will cause the command to hang until the timeout expires
- **Exit code conventions**: An exit code of `0` indicates success. Non-zero exit codes indicate failure and are accompanied by stderr output
