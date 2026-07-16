use std::path::PathBuf;

fn not_available() -> Box<dyn std::error::Error + Send + Sync> {
    std::io::Error::new(std::io::ErrorKind::Unsupported, "YoukiManager is only available on Linux").into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootfsCapability {
    None,
    Overlay,
    Btrfs,
}

pub fn detect_inside_container() -> bool {
    false
}

pub fn detect_rootfs_capability(_: &PathBuf) -> RootfsCapability {
    RootfsCapability::None
}

pub struct YoukiManager;

impl YoukiManager {
    pub fn new(_data_dir: &std::path::Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err(not_available())
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err(not_available())
    }
}

fn err() -> _container::errors::ContainerError {
    _container::errors::ContainerError::OperationFailed {
        container_id: "n/a".into(),
        message: "YoukiManager not available on this platform".into(),
    }
}

#[async_trait::async_trait]
impl _container::ops::ContainerOps for YoukiManager {
    async fn create(&self, _: &_container::types::ContainerCreateParams) -> _container::errors::ContainerResult<_container::types::ContainerInfo> { Err(err()) }
    async fn start(&self, _: &str) -> _container::errors::ContainerResult<()> { Err(err()) }
    async fn stop(&self, _: &str) -> _container::errors::ContainerResult<()> { Err(err()) }
    async fn remove(&self, _: &str, _: bool) -> _container::errors::ContainerResult<()> { Err(err()) }
    async fn restart(&self, _: &str) -> _container::errors::ContainerResult<()> { Err(err()) }
    async fn list(&self) -> _container::errors::ContainerResult<Vec<_container::types::ContainerInfo>> { Ok(vec![]) }
    async fn list_with_filter(&self, _: Option<&str>, _: Option<std::collections::HashMap<String, String>>, _: bool) -> _container::errors::ContainerResult<Vec<_container::types::ContainerInfo>> { Ok(vec![]) }
    async fn inspect(&self, _: &str) -> _container::errors::ContainerResult<_container::types::ContainerDetail> { Err(err()) }
    async fn is_running(&self, _: &str) -> _container::errors::ContainerResult<bool> { Ok(false) }
    async fn exec(&self, _: &str, _: &[&str]) -> _container::errors::ContainerResult<_container::types::ExecOutput> { Err(err()) }
    async fn fork(&self, _: &_container::types::ContainerForkParams) -> _container::errors::ContainerResult<_container::types::ContainerInfo> { Err(err()) }
    async fn commit(&self, _: &str, _: &str, _: Option<&str>) -> _container::errors::ContainerResult<String> { Err(err()) }
    async fn commit_with_labels(&self, _: &str, _: &str, _: Option<&str>, _: Option<&std::collections::HashMap<String, String>>) -> _container::errors::ContainerResult<String> { Err(err()) }
    async fn wait_healthy(&self, _: &str, _: std::time::Duration) -> _container::errors::ContainerResult<()> { Err(err()) }
    async fn ensure_running(&self, _: &str) -> _container::errors::ContainerResult<_container::types::ContainerInfo> { Err(err()) }
    async fn recreate(&self, _: &str, _: &str) -> _container::errors::ContainerResult<_container::types::ContainerInfo> { Err(err()) }
    async fn list_images(&self) -> _container::errors::ContainerResult<Vec<_container::types::ImageInfo>> { Ok(vec![]) }
    async fn pull_image(&self, _: &str) -> _container::errors::ContainerResult<String> { Err(err()) }
    async fn image_exists(&self, _: &str) -> _container::errors::ContainerResult<bool> { Ok(false) }
    async fn remove_image(&self, _: &str, _: bool) -> _container::errors::ContainerResult<()> { Err(err()) }
    async fn remove_with_image(&self, _: &str, _: &str, _: bool) -> _container::errors::ContainerResult<()> { Err(err()) }
    async fn create_volume(&self, _: &str) -> _container::errors::ContainerResult<String> { Err(err()) }
    async fn remove_volume(&self, _: &str, _: bool) -> _container::errors::ContainerResult<()> { Err(err()) }
    async fn volume_exists(&self, _: &str) -> _container::errors::ContainerResult<bool> { Ok(false) }
    async fn list_volumes(&self) -> _container::errors::ContainerResult<Vec<_container::types::DockerVolumeInfo>> { Ok(vec![]) }
    async fn logs(&self, _: &str, _: usize) -> _container::errors::ContainerResult<Vec<String>> { Ok(vec![]) }
    async fn get_container_logs(&self, _: &str, _: usize) -> _container::errors::ContainerResult<String> { Ok(String::new()) }
    async fn writable_rootfs(&self, _: &str) -> _container::errors::ContainerResult<_container::types::WritableRootfs> { Err(err()) }
    async fn diff_workspace(&self, _: &str, _: &std::path::Path, _: &str) -> _container::errors::ContainerResult<Vec<_container::types::PathChange>> { Ok(vec![]) }
    async fn download_archive(&self, _: &str, _: &str) -> _container::errors::ContainerResult<Vec<u8>> { Err(err()) }
    async fn upload_archive(&self, _: &str, _: &str, _: Vec<u8>) -> _container::errors::ContainerResult<()> { Err(err()) }
    fn clone_boxed(&self) -> Box<dyn _container::ops::ContainerOps> { Box::new(YoukiManager) }
}
