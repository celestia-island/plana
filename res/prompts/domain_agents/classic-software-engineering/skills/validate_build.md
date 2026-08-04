+++
name = "validate_build"
agent = "classic_software_engineering"

[description]
en = "Validate build health for a Rust workspace after code modifications. Runs cargo check and cargo test for affected packages, reports results. Designed as a pipeline safety-net hook target."
zh-Hans = "验证代码修改后的构建健康度。对受影响的包运行 cargo check 和 cargo test，报告结果。设计为 pipeline 安全网 hook 的目标。"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "polemos"
tool_name = "host_command_exec"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[features]
execution_mode = "read"
location = "cosmos"
must_touch_next_action = false
+++

# validate_build

Validate that code modifications have not broken the build.

**This skill is designed to be called by the pipeline surgery hook system or by `plan_execute` SoP-6 as a validation step.**

## SoP: Build Validation

**Step 1**: Determine affected packages from changed files.

Parse the list of changed file paths to identify which workspace packages are affected:

```text
packages/scepter/src/foo.rs → scepter
packages/shared/core/src/bar.rs → _shared_core
```

**Step 2**: Run `cargo check` for affected packages.

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /mnt/sdb1/entelecheia && cargo check -p PACKAGE1 -p PACKAGE2 2>&1 | tail -20', timeout: 300 }); const out = r.data.stdout || r.data.stderr || ''; console.log(out); write_to_var({ var_name: 'check_result', content: out });" })
```

**Step 3**: If check passes, run `cargo test` for affected packages (optional, depends on timeout).

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd /mnt/sdb1/entelecheia && cargo test -p PACKAGE1 -p PACKAGE2 --no-run 2>&1 | tail -10', timeout: 300 }); const out = r.data.stdout || r.data.stderr || ''; console.log(out);" })
```

**Step 4**: Report validation result.

Parse the output:

- If contains `error[` or `error: ` → validation FAILED, list errors
- If contains `Finished` or `warning:` only → validation PASSED
- Include: packages checked, packages tested, error count, warning count

## Integration Points

This skill can be triggered by:

1. **pipeline.surgery.post.rollback** hook — automatic validation after chain
1. **`plan_execute` SoP-6** — LLM calls this explicitly during self-surgery
1. **Manual invocation** — user asks to validate build

> Return type: @system/return-type-convention
> IEPL-first: @system/iepl-first
