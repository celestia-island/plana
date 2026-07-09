use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use sea_orm::{ConnectionTrait, DatabaseConnection};

use crate::types::{AgentMetadata, LogContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineAgentInfo {
    pub agent_type: String,
    pub agent_id: String,
    pub started_at: String,
    pub last_heartbeat: String,
}

pub struct Persistence {
    conn: DatabaseConnection,
}

impl Persistence {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub async fn save_agent(
        &self,
        agent_type: &str,
        agent_id: &str,
        status: &str,
        metadata: AgentMetadata,
    ) -> Result<()> {
        let metadata_json = serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string());
        let id = Uuid::now_v7();

        let stmt = sea_orm::Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO agents (id, agent_type, agent_id, status, started_at, last_heartbeat, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW(), $5::jsonb, NOW(), NOW())
            ON CONFLICT (agent_id) DO UPDATE SET
                status = EXCLUDED.status,
                last_heartbeat = EXCLUDED.last_heartbeat,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            "#,
            [
                id.into(),
                agent_type.into(),
                agent_id.into(),
                status.into(),
                metadata_json.into(),
            ],
        );

        self.conn
            .execute_raw(stmt)
            .await
            .map_err(|e| anyhow!("Failed to save agent info: {}", e))?;

        Ok(())
    }

    pub async fn update_agent_status(&self, agent_id: &str, status: &str) -> Result<()> {
        let stmt = sea_orm::Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            UPDATE agents
            SET status = $1, last_heartbeat = NOW()
            WHERE agent_id = $2
            "#,
            [status.into(), agent_id.into()],
        );

        self.conn
            .execute_raw(stmt)
            .await
            .map_err(|e| anyhow!("Failed to update agent status: {}", e))?;

        Ok(())
    }

    pub async fn log(
        &self,
        level: &str,
        agent_type: Option<&str>,
        agent_id: Option<&str>,
        message: &str,
        context: LogContext,
    ) -> Result<()> {
        let context_value = serde_json::to_string(&context).unwrap_or_else(|_| "{}".to_string());
        let agent_type_val = agent_type.unwrap_or("");
        let agent_id_val = agent_id.unwrap_or("");

        let stmt = sea_orm::Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO logs (level, agent_type, agent_id, message, context, created_at)
            VALUES ($1, $2, $3, $4, $5::jsonb, NOW())
            "#,
            [
                level.into(),
                agent_type_val.into(),
                agent_id_val.into(),
                message.into(),
                context_value.into(),
            ],
        );

        self.conn
            .execute_raw(stmt)
            .await
            .map_err(|e| anyhow!("Failed to log entry: {}", e))?;

        Ok(())
    }

    pub async fn get_online_agents(&self) -> Result<Vec<OnlineAgentInfo>> {
        let sql = r#"
            SELECT agent_type, agent_id, started_at, last_heartbeat
            FROM agents
            WHERE status = 'online'
            ORDER BY agent_type
        "#;

        let stmt = sea_orm::Statement::from_string(self.conn.get_database_backend(), sql);

        let rows = self
            .conn
            .query_all_raw(stmt)
            .await
            .map_err(|e| anyhow!("Failed to query online agents: {}", e))?;

        let mut agents: Vec<OnlineAgentInfo> = Vec::new();
        for row in &rows {
            let agent_type: String = match row.try_get("", "agent_type") {
                Ok(v) => v,
                Err(e) => {
                    warn!(error = %e, "skipping row: missing agent_type");
                    continue;
                }
            };
            let agent_id: String = match row.try_get("", "agent_id") {
                Ok(v) => v,
                Err(e) => {
                    warn!(error = %e, "skipping row: missing agent_id");
                    continue;
                }
            };
            let started_at: String = match row.try_get("", "started_at") {
                Ok(v) => v,
                Err(e) => {
                    warn!(error = %e, "skipping row: missing started_at");
                    continue;
                }
            };
            let last_heartbeat: String = match row.try_get("", "last_heartbeat") {
                Ok(v) => v,
                Err(e) => {
                    warn!(error = %e, "skipping row: missing last_heartbeat");
                    continue;
                }
            };

            agents.push(OnlineAgentInfo {
                agent_type,
                agent_id,
                started_at,
                last_heartbeat,
            });
        }

        Ok(agents)
    }
}
