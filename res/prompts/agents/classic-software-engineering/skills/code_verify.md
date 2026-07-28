+++
name = "code_verify"
agent = "classic_software_engineering"

[description]
en = "Build and test generated code in a sandboxed container. Runs compilation, executes tests, and feeds errors back to code_generate if needed."
zh-Hans = "在沙箱容器中构建和测试生成的代码。运行编译、执行测试，并在需要时将错误反馈给 code_generate。"
zh-Hant = "在沙箱容器中構建和測試生成的程式碼。運行編譯、執行測試，並在需要時將錯誤反饋給 code_generate。"
ja = "サンドボックスコンテナで生成されたコードをビルドしてテストします。コンパイルを実行し、テストを実行し、必要に応じてエラーを code_generate にフィードバックします。"
ko = "샌드박스 컨테이너에서 생성된 코드를 빌드하고 테스트합니다. 컴파일을 실행하고, 테스트를 실행하며, 필요한 경우 오류를 code_generate에 피드백합니다."
fr = "Construire et tester le code généré dans un conteneur sandboxé. Exécute la compilation, lance les tests et renvoie les erreurs à code_generate si nécessaire."
es = "Construir y probar el código generado en un contenedor aislado. Ejecuta la compilación, ejecuta pruebas y retroalimenta errores a code_generate si es necesario."
ru = "Собрать и протестировать сгенерированный код в изолированном контейнере. Выполняет компиляцию, запускает тесты и передаёт ошибки обратно в code_generate при необходимости."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "neikos"
tool_name = "exec_on_container"

[[related_tools]]
agent_name = "neikos"
tool_name = "container_fork"

[[related_tools]]
agent_name = "neikos"
tool_name = "container_info"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

[features]
execution_mode = "write"
location = "cosmos"
must_touch_next_action = false
+++

# code_verify

Build and test the generated code in a sandboxed container. Verify compilation, run tests, and report results.

## IMPORTANT: File Path Convention

The **Workspace** in the environment section is the HOST path — do NOT use it. Inside the container, the workspace is always at `/workspace`.

## Build Commands by Language

Detect the language from workspace files and use the appropriate build command:

| Language | Detection File | Build Command | Test Command |
| --- | --- | --- | --- |
| Rust | `Cargo.toml` | `cargo build 2>&1` | `cargo test 2>&1` |
| TypeScript/JavaScript | `package.json` | `npm install 2>&1 && npm run build 2>&1` | `npm test 2>&1` |
| Python | `pyproject.toml` or `setup.py` | `python -m py_compile src/*.py 2>&1` | `pytest 2>&1` |
| Go | `go.mod` | `go build ./... 2>&1` | `go test ./... 2>&1` |
| C/C++ | `CMakeLists.txt` or `Makefile` | `make 2>&1` or `cmake . && make 2>&1` | `make test 2>&1` or `ctest 2>&1` |
| Java | `pom.xml` | `mvn compile 2>&1` | `mvn test 2>&1` |
| General | none | Check for syntax errors manually | N/A |

## SoP

### Phase 1: DETECT & BUILD

1. **Detect language** — Use `smart_read_file` with query `"list top-level config files (Cargo.toml, package.json, go.mod, etc.)"` to determine the language and build system.
1. **Build** — Execute the build command via `exec_on_container`:

   ```json
   exec({ code: "import { exec_on_container } from 'neikos'; const r = await exec_on_container({ command: 'cargo build 2>&1' }); r" })
   ```

1. **Check result** — If build succeeds (exit code 0), proceed to Phase 2. If build fails, proceed to Phase 3.

### Phase 2: TEST

1. **Run tests** — Execute the test command via `exec_on_container`:

   ```json
   exec({ code: "import { exec_on_container } from 'neikos'; const r = await exec_on_container({ command: 'cargo test 2>&1' }); r" })
   ```

1. **Collect results** — Capture test output, pass/fail counts, and any error messages.

### Phase 3: FIX (if build or tests fail)

1. **Analyze errors** — Parse the compiler/test output to identify root causes.
1. **Fix code** — For each error:

   - Read the failing file with `smart_read_file`
   - Apply the fix with `smart_write_file`
   - Re-build to verify the fix

1. **Retry limit** — Maximum 3 build-fix cycles. After 3 failures, report the remaining errors and move to Phase 4.

### Phase 4: REPORT

1. **Compile results** — Summarize:

   - Build status: success/failure
   - Test results: pass/fail counts
   - Files modified during fix attempts
   - Remaining issues (if any)

1. **Report to human** — Use the `report_human` convention:

    ```json
    write_to_var({ var_name: "reply_summary", content: "Code verification: [BUILD OK/BUILD FAILED], [N/M] tests passed" })
    write_to_var({ var_name: "reply_body", content: "## Verification Results\n\n### Build\n...\n### Tests\n...\n### Fixes Applied\n..." })
    exec({ code: "import { report_human } from 'hubris'; import vars from 'vars'; report_human({ summary: vars['reply_summary'], body: vars['reply_body'] });" })
    ```

## Critical Rules

- **Always build first** — Do not skip to tests. Compilation errors must be resolved before testing.
- **Preserve generated code intent** — When fixing errors, make minimal changes. Do not rewrite the entire file.
- **Timeout awareness** — If a build or test command takes longer than 60 seconds, consider it a timeout. Report the issue.
- **No network access assumed** — The sandbox may not have network access. If a build requires downloading dependencies (e.g., `npm install`, `cargo build` fetching crates), report the limitation.

> Return type and IEPL enforcement: @system/return-type-convention
> IEPL-first: @system/iepl-first
