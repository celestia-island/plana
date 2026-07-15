use serde::{Deserialize, Serialize};

use super::types::ContainerStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerEvent {
    Created {
        id: String,
        name: String,
        image: String,
    },
    Started {
        id: String,
    },
    Stopped {
        id: String,
    },
    Died {
        id: String,
        exit_code: i64,
    },
    Destroyed {
        id: String,
    },
    Renamed {
        id: String,
        old_name: String,
        new_name: String,
    },
    StatusChanged {
        id: String,
        old_status: ContainerStatus,
        new_status: ContainerStatus,
    },
    Updated {
        id: String,
        name: String,
        old_image: String,
        new_image: String,
    },
}

impl ContainerEvent {
    pub fn container_id(&self) -> &str {
        match self {
            Self::Created { id, .. } => id,
            Self::Started { id } => id,
            Self::Stopped { id } => id,
            Self::Died { id, .. } => id,
            Self::Destroyed { id } => id,
            Self::Renamed { id, .. } => id,
            Self::StatusChanged { id, .. } => id,
            Self::Updated { id, .. } => id,
        }
    }
}
