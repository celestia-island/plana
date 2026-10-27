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

    /// Soft-offline every `agents` row whose `last_heartbeat` is older than
    /// `stale_after_secs` seconds AND whose `status` is not already
    /// `Offline`. Returns the number of rows newly marked offline.
    ///
    /// Backs the scepter agent-cleanup loop (`scepter-stale-offline`): the
    /// in-process heartbeat monitor only watches the in-memory
    /// `PoleMosState` workspace/device nodes — the persisted `agents` rows
    /// were accumulating forever with `status = 'Online'`.
    ///
    /// Indexing NOTE: a partial index on
    /// `agents (last_heartbeat) WHERE status <> 'Offline'` (created in
    /// scepter migration `m20261010_create_stale_offline_index`) keeps
    /// this an index-only scan even after the table grows. The index
    /// predicate has to match the WHERE clause (Postgres's partial-index
    /// planner does not infer `status <> 'Offline'` from
    /// `status = 'Online'`); a narrow `WHERE status = 'Online'` index
    /// will NOT be picked. Keep them in lockstep when either is changed.
    ///
    /// Bound NOTE: `stale_after_secs` is a `u64` at the API boundary but
    /// bound as `i64` for `make_interval(secs => $1)` — any value above
    /// `i64::MAX` (~9.22e18) saturates into a negative interval and the
    /// statement will fail at the database. Callers should pass a sane
    /// wall-clock duration (minutes to hours, not epoch seconds).
    ///
    /// Status casing NOTE: this writes the bare `Offline` — the same
    /// literal that `plana_state_sync::AgentStatus::Offline` produces via
    /// its `Display` impl and that `save_agent`/`update_agent_status`
    /// persist. scepter's persisted data is therefore capitalised
    /// (`Initializing` / `Online` / `Busy` / `Offline` / `Error`), not
    /// lower-case. The earlier `get_online_agents` query used
    /// `status = 'online'` and matched nothing in production — that was
    /// a pre-existing latent bug, also fixed in this PR.
    ///
    /// Soft-delete only: rows stay in the table for audit / re-registration
    /// to keep the same `id` and to keep `get_online_agents` (which
    /// filters on `status = 'Online'`) the single source of truth for
    /// "alive". Hard deletion is intentionally NOT provided here — scepter
    /// owns lifecycle and would need a separate decision.
    pub async fn mark_stale_agents_offline(&self, stale_after_secs: u64) -> Result<u64> {
        let sql = r#"
            WITH moved AS (
                UPDATE agents
                SET status = 'Offline',
                    updated_at = NOW()
                WHERE status <> 'Offline'
                  AND last_heartbeat < NOW() - make_interval(secs => $1)
                RETURNING 1
            )
            SELECT count(*)::bigint FROM moved
        "#;

        let stmt = sea_orm::Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            sql,
            [(stale_after_secs as i64).into()],
        );

        let row = self
            .conn
            .query_one_raw(stmt)
            .await
            .map_err(|e| anyhow!("Failed to mark stale agents offline: {}", e))?
            .ok_or_else(|| anyhow!("mark_stale_agents_offline: no row returned"))?;

        let count: i64 = row
            .try_get("", "count")
            .map_err(|e| anyhow!("mark_stale_agents_offline: missing count: {}", e))?;

        Ok(count.max(0) as u64)
    }

    pub async fn get_online_agents(&self) -> Result<Vec<OnlineAgentInfo>> {
        // Status casing: scepter persists `Online` (via `Display` of
        // `plana_state_sync::AgentStatus::Online`). The earlier `status =
        // 'online'` filter matched nothing in production and was a
        // pre-existing latent bug; this filter uses the same capitalised
        // literal the scepter write path produces.
        let sql = r#"
            SELECT agent_type, agent_id, started_at, last_heartbeat
            FROM agents
            WHERE status = 'Online'
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
            // `agents.started_at` / `last_heartbeat` are TIMESTAMPTZ
            // (scepter baseline schema). Decoding them as `String` failed
            // for EVERY row ("mismatched types"), so each `list_agents`
            // call spammed one warn per row into the journal AND returned
            // an empty roster — the 2026-10-27 journal-spam finding.
            // Decode the timestamp and render RFC 3339; the wire shape of
            // `OnlineAgentInfo` (String fields) is unchanged.
            let started_at =
                match row.try_get::<chrono::DateTime<chrono::FixedOffset>>("", "started_at") {
                    Ok(v) => v.to_rfc3339(),
                    Err(e) => {
                        warn!(error = %e, "skipping row: missing started_at");
                        continue;
                    }
                };
            let last_heartbeat =
                match row.try_get::<chrono::DateTime<chrono::FixedOffset>>("", "last_heartbeat") {
                    Ok(v) => v.to_rfc3339(),
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

#[cfg(test)]
mod persistence_tests {
    use super::*;

    /// The 2026-10-27 journal-spam fix, pinned against a live database:
    /// `agents.started_at` / `last_heartbeat` are TIMESTAMPTZ (scepter's
    /// baseline DDL). The historical `String` decode failed for EVERY row
    /// ("mismatched types"), spamming one warn per row per `list_agents`
    /// call and returning an EMPTY roster; the fix decodes the timestamp
    /// and renders RFC 3339. Reverting to the `String` decode must turn
    /// this red (every row lands in the skip arms again).
    ///
    /// Skips without `DATABASE_URL` — same guard shape as the services'
    /// DB-backed tests. Builds the table inline because the production
    /// DDL lives in scepter's baseline schema, outside this repo.
    #[tokio::test]
    async fn get_online_agents_decodes_timestamptz_rows() -> Result<()> {
        let url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                eprintln!("SKIP: DATABASE_URL not set — TIMESTAMPTZ decode test skipped");
                return Ok(());
            }
        };
        // The pglite bridge multiplexes ONE in-memory session: cap the pool
        // to one connection and run the whole fixture as a single statement
        // — multiple pool round-trips degrade the bridge mid-test (ledger
        // known-env 坑①/坑② family). Idempotent (IF NOT EXISTS + DELETE)
        // so re-runs within one bridge lifetime work.
        let mut opts = sea_orm::ConnectOptions::new(url);
        opts.max_connections(1);
        let conn = sea_orm::Database::connect(opts)
            .await
            .map_err(|e| anyhow!("connect test db: {e}"))?;

        // The production baseline shape: TIMESTAMPTZ, one Online row.
        conn.execute_unprepared(
            "CREATE TEMP TABLE IF NOT EXISTS agents (
                id UUID PRIMARY KEY,
                agent_type TEXT NOT NULL,
                agent_id TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL,
                started_at TIMESTAMPTZ NOT NULL,
                last_heartbeat TIMESTAMPTZ NOT NULL,
                metadata JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL
            );
            DELETE FROM agents WHERE agent_id = 'decode-probe';
            INSERT INTO agents (id, agent_type, agent_id, status, started_at, last_heartbeat, metadata, created_at, updated_at)
             VALUES ('01800000-0000-7000-8000-000000000001', 'aporia', 'decode-probe', 'Online', NOW(), NOW(), '{}', NOW(), NOW());",
        )
        .await
        .map_err(|e| anyhow!("fixture: {e}"))?;

        let persistence = Persistence::new(conn);
        let agents = persistence.get_online_agents().await?;

        assert_eq!(agents.len(), 1, "the Online row must decode, not skip");
        let agent = &agents[0];
        assert_eq!(agent.agent_id, "decode-probe");
        // A real RFC 3339 timestamp (the old bug returned NO rows at all —
        // every row hit the skip arm before this line was reachable).
        chrono::DateTime::parse_from_rfc3339(&agent.started_at).unwrap_or_else(|e| {
            panic!(
                "started_at must be RFC 3339, got {:?}: {e}",
                agent.started_at
            )
        });
        chrono::DateTime::parse_from_rfc3339(&agent.last_heartbeat).unwrap_or_else(|e| {
            panic!(
                "last_heartbeat must be RFC 3339, got {:?}: {e}",
                agent.last_heartbeat
            )
        });
        Ok(())
    }
}
