use super::{
    errors::{ContainerError, ContainerResult},
    manager::ContainerManager,
};

impl ContainerManager {
    pub async fn list_volumes(&self) -> ContainerResult<Vec<super::types::DockerVolumeInfo>> {
        let options = bollard::query_parameters::ListVolumesOptions {
            ..Default::default()
        };
        let result = self.docker.list_volumes(Some(options)).await?;
        let volumes = result
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|v| super::types::DockerVolumeInfo {
                name: v.name,
                driver: v.driver,
                mountpoint: Some(v.mountpoint),
            })
            .collect();
        Ok(volumes)
    }

    pub async fn create_volume(&self, name: &str) -> ContainerResult<String> {
        let options = bollard::models::VolumeCreateRequest {
            name: Some(name.to_string()),
            driver: Some("local".to_string()),
            ..Default::default()
        };
        let vol = self
            .docker
            .create_volume(options)
            .await
            .map_err(|e| ContainerError::VolumeFailed(e.to_string()))?;
        Ok(vol.name)
    }

    pub async fn remove_volume(&self, name: &str, force: bool) -> ContainerResult<()> {
        self.docker
            .remove_volume(
                name,
                Some(bollard::query_parameters::RemoveVolumeOptions { force }),
            )
            .await
            .map_err(|e| ContainerError::VolumeFailed(e.to_string()))?;
        Ok(())
    }

    pub async fn volume_exists(&self, name: &str) -> ContainerResult<bool> {
        match self.docker.inspect_volume(name).await {
            Ok(_) => Ok(true),
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => Ok(false),
            Err(e) => Err(ContainerError::VolumeFailed(e.to_string())),
        }
    }
}
