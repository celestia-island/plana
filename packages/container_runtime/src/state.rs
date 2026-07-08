use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use _container::types::{ContainerInfo, ContainerStatus};

#[derive(Debug, Clone)]
pub struct YoukiContainerRecord {
    pub info: ContainerInfo,
    pub bundle_path: std::path::PathBuf,
    pub rootfs_path: std::path::PathBuf,
    pub pid: Option<i32>,
    pub exit_code: Option<i64>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct YoukiState {
    containers: Arc<RwLock<HashMap<String, YoukiContainerRecord>>>,
}

impl Default for YoukiState {
    fn default() -> Self {
        Self::new()
    }
}

impl YoukiState {
    pub fn new() -> Self {
        Self {
            containers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, record: YoukiContainerRecord) {
        let mut state = self.containers.write().await;
        state.insert(record.info.id.clone(), record);
    }

    pub async fn remove(&self, id: &str) -> Option<YoukiContainerRecord> {
        let mut state = self.containers.write().await;
        state.remove(id)
    }

    pub async fn get(&self, id: &str) -> Option<YoukiContainerRecord> {
        let state = self.containers.read().await;
        state.get(id).cloned()
    }

    pub async fn get_by_name(&self, name: &str) -> Option<YoukiContainerRecord> {
        let state = self.containers.read().await;
        state
            .values()
            .find(|r| r.info.name == name || r.info.name == format!("/{}", name))
            .cloned()
    }

    pub async fn list_all(&self) -> Vec<ContainerInfo> {
        let state = self.containers.read().await;
        state.values().map(|r| r.info.clone()).collect()
    }

    pub async fn list_with_filter(
        &self,
        name_prefix: Option<&str>,
        label_filter: Option<&HashMap<String, String>>,
        all: bool,
    ) -> Vec<ContainerInfo> {
        let state = self.containers.read().await;
        state
            .values()
            .filter(|r| {
                if let Some(prefix) = name_prefix {
                    let name = r.info.name.trim_start_matches('/');
                    if !name.starts_with(prefix) {
                        return false;
                    }
                }
                if let Some(labels) = label_filter {
                    for (k, v) in labels {
                        match r.info.labels.get(k) {
                            Some(lv) if v.is_empty() || lv == v => {},
                            _ => return false,
                        }
                    }
                }
                if !all && r.info.status != ContainerStatus::Running {
                    return false;
                }
                true
            })
            .map(|r| r.info.clone())
            .collect()
    }

    pub async fn update_status(&self, id: &str, status: ContainerStatus) {
        let mut state = self.containers.write().await;
        if let Some(record) = state.get_mut(id) {
            record.info.status = status;
        }
    }

    pub async fn update_pid(&self, id: &str, pid: Option<i32>) {
        let mut state = self.containers.write().await;
        if let Some(record) = state.get_mut(id) {
            record.pid = pid;
        }
    }

    pub async fn update_exit_status(
        &self,
        id: &str,
        exit_code: Option<i64>,
        finished_at: Option<String>,
        error: Option<String>,
    ) {
        let mut state = self.containers.write().await;
        if let Some(record) = state.get_mut(id) {
            record.exit_code = exit_code;
            record.finished_at = finished_at;
            record.error = error;
        }
    }

    pub async fn clear(&self) {
        let mut state = self.containers.write().await;
        state.clear();
    }

    pub async fn replace_all(&self, records: Vec<YoukiContainerRecord>) {
        let mut state = self.containers.write().await;
        state.clear();
        for record in records {
            state.insert(record.info.id.clone(), record);
        }
    }
}
