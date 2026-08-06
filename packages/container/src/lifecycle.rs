use std::collections::HashMap;

use bollard::{
    models::ContainerCreateBody,
    query_parameters::{
        CreateContainerOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    },
    service::HostConfig,
};
use tracing::{debug, warn};

use super::{
    egress::EgressPolicy,
    errors::{ContainerError, ContainerResult},
    events::ContainerEvent,
    manager::ContainerManager,
    seccomp, security_profile,
    types::{ContainerCreateParams, ContainerInfo, ContainerStatus, VolumeMount},
};
use _core::constants::DEFAULT_NETWORK;

fn emit_event(tx: &tokio::sync::broadcast::Sender<ContainerEvent>, event: ContainerEvent) {
    if let Err(e) = tx.send(event) {
        debug!(error = %e, "container event broadcast failed (no active receivers)");
    }
}

/// Default bridge network for containers created without an explicit one:
/// the generic CONTAINER_NETWORK env override wins, then the legacy
/// entelecheia-network default.
fn default_network() -> String {
    std::env::var("CONTAINER_NETWORK").unwrap_or_else(|_| DEFAULT_NETWORK.to_string())
}

impl ContainerManager {
    pub async fn create(&self, params: &ContainerCreateParams) -> ContainerResult<ContainerInfo> {
        // The default bridge network is deployment-specific; the generic
        // CONTAINER_NETWORK env override wins, the legacy entelecheia-network
        // default is kept for existing deployments.
        let network: std::borrow::Cow<str> = match &params.network {
            Some(network) => std::borrow::Cow::Borrowed(network),
            None => std::borrow::Cow::Owned(default_network()),
        };

        let env_vec: Vec<String> = params
            .env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        let exposed_ports = self.build_exposed_ports(&params.ports);
        let port_bindings = self.build_port_bindings(&params.ports);
        let mounts = self.build_mounts(&params.volumes);

        let mut labels = HashMap::new();
        for (k, v) in &params.labels {
            labels.insert(k.clone(), v.clone());
        }

        let cap_drop = params
            .cap_drop
            .clone()
            .unwrap_or_else(|| vec!["ALL".to_string()]);
        let security_opt = params
            .security_opt
            .clone()
            .unwrap_or_else(|| seccomp::build_security_opts(None));

        let mut host_config = HostConfig {
            network_mode: Some(network.to_string()),
            port_bindings: Some(port_bindings),
            mounts: if mounts.is_empty() {
                None
            } else {
                Some(mounts)
            },
            memory: params.memory_limit,
            nano_cpus: params.nano_cpus,
            pids_limit: params.pids_limit,
            cap_drop: Some(cap_drop),
            cap_add: params.cap_add.clone(),
            security_opt: Some(security_opt),
            readonly_rootfs: params.read_only_rootfs,
            group_add: params.group_add.clone(),
            devices: if params.devices.is_empty() {
                None
            } else {
                Some(
                    params
                        .devices
                        .iter()
                        .map(|d| bollard::service::DeviceMapping {
                            path_on_host: Some(d.host_path.clone()),
                            path_in_container: Some(d.container_path.clone()),
                            cgroup_permissions: Some(d.permissions.clone()),
                        })
                        .collect(),
                )
            },
            ..Default::default()
        };

        if let Some(ref driver) = params.log_driver {
            let mut log_config = bollard::service::HostConfigLogConfig {
                typ: Some(driver.clone()),
                config: Some(HashMap::new()),
            };
            if let Some(config_map) = log_config.config.as_mut() {
                for (k, v) in &params.log_opts {
                    config_map.insert(k.clone(), v.clone());
                }
            }
            host_config.log_config = Some(log_config);
        }

        let default_egress = EgressPolicy::entelecheia_default();
        let egress = params.egress_policy.as_ref().unwrap_or(&default_egress);
        egress.apply_to_docker_config(&mut host_config);

        let healthcheck = params
            .healthcheck
            .as_ref()
            .map(|hc| bollard::service::HealthConfig {
                test: Some(hc.test.clone()),
                interval: hc.interval_ns,
                timeout: hc.timeout_ns,
                retries: hc.retries,
                start_period: hc.start_period_ns,
                ..Default::default()
            });

        let config = ContainerCreateBody {
            image: Some(params.image.clone()),
            user: params.user.clone(),
            env: if env_vec.is_empty() {
                None
            } else {
                Some(env_vec)
            },
            labels: Some(labels),
            exposed_ports: if exposed_ports.is_empty() {
                None
            } else {
                Some(exposed_ports)
            },
            host_config: Some(host_config),
            working_dir: params.working_dir.clone(),
            healthcheck,
            ..Default::default()
        };

        let create_options = CreateContainerOptions {
            name: Some(params.name.clone()),
            ..Default::default()
        };

        let response = self
            .docker
            .create_container(Some(create_options), config)
            .await
            .map_err(|e| {
                if matches!(
                    e,
                    bollard::errors::Error::DockerResponseServerError {
                        status_code: 409,
                        ..
                    }
                ) {
                    ContainerError::AlreadyExists(params.name.clone())
                } else {
                    ContainerError::from(e)
                }
            })?;

        if let Err(e) = self
            .docker
            .start_container(&response.id, None::<StartContainerOptions>)
            .await
        {
            // Remove the already-created container to prevent leaks.
            // If start fails (resource exhaustion, race condition), the
            // container ID is lost and would otherwise accumulate indefinitely.
            let container_id = response.id.clone();
            let _ = self
                .docker
                .remove_container(
                    &container_id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await;
            return Err(e.into());
        }

        if !network.is_empty() {
            // Validate network parameter to prevent Docker CLI injection
            // (e.g. network="--help" or "-x" would be interpreted as Docker
            // flags).
            if network.starts_with('-') || network.contains(' ') {
                warn!(
                    container = %params.name,
                    network = %network,
                    "invalid network name rejected (starts with '-' or contains space)"
                );
            } else {
                match tokio::process::Command::new("docker")
                    .args(["network", "connect", &network, &response.id])
                    .output()
                    .await
                {
                    Ok(output) => {
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            // "endpoint already exists" is non-fatal — the container
                            // is already on the network from a previous creation
                            // attempt with the same name. Log at debug, not warn.
                            if stderr.contains("already exists") {
                                debug!(
                                    container = %params.name,
                                    network = %network,
                                    "container already on network (non-fatal)"
                                );
                            } else {
                                warn!(
                                    container = %params.name,
                                    network = %network,
                                    error = %stderr,
                                    "docker network connect failed, container may not be on network"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            container = %params.name,
                            network = %network,
                            error = %e,
                            "docker network connect command failed"
                        );
                    }
                }
            }
        }

        let info = match self.inspect_to_container_info(&response.id).await {
            Ok(info) => info,
            Err(e) => {
                // Container was created+started but inspect failed. Remove the
                // running container to prevent leaks.
                let container_id = response.id.clone();
                let _ = self
                    .docker
                    .remove_container(
                        &container_id,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await;
                return Err(e);
            }
        };

        let mut state = self.state.write().await;
        state.insert(info.id.clone(), info.clone());

        emit_event(
            &self.event_tx,
            ContainerEvent::Created {
                id: info.id.clone(),
                name: info.name.clone(),
                image: info.image.clone(),
            },
        );

        Ok(info)
    }

    pub async fn start(&self, container_id: &str) -> ContainerResult<()> {
        let old_status = self.get_status(container_id).await;

        self.docker
            .start_container(container_id, None::<StartContainerOptions>)
            .await
            .map_err(|e| {
                if matches!(
                    e,
                    bollard::errors::Error::DockerResponseServerError {
                        status_code: 404,
                        ..
                    }
                ) {
                    ContainerError::NotFound(container_id.to_string())
                } else {
                    ContainerError::from(e)
                }
            })?;

        self.refresh_state(container_id).await?;

        if let Some(old) = old_status {
            emit_event(
                &self.event_tx,
                ContainerEvent::StatusChanged {
                    id: container_id.to_string(),
                    old_status: old,
                    new_status: ContainerStatus::Running,
                },
            );
        }
        emit_event(
            &self.event_tx,
            ContainerEvent::Started {
                id: container_id.to_string(),
            },
        );

        Ok(())
    }

    pub async fn stop(&self, container_id: &str) -> ContainerResult<()> {
        let old_status = self.get_status(container_id).await;

        self.docker
            .stop_container(
                container_id,
                Some(StopContainerOptions {
                    signal: None,
                    t: Some(30),
                }),
            )
            .await
            .map_err(|e| {
                if matches!(
                    e,
                    bollard::errors::Error::DockerResponseServerError {
                        status_code: 404,
                        ..
                    }
                ) {
                    ContainerError::NotFound(container_id.to_string())
                } else {
                    ContainerError::from(e)
                }
            })?;

        self.refresh_state(container_id).await?;

        if let Some(old) = old_status {
            emit_event(
                &self.event_tx,
                ContainerEvent::StatusChanged {
                    id: container_id.to_string(),
                    old_status: old,
                    new_status: ContainerStatus::Exited,
                },
            );
        }
        emit_event(
            &self.event_tx,
            ContainerEvent::Stopped {
                id: container_id.to_string(),
            },
        );

        Ok(())
    }

    pub async fn restart(&self, container_id: &str) -> ContainerResult<()> {
        self.docker
            .restart_container(container_id, None)
            .await
            .map_err(|e| {
                if matches!(
                    e,
                    bollard::errors::Error::DockerResponseServerError {
                        status_code: 404,
                        ..
                    }
                ) {
                    ContainerError::NotFound(container_id.to_string())
                } else {
                    ContainerError::from(e)
                }
            })?;

        self.refresh_state(container_id).await?;

        Ok(())
    }

    pub async fn remove(&self, container_id: &str, force: bool) -> ContainerResult<()> {
        let options = RemoveContainerOptions {
            force,
            ..Default::default()
        };

        self.docker
            .remove_container(container_id, Some(options))
            .await
            .map_err(|e| {
                if matches!(
                    e,
                    bollard::errors::Error::DockerResponseServerError {
                        status_code: 404,
                        ..
                    }
                ) {
                    ContainerError::NotFound(container_id.to_string())
                } else {
                    ContainerError::from(e)
                }
            })?;

        {
            let mut state = self.state.write().await;
            state.remove(container_id);
        }

        emit_event(
            &self.event_tx,
            ContainerEvent::Destroyed {
                id: container_id.to_string(),
            },
        );

        Ok(())
    }

    pub async fn wait_healthy(
        &self,
        container_name: &str,
        timeout: std::time::Duration,
    ) -> ContainerResult<()> {
        let poll_interval = std::env::var("DOCKER_HEALTH_POLL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(std::time::Duration::from_millis)
            .unwrap_or(std::time::Duration::from_millis(500));

        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let inspect = self
                .docker
                .inspect_container(container_name, None)
                .await
                .map_err(ContainerError::from)?;

            let healthy = inspect
                .state
                .as_ref()
                .and_then(|s| s.health.as_ref())
                .map(|h| h.status.as_ref() == Some(&bollard::service::HealthStatusEnum::HEALTHY))
                .unwrap_or(true);

            if healthy {
                return Ok(());
            }

            if tokio::time::Instant::now() + poll_interval > deadline {
                return Err(ContainerError::OperationFailed {
                    container_id: container_name.to_string(),
                    message: format!("did not become healthy within {:?}", timeout),
                });
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    pub async fn ensure_running(&self, container_id: &str) -> ContainerResult<ContainerInfo> {
        let detail = self.inspect(container_id).await?;

        if !detail.info.status.is_running() {
            self.start(container_id).await?;
            return self.inspect_to_container_info(container_id).await;
        }

        Ok(detail.info)
    }

    pub async fn recreate(
        &self,
        container_name: &str,
        new_image: &str,
    ) -> ContainerResult<ContainerInfo> {
        let old = self
            .list_with_filter(Some(&format!("^{}$", container_name)), None, true)
            .await
            .map_err(|e| {
                ContainerError::NotFound(format!("container {} not found: {}", container_name, e))
            })?;

        let old_info = old
            .into_iter()
            .next()
            .ok_or_else(|| ContainerError::NotFound(container_name.to_string()))?;

        self.inspect(&old_info.id).await?;

        let full_inspect = self
            .docker
            .inspect_container(&old_info.id, None)
            .await
            .map_err(ContainerError::from)?;

        let old_config = full_inspect.config.as_ref();
        let old_host = full_inspect.host_config.as_ref();

        let env_map: HashMap<String, String> = old_config
            .and_then(|c| c.env.as_ref())
            .map(|env_vec| {
                env_vec
                    .iter()
                    .filter_map(|e| {
                        let parts: Vec<&str> = e.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            Some((parts[0].to_string(), parts[1].to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let labels: HashMap<String, String> = old_config
            .and_then(|c| c.labels.clone())
            .unwrap_or_default();

        let volumes: Vec<VolumeMount> = full_inspect
            .mounts
            .as_ref()
            .map(|mounts| {
                mounts
                    .iter()
                    .map(|m| VolumeMount {
                        host_path: m.source.as_ref().cloned().unwrap_or_default(),
                        container_path: m.destination.as_ref().cloned().unwrap_or_default(),
                        read_only: m.rw.map(|rw| !rw).unwrap_or(false),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let network = old_host
            .and_then(|h| h.network_mode.clone())
            .unwrap_or_else(default_network);

        let memory_limit = old_host.and_then(|h| h.memory);
        let nano_cpus = old_host.and_then(|h| h.nano_cpus);
        let pids_limit = old_host.and_then(|h| h.pids_limit);
        let user = old_config.and_then(|c| c.user.clone());

        let healthcheck = old_config.and_then(|c| c.healthcheck.as_ref()).map(|hc| {
            super::types::HealthcheckParams {
                test: hc.test.clone().unwrap_or_default(),
                interval_ns: hc.interval,
                timeout_ns: hc.timeout,
                retries: hc.retries,
                start_period_ns: hc.start_period,
            }
        });

        warn!(
            event = "container_recreating",
            container = %container_name,
            old_image = %old_info.image,
            new_image = %new_image,
            "recreating container with new image"
        );

        self.stop(&old_info.id).await?;
        self.remove(&old_info.id, true).await?;

        self.pull_image(new_image).await?;

        let sec = security_profile::scepter();

        let create_params = ContainerCreateParams {
            name: container_name.to_string(),
            image: new_image.to_string(),
            network: Some(network),
            env: env_map,
            ports: Vec::new(),
            volumes,
            labels,
            working_dir: old_config.and_then(|c| c.working_dir.clone()),
            log_driver: None,
            log_opts: HashMap::new(),
            healthcheck,
            user,
            memory_limit,
            nano_cpus,
            pids_limit,
            cap_drop: sec.cap_drop,
            cap_add: sec.cap_add,
            security_opt: sec.security_opt,
            read_only_rootfs: None,
            egress_policy: sec.egress_policy,
            group_add: None,
            devices: Vec::new(),
            compile_mode: false,
            egress_domain_allowlist_extra: Vec::new(),
        };

        let new_info = self.create(&create_params).await?;

        emit_event(
            &self.event_tx,
            ContainerEvent::Updated {
                id: new_info.id.clone(),
                name: container_name.to_string(),
                old_image: old_info.image.clone(),
                new_image: new_image.to_string(),
            },
        );

        Ok(new_info)
    }
}

#[cfg(test)]
mod network_default_tests {
    use super::default_network;

    #[test]
    fn default_network_uses_generic_env_then_legacy_constant() {
        let ambient = std::env::var("CONTAINER_NETWORK").ok();
        // SAFETY: test-only env mutation; no other thread reads this var.
        unsafe {
            std::env::set_var("CONTAINER_NETWORK", "custom-net");
            assert_eq!(default_network(), "custom-net");
            std::env::remove_var("CONTAINER_NETWORK");
            assert_eq!(default_network(), "entelecheia-network");
        }
        match ambient {
            Some(value) => unsafe { std::env::set_var("CONTAINER_NETWORK", value) },
            None => unsafe { std::env::remove_var("CONTAINER_NETWORK") },
        }
    }
}
