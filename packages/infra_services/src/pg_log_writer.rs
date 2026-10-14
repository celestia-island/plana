use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{io::Write, time::Duration};
use tokio::sync::mpsc;

use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

const DEFAULT_BUFFER_SIZE: usize = 50;
const DEFAULT_FLUSH_INTERVAL_MS: u64 = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub source: String,
    pub instance_uuid: Option<String>,
    pub level: String,
    pub target: Option<String>,
    pub message: String,
    pub fields: serde_json::Value,
    pub created_at: String,
}

pub struct PgLogWriter {
    tx: mpsc::UnboundedSender<LogEntry>,
}

impl PgLogWriter {
    pub fn new(
        conn: DatabaseConnection,
        buffer_size: Option<usize>,
        flush_interval: Option<Duration>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let buf_sz = buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE);
        let interval = flush_interval.unwrap_or(Duration::from_millis(DEFAULT_FLUSH_INTERVAL_MS));

        tokio::spawn(writer_task(conn, rx, buf_sz, interval, None));

        Self { tx }
    }

    pub fn channel() -> (
        mpsc::UnboundedSender<LogEntry>,
        mpsc::UnboundedReceiver<LogEntry>,
    ) {
        mpsc::unbounded_channel()
    }

    pub fn from_receiver(
        rx: mpsc::UnboundedReceiver<LogEntry>,
        conn: DatabaseConnection,
        buffer_size: Option<usize>,
        flush_interval: Option<Duration>,
    ) -> Self {
        let (tx, _) = mpsc::unbounded_channel();
        let buf_sz = buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE);
        let interval = flush_interval.unwrap_or(Duration::from_millis(DEFAULT_FLUSH_INTERVAL_MS));

        tokio::spawn(writer_task(conn, rx, buf_sz, interval, None));

        Self { tx }
    }

    pub fn from_receiver_with_tap(
        rx: mpsc::UnboundedReceiver<LogEntry>,
        conn: DatabaseConnection,
        buffer_size: Option<usize>,
        flush_interval: Option<Duration>,
        tap: mpsc::UnboundedSender<LogEntry>,
    ) -> Self {
        let (tx, _) = mpsc::unbounded_channel();
        let buf_sz = buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE);
        let interval = flush_interval.unwrap_or(Duration::from_millis(DEFAULT_FLUSH_INTERVAL_MS));

        tokio::spawn(writer_task(conn, rx, buf_sz, interval, Some(tap)));

        Self { tx }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<LogEntry> {
        self.tx.clone()
    }

    pub fn write(&self, entry: LogEntry) {
        if self.tx.send(entry).is_err() {
            let _ = std::io::stderr()
                .write_all(b"[pg_log_writer] channel closed, dropping log entry\n");
        }
    }

    pub fn write_batch(&self, entries: Vec<LogEntry>) {
        for entry in entries {
            self.write(entry);
        }
    }
}

async fn writer_task(
    conn: DatabaseConnection,
    mut rx: mpsc::UnboundedReceiver<LogEntry>,
    buffer_size: usize,
    flush_interval: Duration,
    tap: Option<mpsc::UnboundedSender<LogEntry>>,
) {
    let mut buffer: Vec<LogEntry> = Vec::with_capacity(buffer_size);
    let mut interval = tokio::time::interval(flush_interval);
    interval.tick().await;

    // Circuit-breaker state (2026-10-14 finding): entelecheia's
    // `m20261009_drop_legacy_log_schema` migration dropped `log.entries`,
    // but this writer kept flushing every 500 ms — every flush failed and
    // reported to stderr, ~2 lines/second forever, 1.2 GB of container
    // log in four days on an otherwise IDLE instance. A missing target
    // table can never self-heal, so it disables the DB sink permanently
    // (the tap keeps serving subscriptions); other failures stay retried
    // with rate-limited reporting.
    let mut db_enabled = true;
    let mut consecutive_failures: u64 = 0;

    loop {
        tokio::select! {
            Some(entry) = rx.recv() => {
                if let Some(ref tap_tx) = tap {
                    let _ = tap_tx.send(entry.clone());
                }
                buffer.push(entry);
                if buffer.len() >= buffer_size {
                    drain(&conn, &mut buffer, &mut db_enabled, &mut consecutive_failures).await;
                }
            }
            _ = interval.tick() => {
                drain(&conn, &mut buffer, &mut db_enabled, &mut consecutive_failures).await;
            }
            else => {
                drain(&conn, &mut buffer, &mut db_enabled, &mut consecutive_failures).await;
                break;
            }
        }
    }
}

/// What a failed flush means for the DB sink's future.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SinkAction {
    /// The target relation does not exist (Postgres `42P01`-shaped
    /// "does not exist" errors): retrying can never succeed until a
    /// migration recreates the table — disable the sink permanently.
    Disable,
    /// Transient-looking failure: keep retrying, report rate-limited.
    Keep,
}

/// Pure classifier so the disable policy is unit-pinnable.
fn classify_flush_failure(err: &sea_orm::DbErr) -> SinkAction {
    if err.to_string().contains("does not exist") {
        SinkAction::Disable
    } else {
        SinkAction::Keep
    }
}

/// One buffered flush attempt, breaker-aware. Entries are dropped either
/// way once attempted (same loss semantics as the previous writer) — the
/// tap already received its copy on ingest.
async fn drain(
    conn: &DatabaseConnection,
    buffer: &mut Vec<LogEntry>,
    db_enabled: &mut bool,
    consecutive_failures: &mut u64,
) {
    if !*db_enabled {
        buffer.clear();
        return;
    }
    if buffer.is_empty() {
        return;
    }
    match flush(conn, buffer).await {
        Ok(()) => {
            *consecutive_failures = 0;
        }
        Err(e) => {
            *consecutive_failures += 1;
            match classify_flush_failure(&e) {
                SinkAction::Disable => {
                    *db_enabled = false;
                    let _ = std::io::stderr().write_all(
                        format!("[pg_log_writer] DB sink disabled (target table missing?): {e}\n")
                            .as_bytes(),
                    );
                }
                SinkAction::Keep => {
                    // First failure and then every 1000th: a sustained
                    // outage must not reproduce the line-per-flush spam.
                    if *consecutive_failures == 1 || (*consecutive_failures).is_multiple_of(1000) {
                        let _ = std::io::stderr().write_all(
                            format!(
                                "[pg_log_writer] flush failed ({} consecutive): {e}\n",
                                *consecutive_failures
                            )
                            .as_bytes(),
                        );
                    }
                }
            }
        }
    }
}

async fn flush(
    conn: &DatabaseConnection,
    buffer: &mut Vec<LogEntry>,
) -> Result<(), sea_orm::DbErr> {
    if buffer.is_empty() {
        return Ok(());
    }

    let mut values_parts: Vec<String> = Vec::with_capacity(buffer.len());
    let mut param_idx = 1;
    let mut params: Vec<sea_orm::Value> = Vec::new();

    for entry in buffer.drain(..) {
        values_parts.push(format!(
            "(${},${},${},${},${},${}::jsonb,${}::timestamptz)",
            param_idx,
            param_idx + 1,
            param_idx + 2,
            param_idx + 3,
            param_idx + 4,
            param_idx + 5,
            param_idx + 6,
        ));
        param_idx += 7;

        params.push(entry.source.into());
        params.push(entry.instance_uuid.unwrap_or_default().into());
        params.push(entry.level.into());
        params.push(entry.target.unwrap_or_default().into());
        params.push(entry.message.into());
        params.push(
            serde_json::to_string(&entry.fields)
                .unwrap_or_else(|_| "{}".into())
                .into(),
        );
        params.push(entry.created_at.into());
    }

    let sql = format!(
        "INSERT INTO log.entries (source, instance_uuid, level, target, message, fields, created_at) VALUES {}",
        values_parts.join(", ")
    );

    let stmt = sea_orm::Statement::from_sql_and_values(conn.get_database_backend(), sql, params);

    conn.execute_raw(stmt).await.map(|_| ())
}

/// Batch size for [`cleanup_old_logs`]: each DELETE touches at most this
/// many rows, keeping every transaction short enough that concurrent
/// INSERTs never queue behind it.
pub const LOG_CLEANUP_BATCH: i64 = 5000;

pub async fn cleanup_old_logs(conn: &DatabaseConnection, retention_days: u32) -> Result<u64> {
    // Bounded batches (the 2026-10-07 incident review): a single
    // unbounded DELETE over an accumulated backlog holds its lock long
    // enough to block EVERY concurrent log INSERT — the insert queue
    // stalls, the WARN about slow inserts generates more inserts, and
    // the storm self-excites (~100k rows/s observed). Deleting in
    // bounded chunks instead keeps each transaction short; the loop
    // drains the backlog to completion and reports the total.
    let mut total: u64 = 0;
    loop {
        let stmt = Statement::from_sql_and_values(
            conn.get_database_backend(),
            "DELETE FROM log.entries WHERE id IN (\
             SELECT id FROM log.entries \
             WHERE created_at < NOW() - ($1::text || ' days')::interval \
             LIMIT $2)",
            [(retention_days as i64).into(), LOG_CLEANUP_BATCH.into()],
        );
        let n = conn
            .execute_raw(stmt)
            .await
            .map_err(|e| anyhow!("Failed to clean up old logs: {}", e))?
            .rows_affected();
        total += n;
        if (n as i64) < LOG_CLEANUP_BATCH {
            break;
        }
    }
    Ok(total)
}

#[cfg(test)]
mod cleanup_tests {
    use super::*;

    /// The breaker policy, pinned pure: a "does not exist" failure
    /// (Postgres 42P01 — the exact entelecheia `m20261009` aftermath that
    /// spammed 1.2 GB of container log) permanently disables the DB sink;
    /// every other failure class keeps retrying.
    #[test]
    fn missing_relation_disables_the_sink_other_errors_retry() {
        assert_eq!(
            classify_flush_failure(&sea_orm::DbErr::Custom(
                "error returned from database: relation \"log.entries\" does not exist".into()
            )),
            SinkAction::Disable
        );
        assert_eq!(
            classify_flush_failure(&sea_orm::DbErr::Custom(
                "connection refused (os error 111)".into()
            )),
            SinkAction::Keep
        );
        assert_eq!(
            classify_flush_failure(&sea_orm::DbErr::Custom(String::new())),
            SinkAction::Keep
        );
    }

    /// The batching loop is verified end-to-end against a real PG when
    /// `PLANA_LOG_PG_URL` is set (the harness seeds 12000 rows deeper
    /// than the retention window — more than LOG_CLEANUP_BATCH — and
    /// asserts the full backlog drains to zero while a fresh row inside
    /// the window survives).
    #[tokio::test]
    async fn cleanup_old_logs_drains_a_backlog_in_batches() {
        let Ok(url) = std::env::var("PLANA_LOG_PG_URL") else {
            eprintln!("SKIP: set PLANA_LOG_PG_URL to run the batching drain test");
            return;
        };
        let conn = sea_orm::Database::connect(url)
            .await
            .expect("connect test pg");

        // Seed: 12000 stale rows (older than the 30-day window used
        // below) + one fresh row that must survive.
        conn.execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "TRUNCATE log.entries RESTART IDENTITY".to_string(),
        ))
        .await
        .expect("truncate");
        conn.execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "INSERT INTO log.entries (source, level, message, created_at) \
             SELECT 'test', 'info', 'stale', NOW() - (g || ' days')::interval \
             FROM generate_series(1, 12000) g"
                .to_string(),
        ))
        .await
        .expect("seed stale");
        conn.execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "INSERT INTO log.entries (source, level, message) VALUES ('test', 'info', 'fresh')"
                .to_string(),
        ))
        .await
        .expect("seed fresh");

        // The 12000-row fixture exceeds LOG_CLEANUP_BATCH (5000), so
        // the drain spans multiple batches. The fixture spreads rows at
        // g days back for g in 1..=12000, so
        // under a 30-day retention exactly g=30..=12000 (11971 rows) is
        // stale and g=1..=29 plus the fresh row (30 total) survive.
        let deleted = cleanup_old_logs(&conn, 30).await.expect("cleanup");
        assert_eq!(
            deleted, 11971,
            "the whole stale backlog drains (multi-batch)"
        );

        let remaining = conn
            .query_one_raw(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT count(*) AS n, count(*) FILTER (WHERE message = 'fresh') AS f FROM log.entries".to_string(),
            ))
            .await
            .expect("count")
            .expect("row");
        let n: i64 = remaining.try_get("", "n").expect("n");
        let f: i64 = remaining.try_get("", "f").expect("f");
        assert_eq!(
            (n, f),
            (30, 1),
            "the in-window rows and the fresh row survive"
        );
    }
}
