use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct Session {
    pub id: Uuid,
}

pub struct SessionManager {
    sessions: RwLock<HashMap<Uuid, Arc<Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create(&self) -> Arc<Session> {
        let session = Arc::new(Session { id: Uuid::now_v7() });
        self.sessions
            .write()
            .await
            .insert(session.id, session.clone());
        session
    }

    pub async fn get(&self, id: &Uuid) -> Option<Arc<Session>> {
        self.sessions.read().await.get(id).cloned()
    }

    pub async fn remove(&self, id: &Uuid) -> Option<Arc<Session>> {
        self.sessions.write().await.remove(id)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
