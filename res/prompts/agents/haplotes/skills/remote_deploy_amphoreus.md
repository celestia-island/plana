+++
name = "remote_deploy_amphoreus"
agent = "haplotes"
[[next_action]]
agent = "skopeo"
name = "node_task_summary"

[description]
en = "Remote deployment of Entelecheia system to new hosts."
zh-Hans = "将 Entelecheia 系统远程部署到新主机。"
zh-Hant = "將 Entelecheia 系統遠端部署到新主機。"
ja = "Entelecheiaシステムを新しいホストにリモートデプロイ。"
ko = "Entelecheia 시스템을 새 호스트에 원격 배포."
fr = "Déploiement à distance du système Entelecheia sur de nouveaux hôtes."
es = "Despliegue remoto del sistema Entelecheia en nuevos hosts."
ru = "Удаленное развертывание системы Entelecheia на новых хостах."

[[related_tools]]
agent_name = "haplotes"
tool_name = "llm_provider_call"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "remote_operations"
tool_name = "connect_remote_via_ssh"

[[related_tools]]
agent_name = "remote_operations"
tool_name = "exec_on_remote"

[[related_tools]]
agent_name = "remote_operations"
tool_name = "node_file_upload"

[[related_tools]]
agent_name = "remote_operations"
tool_name = "disconnect_remote"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[features]
execution_mode = "edge"
location = "cosmos"
+++

# remote_deploy_amphoreus

Deploy the Entelecheia system to remote hosts over SSH, covering environment preparation through service health verification.

## SoP

1. **Gather host information** — Collect target host address, SSH credentials (key path, user, port), and confirm network reachability. Use `report_human()` for any missing parameters.
1. **Validate prerequisites** — SSH into the target and verify OS version, architecture, CPU, memory, and disk meet minimum requirements. Abort with `report_human()` if incompatible.
1. **Assess risks** — Evaluate security posture (key-based auth enforced, no critical CVEs), resource headroom (disk ≥ 20% free, memory ≥ 25% free), and service-port conflicts. Mitigate or escalate before proceeding.
1. **Create rollback point** — Snapshot the target host state (disk snapshot or record current package versions) so a rollback is possible if later steps fail.
1. **Install dependencies** — Deploy required system packages (e.g., Docker, docker-compose, git) using the host's native package manager. Verify each installation.
1. **Transfer configuration** — Copy Entelecheia config files, credentials, and environment files to the target (e.g., `/etc/amphoreus/`). Validate file permissions.
1. **Configure the service** — Apply environment-specific settings (ports, TLS, logging) from the transferred config. Open required firewall ports.
1. **Start Entelecheia** — Launch the service in daemon mode. Capture the PID and confirm the process is running.
1. **Run health checks** — Verify the process is alive, the target port is listening, and the API endpoint returns a healthy response. Collect CPU/memory metrics.
1. **Report results** — Compile a structured report including host, status, duration, endpoints, installed packages, applied settings, and recommendations. Use `report()` for machine output or `report_human()` for operator review.
1. **Capture lessons** — Record any deviations, failures, or configuration tweaks encountered during deployment for future reference using `report()`.

If any step fails, attempt one automatic retry. On second failure, report the error to the operator via `report_human()` and optionally roll back to the snapshot from step 4.

> Return type and IEPL enforcement: @system/return-type-convention
