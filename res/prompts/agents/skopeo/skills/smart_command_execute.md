+++
name = "Smart Command Execution"
agent = "skopeo"

[[next_action]]
agent = "hubris"
name = "plan_execute"

[description]
en = "Intelligent command execution gateway: converts natural language to safe shell commands within containers, executes with security scanning, and intelligently compresses output. This is the sole command execution gateway — the upstream caller does NOT have direct access to exec."
zh-Hans = "智能命令执行网关：将自然语言转换为容器内的安全shell命令，执行时进行安全扫描，并智能压缩输出。这是唯一的命令执行网关——上游调用方不直接访问 exec。"
zh-Hant = "智慧命令執行閘道：將自然語言轉換為容器內的安全shell命令，執行時進行安全掃描，並智慧壓縮輸出。這是唯一的命令執行閘道——上游調用方不直接訪問 exec。"
ja = "インテリジェントコマンド実行ゲートウェイ：自然言語をコンテナ内の安全なシェルコマンドに変換し、セキュリティスキャン付きで実行し、出力をインテリジェントに圧縮します。これは唯一のコマンド実行ゲートウェイです。"
ko = "지능형 명령 실행 게이트웨이: 자연어를 컨테이너 내 안전한 셸 명령으로 변환하고, 보안 스캔과 함께 실행하며, 출력을 지능적으로 압축합니다. 이것이 유일한 명령 실행 게이트웨이입니다."
fr = "Passerelle intelligente d'exécution de commandes : convertit le langage naturel en commandes shell sécurisées dans les conteneurs, exécute avec analyse de sécurité et compresse intelligemment la sortie. C'est la seule passerelle d'exécution de commandes."
es = "Puerta de enlace inteligente de ejecución de comandos: convierte lenguaje natural en comandos shell seguros dentro de contenedores, ejecuta con escaneo de seguridad y comprime inteligentemente la salida. Es la única puerta de enlace de ejecución de comandos."
ru = "Интеллектуальный шлюз выполнения команд: преобразует естественный язык в безопасные команды оболочки внутри контейнеров, выполняет с проверкой безопасности и интеллектуально сжимает вывод. Это единственный шлюз выполнения команд."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "neikos"
tool_name = "exec_on_container"

[features]
execution_mode = "write"
location = "cosmos"
+++

Convert natural language commands into executable container instructions, execute them safely, and intelligently compress output to optimize context usage. This is the **sole command execution gateway** — the upstream caller does NOT have direct access to `exec`.

## SoP

1. **Parse intent** — Receive the natural language command request. Extract the target container, desired operation, and parameters. Use `llm_chat()` for complex intent resolution.
1. **Validate environment** — Confirm the target container exists and is running. Query container metadata (image, runtime, resource limits). If stopped, prompt to start it.
1. **Security scan** — Check the synthesized command against a blocked-pattern list (e.g., `rm -rf /`, `mkfs`). If a dangerous command is detected in safe mode, block execution and call `report_human()` for explicit approval with safe alternatives.
1. **Synthesize command** — Translate the natural language intent into a concrete shell command. If the translation is uncertain, present alternatives to the user for selection.
1. **Execute** — Run the command inside the target container with the configured shell, working directory, environment variables, and timeout. Capture stdout, stderr, and exit code.
1. **Handle errors** — If exit code is non-zero, classify the error type. Retry up to the configured count if the error is transient. For persistent failures, log diagnostics and report.
1. **Compress output** — If output exceeds `max_output_lines`, apply intelligent compression: retain error lines, summary lines, and key patterns; compress repetitive and verbose sections.
1. **Report** — Call `report()` with the execution summary: command, exit code, compressed output, timing, and any errors encountered.

> Return type and IEPL enforcement: @system/return-type-convention

## IEPL Preference Check

Before executing any shell command via `exec_on_container()`:

1. Can this be done with JavaScript string methods (match, replace, filter)?
1. If YES → use `exec()` with JavaScript code instead of `exec_on_container()`
1. If NO → proceed with `exec_on_container()` but document why IEPL is insufficient
