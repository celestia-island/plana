+++
name = "image_build"
agent = "neikos"

[[next_action]]
agent = "hubris"
name = "plan_execute"

[description]
en = "Container image build gateway with Dockerfile validation, multi-stage support, and registry push"
zh-Hans = "容器镜像构建网关：Dockerfile验证、多阶段构建、仓库推送"

[[related_tools]]
agent_name = "cosmos"
tool_name = "exec"

[[related_tools]]
agent_name = "neikos"
tool_name = "container_list"

[[related_tools]]
agent_name = "neikos"
tool_name = "container_info"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
location = "cosmos"
execution_mode = "write"
+++

# image_build

Build container images based on upstream context. This skill is the **gateway** for image operations — the upstream caller does NOT have direct access to docker commands.

## SoP

1. **Parse request** — The upstream context describes what image to build, from what source, and with what configuration.
1. **Validate Dockerfile** — If a Dockerfile path is provided, read and validate it:

   - Check for `FROM` with known base images
   - Warn about `latest` tags in production Dockerfiles
   - Verify `COPY`/`ADD` source paths exist in workspace

1. **Check prerequisites** — Use `container_list()` and `container_info()` to assess:

   - Available disk space for build context
   - Whether base images are already pulled
   - Whether a previous build of this image exists

1. **Build** — Use `exec` to run `docker build`:

   - Set `--no-cache` only if explicitly requested
   - Use `--build-arg` for configuration injection
   - Tag with both human-readable name and content hash

1. **Verify** — Run `docker images` to confirm the image was created. Check size is reasonable.
1. **Report** — Call `report()` with image ID, size, tags, and any warnings.

> Return type and IEPL enforcement: @system/return-type-convention

## Safety Rules

- Never build images with embedded secrets
- Always pin base image versions (no `latest` in `FROM`)
- Set resource limits on build (`--memory`, `--cpu-quota`)
- Clean up intermediate images after multi-stage builds

## Edge Cases

- **Dockerfile not found**: Report error, suggest creating one
- **Build fails mid-way**: Report the failing layer, suggest fix
- **Disk full**: Report space requirements, suggest cleanup
- **Base image pull fails**: Check registry connectivity, suggest mirror or local image
- **Timeout**: Large builds may need longer timeout, use `report_human()` to confirm
