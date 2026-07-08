use chrono::{DateTime, Utc};
use std::collections::HashMap;

use bollard::service::{Mount, MountTypeEnum, PortBinding};

use super::{
    errors::{ContainerError, ContainerResult},
    manager::ContainerManager,
    types::{ContainerInfo, ContainerStatus, VolumeMount},
};

impl ContainerManager {
    pub(crate) async fn refresh_state(&self, container_id: &str) -> ContainerResult<()> {
        match self.inspect_to_container_info(container_id).await {
            Ok(info) => {
                let mut state = self.state.write().await;
                state.insert(info.id.clone(), info);
                Ok(())
            },
            Err(ContainerError::NotFound(_)) => {
                let mut state = self.state.write().await;
                state.remove(container_id);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub(crate) async fn get_status(&self, container_id: &str) -> Option<ContainerStatus> {
        let state = self.state.read().await;
        state.get(container_id).map(|c| c.status)
    }

    pub(crate) async fn inspect_to_container_info(
        &self,
        container_id: &str,
    ) -> ContainerResult<ContainerInfo> {
        let response = self
            .docker
            .inspect_container(container_id, None)
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

        Ok(self.inspect_to_info(&response))
    }

    pub(crate) fn summary_to_info(
        &self,
        summary: &bollard::service::ContainerSummary,
    ) -> ContainerInfo {
        let id = summary.id.clone().unwrap_or_default();
        let name = summary
            .names
            .as_ref()
            .and_then(|n| n.first().cloned())
            .unwrap_or_default();
        let name = name.trim_start_matches('/').to_string();

        let image = summary.image.clone().unwrap_or_default();

        let running = matches!(
            summary.state,
            Some(bollard::models::ContainerSummaryStateEnum::RUNNING)
        );
        let status_str = summary.status.as_deref().unwrap_or("unknown");
        let status = ContainerStatus::from_str(status_str, running);

        let labels = summary.labels.clone().unwrap_or_default();
        let ip_address = summary
            .network_settings
            .as_ref()
            .and_then(|ns| ns.networks.as_ref())
            .and_then(|networks| networks.values().next().and_then(|n| n.ip_address.clone()))
            .filter(|ip| !ip.is_empty());

        ContainerInfo {
            id,
            name,
            image,
            status,
            created_at: None,
            ports: Vec::new(),
            env: HashMap::new(),
            volumes: Vec::new(),
            ip_address,
            labels,
        }
    }

    pub(crate) fn inspect_to_info(
        &self,
        inspect: &bollard::service::ContainerInspectResponse,
    ) -> ContainerInfo {
        let id = inspect.id.clone().unwrap_or_default();
        let name = inspect
            .name
            .clone()
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string();
        let image = inspect.image.clone().unwrap_or_default();

        let running = inspect
            .state
            .as_ref()
            .and_then(|s| s.running)
            .unwrap_or(false);
        let status_enum = inspect
            .state
            .as_ref()
            .and_then(|s| s.status)
            .unwrap_or(bollard::service::ContainerStateStatusEnum::EMPTY);
        let status = ContainerStatus::from_docker_state(&status_enum, running);

        let labels = inspect.config.as_ref().and_then(|c| c.labels.clone());
        let labels = labels.unwrap_or_default();

        let env: HashMap<String, String> = inspect
            .config
            .as_ref()
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

        let ip_address = inspect
            .network_settings
            .as_ref()
            .and_then(|ns| ns.networks.as_ref())
            .and_then(|networks| {
                networks
                    .values()
                    .find_map(|n| n.ip_address.as_ref().filter(|ip| !ip.is_empty()).cloned())
            });

        let mounts: Vec<VolumeMount> = inspect
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

        ContainerInfo {
            id,
            name,
            image,
            status,
            created_at: inspect.created.as_ref().and_then(|c| {
                DateTime::parse_from_rfc3339(c)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            ports: Vec::new(),
            env,
            volumes: mounts,
            ip_address,
            labels,
        }
    }

    pub(crate) fn build_exposed_ports(&self, ports: &[super::types::PortMapping]) -> Vec<String> {
        let mut vec = Vec::new();
        for p in ports {
            let key = format!("{}/{}", p.container_port, p.protocol);
            vec.push(key);
        }
        vec
    }

    pub(crate) fn build_port_bindings(
        &self,
        ports: &[super::types::PortMapping],
    ) -> HashMap<String, Option<Vec<PortBinding>>> {
        let mut map = HashMap::new();
        for p in ports {
            let key = format!("{}/{}", p.container_port, p.protocol);
            map.insert(
                key,
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(p.host_port.to_string()),
                }]),
            );
        }
        map
    }

    pub(crate) fn build_mounts(&self, volumes: &[VolumeMount]) -> Vec<Mount> {
        volumes
            .iter()
            .map(|v| {
                let is_named_volume = !v.host_path.starts_with('/');
                let (typ, bind_options, volume_options) = if is_named_volume {
                    (
                        Some(MountTypeEnum::VOLUME),
                        None,
                        Some(bollard::service::MountVolumeOptions {
                            no_copy: None,
                            labels: None,
                            driver_config: None,
                            subpath: None,
                        }),
                    )
                } else {
                    (
                        Some(MountTypeEnum::BIND),
                        Some(bollard::service::MountBindOptions {
                            propagation: None,
                            non_recursive: None,
                            create_mountpoint: None,
                            read_only_force_recursive: None,
                            read_only_non_recursive: None,
                        }),
                        None,
                    )
                };
                Mount {
                    target: Some(v.container_path.clone()),
                    source: Some(v.host_path.clone()),
                    typ,
                    read_only: if v.read_only { Some(true) } else { None },
                    consistency: None,
                    bind_options,
                    volume_options,
                    tmpfs_options: None,
                    image_options: None,
                }
            })
            .collect()
    }
}
