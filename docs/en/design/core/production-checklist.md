+++
title = "Entelecheia Production Deployment Checklist"
description = """> 12-step checklist for deploying Entelecheia to production."""
lang = "en"
category = "design"
subcategory = "core"
+++

# Entelecheia Production Deployment Checklist

> 12-step checklist for deploying Entelecheia to production.

## Pre-Deployment

- [ ] **1. Choose Database Mode**
  - Embedded pglite: single-binary, no external DB. Suitable for <50 concurrent agents.
  - PostgreSQL: recommended for production. Set `DATABASE_URL`.

  ```bash
  # Embedded mode
  docker run -d -p 8080:8080 -v data:/data entelecheia:latest

  # PostgreSQL mode
  docker-compose up -d
  ```

- [ ] **2. Configure User Identity**

  ```bash
  export ENTELECHEIA_USER_UUID=$(uuidgen)
  ```

This UUID is the workspace owner identity. All agent operations are scoped to it.

- [ ] **3. Set Up LLM Providers**

  ```bash
  entelecheia-cli config set-provider openai --api-key sk-...
  entelecheia-cli config set-provider anthropic --api-key sk-ant-...
  ```

API keys are encrypted at rest with AES-256-GCM via the Aporia agent.

- [ ] **4. Configure Container Runtime**
  - Docker (default): `--container-backend docker`
  - Youki (rootless OCI): `--container-backend youki`
  - Verify seccomp profile: `configs/seccomp/`

- [ ] **5. Review Security Policies**

  ```bash
  # List registered security policies
  entelecheia-cli security policy-list

  # Review OreXis sentinel configuration
  entelecheia-cli config show orexis
  ```

## Deployment

- [ ] **6. Build or Pull Image**

  ```bash
  # Build from source
  docker build -t entelecheia:latest .

  # Or use release
  curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
  ```

- [ ] **7. Start the Service**

  ```bash
  # Using Docker Compose (recommended)
  docker-compose up -d

  # Or standalone
  docker run -d --name entelecheia \
    -p 8080:8080 \
    -v entelecheia-data:/data \
    -e ENTELECHEIA_USER_UUID=$ENTELECHEIA_USER_UUID \
    --restart unless-stopped \
    entelecheia:latest
  ```

- [ ] **8. Verify Health**

  ```bash
  entelecheia-cli status
  curl http://localhost:8080/health
  ```

- [ ] **9. Initialize Docker Images for Agents**

  ```bash
  entelecheia-cli init-docker-images
  ```

This builds the container images used by each Layer-1 agent for isolated execution.

## Post-Deployment

- [ ] **10. Set Up Monitoring**

  ```bash
  # Enable tracing
  export RUST_LOG=info,entelecheia=debug

  # Check timeline for issues
  entelecheia-cli timeline list --agent orexis
  ```

- [ ] **11. Configure Backups**
  - Embedded mode: back up `/data` directory
  - PostgreSQL: `pg_dump` or WAL archiving
  - Timeline audit logs: export periodically

- [ ] **12. Load Test**

  ```bash
  # Send a test message
  entelecheia-cli send "Hello, verify the system is operational"

  # Check agent status
  entelecheia-cli agent list

  # Verify audit trail
  entelecheia-cli trace-chain demiurge.001
  ```

## Security Hardening (Recommended)

| Check | Command |
| --- | --- |
| Verify no secrets in env | `env \| grep -i key` |
| Review RBAC groups | `entelecheia-cli security rbac-list` |
| Check rate limits | `entelecheia-cli config show channel.rate_limit` |
| Verify container isolation | `docker inspect entelecheia \| grep SecurityOpt` |
| Review OreXis audit log | `entelecheia-cli logs --agent orexis --lines 100` |

## Troubleshooting

| Symptom | Diagnostic |
| --- | --- |
| Agents not responding | `entelecheia-cli status` → check scepter is running |
| LLM calls failing | Check API keys: `entelecheia-cli config show providers` |
| Container errors | `docker logs entelecheia` → look for Youki/Docker errors |
| Database issues | Check `DATABASE_URL` or pglite data directory permissions |
| Tool permission denied | `entelecheia-cli security policy-list` → review denied calls |
