use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use lacrymira::{Command, Dialect, Reply, Value};
use lacrymira_pg::PgExecutor;

use crate::types::{AgentMetadata, LogContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineAgentInfo {
    pub agent_type: String,
    pub agent_id: String,
    pub started_at: String,
    pub last_heartbeat: String,
}

pub struct Persistence {
    conn: PgExecutor,
}

/// Positional cell lookup by column name inside one decoded row.
fn cell<'a>(row: &'a [Value], columns: &[String], name: &str) -> Option<&'a Value> {
    let idx = columns.iter().position(|c| c == name)?;
    row.get(idx)
}

impl Persistence {
    pub fn new(conn: PgExecutor) -> Self {
        Self { conn }
    }

    pub fn get_connection(&self) -> &PgExecutor {
        &self.conn
    }

    /// Run one raw single-statement SQL command with bound parameters
    /// (the `Command::Sql` surface is prepared-statement based, so the
    /// text must be a single statement; PostgreSQL type casts stay in
    /// the SQL text, e.g. `$5::jsonb`).
    async fn exec(&self, sql: &str, params: Vec<Value>) -> Result<Reply, lacrymira::Error> {
        self.conn
            .execute(&Command::Sql {
                dialect: Dialect::Postgres,
                text: sql.to_string(),
                params,
            })
            .await
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

        self.exec(
            r#"
            INSERT INTO agents (id, agent_type, agent_id, status, started_at, last_heartbeat, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW(), $5::jsonb, NOW(), NOW())
            ON CONFLICT (agent_id) DO UPDATE SET
                status = EXCLUDED.status,
                last_heartbeat = EXCLUDED.last_heartbeat,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            "#,
            vec![
                Value::Uuid(id.to_string()),
                Value::Text(agent_type.to_string()),
                Value::Text(agent_id.to_string()),
                Value::Text(status.to_string()),
                Value::Text(metadata_json),
            ],
        )
        .await
        .map_err(|e| anyhow!("Failed to save agent info: {}", e))?;

        Ok(())
    }

    pub async fn update_agent_status(&self, agent_id: &str, status: &str) -> Result<()> {
        self.exec(
            r#"
            UPDATE agents
            SET status = $1, last_heartbeat = NOW()
            WHERE agent_id = $2
            "#,
            vec![
                Value::Text(status.to_string()),
                Value::Text(agent_id.to_string()),
            ],
        )
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

        self.exec(
            r#"
            INSERT INTO logs (level, agent_type, agent_id, message, context, created_at)
            VALUES ($1, $2, $3, $4, $5::jsonb, NOW())
            "#,
            vec![
                Value::Text(level.to_string()),
                Value::Text(agent_type_val.to_string()),
                Value::Text(agent_id_val.to_string()),
                Value::Text(message.to_string()),
                Value::Text(context_value),
            ],
        )
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

        let reply = self
            .exec(sql, vec![Value::I64(stale_after_secs as i64)])
            .await
            .map_err(|e| anyhow!("Failed to mark stale agents offline: {}", e))?;

        let count: i64 = match &reply {
            Reply::Rows { columns, rows } if !rows.is_empty() => {
                match cell(&rows[0], columns, "count") {
                    Some(Value::I64(v)) => *v,
                    other => {
                        return Err(anyhow!(
                            "mark_stale_agents_offline: missing count: got {:?}",
                            other
                        ));
                    }
                }
            }
            _ => return Err(anyhow!("mark_stale_agents_offline: no row returned")),
        };

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

        let reply = self
            .exec(sql, Vec::new())
            .await
            .map_err(|e| anyhow!("Failed to query online agents: {}", e))?;

        let (columns, rows) = match reply {
            Reply::Rows { columns, rows } => (columns, rows),
            _ => return Ok(Vec::new()),
        };

        let mut agents: Vec<OnlineAgentInfo> = Vec::new();
        for row in &rows {
            let agent_type: String = match cell(row, &columns, "agent_type") {
                Some(Value::Text(v)) => v.clone(),
                other => {
                    warn!(value = ?other, "skipping row: missing agent_type");
                    continue;
                }
            };
            let agent_id: String = match cell(row, &columns, "agent_id") {
                Some(Value::Text(v)) => v.clone(),
                other => {
                    warn!(value = ?other, "skipping row: missing agent_id");
                    continue;
                }
            };
            // `agents.started_at` / `last_heartbeat` are TIMESTAMPTZ
            // (scepter baseline schema). lacrymira decodes them as RFC
            // 3339 strings (`Value::Timestamp`); the wire shape of
            // `OnlineAgentInfo` (String fields) is unchanged. The skip
            // arms keep the 2026-10-27 journal-spam shape — a column
            // that decodes to anything else must warn-and-skip, never
            // fail the whole roster (nor return an empty one silently).
            let started_at = match cell(row, &columns, "started_at") {
                Some(Value::Timestamp(v)) => v.clone(),
                other => {
                    warn!(value = ?other, "skipping row: missing started_at");
                    continue;
                }
            };
            let last_heartbeat = match cell(row, &columns, "last_heartbeat") {
                Some(Value::Timestamp(v)) => v.clone(),
                other => {
                    warn!(value = ?other, "skipping row: missing last_heartbeat");
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
    /// and renders RFC 3339. Reverting to a decode that cannot produce a
    /// timestamp must turn this red (every row lands in the skip arms
    /// again).
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
        // to one connection and keep the fixture a single multi-statement
        // round-trip via the raw pool (the `Command::Sql` surface is
        // prepared-statement single-statement only) — multiple pool
        // round-trips degrade the bridge mid-test (ledger known-env 坑①/坑②
        // family). Idempotent (IF NOT EXISTS + DELETE) so re-runs within
        // one bridge lifetime work.
        let conn = PgExecutor::connect_with(
            &url,
            sqlx::postgres::PgPoolOptions::new().max_connections(1),
        )
        .await
        .map_err(|e| anyhow!("connect test db: {e}"))?;

        // The production baseline shape: TIMESTAMPTZ, one Online row.
        sqlx::raw_sql(
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
        .execute(conn.pool())
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
