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

    loop {
        tokio::select! {
            Some(entry) = rx.recv() => {
                if let Some(ref tap_tx) = tap {
                    let _ = tap_tx.send(entry.clone());
                }
                buffer.push(entry);
                if buffer.len() >= buffer_size {
                    flush(&conn, &mut buffer).await;
                }
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    flush(&conn, &mut buffer).await;
                }
            }
            else => {
                if !buffer.is_empty() {
                    flush(&conn, &mut buffer).await;
                }
                break;
            }
        }
    }
}

async fn flush(conn: &DatabaseConnection, buffer: &mut Vec<LogEntry>) {
    if buffer.is_empty() {
        return;
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

    if let Err(e) = conn.execute_raw(stmt).await {
        let _ = std::io::stderr()
            .write_all(format!("[pg_log_writer] flush failed: {}\n", e).as_bytes());
    }
}

pub async fn cleanup_old_logs(conn: &DatabaseConnection, retention_days: u32) -> Result<u64> {
    let stmt = Statement::from_sql_and_values(
        conn.get_database_backend(),
        "DELETE FROM log.entries WHERE created_at < NOW() - ($1::text || ' days')::interval",
        [(retention_days as i64).into()],
    );
    let result = conn
        .execute_raw(stmt)
        .await
        .map_err(|e| anyhow!("Failed to clean up old logs: {}", e))?;
    Ok(result.rows_affected())
}
