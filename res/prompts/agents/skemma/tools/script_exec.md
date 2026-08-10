+++
name = "script_exec"
agent = "skemma"

[description]
en = "Execute scripts in an isolated environment"
+++

# script_exec

## Description

Executes multi-language scripts in an isolated sandbox environment. Supports Bash, Python, JavaScript, TypeScript, and other languages. Returns stdout, stderr, exit code, and execution time. Use this for quick script execution, automation tasks, cross-language code testing, and data processing without creating a full container.

## Parameters

- **language** (string, optional): Script language. Options: `"bash"`, `"python"`, `"javascript"`, `"typescript"`, `"zsh"`. Default: `"bash"`
- **code** (string, required, separate-call): The source code to execute. Provide via `script_exec.code("...")` in a follow-up call.
- **timeout** (integer, optional): Execution timeout in seconds. Default: `30`

## Supported Languages

| Alias         | Language       |
| --- | --- |
| `bash`, `sh`  | Bash           |
| `python`, `py`| Python         |
| `javascript`, `js`, `node` | JavaScript (Node.js) |
| `typescript`, `ts` | TypeScript |
| `zsh`         | Zsh shell      |

## Returns

### Success

```text
Script executed successfully

Language: python
Exit code: 0

Standard output:
Processing complete
Result: 42

Standard error: (none)

Execution time: 150ms
```

### Failure

```text
Script execution failed

Language: python
Exit code: 1

Standard error:
Traceback (most recent call last):
  File "<string>", line 1, in <module>
NameError: name 'x' is not defined

Execution time: 10ms
```

## Examples

### Example 1: Run a Python script

```json
{
  "language": "python",
  "code": "import json\ndata = {\"status\": \"ok\", \"count\": 42}\nprint(json.dumps(data))"
}
```

### Example 2: Run a Bash command

```json
{
  "language": "bash",
  "code": "echo 'Hello, World!' && date"
}
```

### Example 3: Run JavaScript

```json
{
  "language": "javascript",
  "code": "console.log(JSON.stringify({result: 1 + 1}))"
}
```

## Important Notes

- **Isolated environment**: Scripts run in a sandbox with no persistent state between calls. Files written in one call are not available in the next
- **Timeout**: Long-running scripts are killed after the timeout expires. Increase `timeout` for heavy computations
- **Dependencies**: Only standard library modules are available by default. Third-party packages are not installed
- **Output size**: Very large outputs may be truncated. Keep scripts focused and output concise
