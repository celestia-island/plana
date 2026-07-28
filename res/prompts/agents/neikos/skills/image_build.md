+++
name = "Image Build"
agent = "neikos"

[description]
en = "Verify, pull, or build container images to ensure image availability and correctness."
zhs = "验证、拉取或构建容器镜像，确保镜像的可用性和正确性。"
zht = "驗證、拉取或建構容器映像，確保映像的可用性和正確性。"
ja = "コンテナイメージを検証、プル、またはビルドし、イメージの可用性と正確性を確保します。"
ko = "컨테이너 이미지를 검증, 풀 또는 빌드하여 이미지 가용성과 정확성을 보장합니다."
fr = "Vérifier, extraire ou construire des images de conteneur pour assurer leur disponibilité et leur exactitude."
es = "Verificar, extraer o construir imágenes de contenedor para asegurar su disponibilidad y corrección."
ru = "Проверить, извлечь или собрать образы контейнеров для обеспечения их доступности и корректности."

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
agent_name = "neikos"
tool_name = "container_create"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[features]
location = "cosmos"
execution_mode = "write"
+++

# Image Build

Verify, pull, or build container images to ensure image availability and correctness.

## SoP

1. **Gather requirements** — Determine the target image name, tag, registry source, or Dockerfile path. Use `report_human()` if any required detail is missing.

1. **Check local state** — Use `exec` to run `docker images` and verify whether the desired image already exists locally. Use `container_list()` to check if any running container uses it.

1. **Assess feasibility** — Use `exec` to check disk space (`df -h`) and network connectivity to the registry. If building from a Dockerfile, verify the file exists and read its contents.

1. **Decide strategy** — If the image exists locally and meets requirements, skip to step 6. Otherwise decide between pulling from a registry or building locally based on the Dockerfile availability.

1. **Execute operation** — Use `exec` to run the appropriate command:

   - **Pull**: `docker pull <image>:<tag>`
   - **Build**: `docker build -t <name>:<tag> -f <dockerfile> <context>`
   - On failure, analyze error output, retry with adjusted parameters, or fall back to an alternative source.

1. **Verify result** — Use `exec` to run `docker inspect <image>` and confirm the image ID, size, layers, entrypoint, exposed ports, and environment variables match expectations. Optionally use `container_create()` to launch a short-lived test container and validate runtime behavior.

1. **Tag and push** (optional) — If required, use `exec` to run `docker tag` and `docker push` to publish the image to the target registry.

1. **Report** — Use `report()` to summarize the operation outcome, image metadata, and any warnings encountered.

> Return type and IEPL enforcement: @system/return-type-convention
