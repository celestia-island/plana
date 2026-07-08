use chrono::Utc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use tracing::warn;

use super::tool_security::SecurityAction;

pub trait AuditSink: Send + Sync {
    fn flush(&self, entries: &[ToolAuditEntry]);
    fn query(&self, agent_filter: Option<&str>, limit: usize) -> Vec<ToolAuditEntry>;
}

pub struct JsonlAuditSink {
    path: std::path::PathBuf,
}

impl JsonlAuditSink {
    pub fn new(path: impl Into<std::path::PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self { path })
    }
}

impl AuditSink for JsonlAuditSink {
    fn flush(&self, entries: &[ToolAuditEntry]) {
        if entries.is_empty() {
            return;
        }
        let mut file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(f) => f,
            Err(e) => {
                warn!(error = %e, path = %self.path.display(), "audit sink: failed to open file");
                return;
            },
        };
        use std::io::Write;
        for entry in entries {
            match serde_json::to_string(entry) {
                Ok(line) => {
                    if let Err(e) = writeln!(file, "{line}") {
                        warn!(error = %e, "audit sink: write failed");
                    }
                },
                Err(e) => warn!(error = %e, "audit sink: serialization failed"),
            }
        }
    }

    fn query(&self, agent_filter: Option<&str>, limit: usize) -> Vec<ToolAuditEntry> {
        let file = match std::fs::File::open(&self.path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };
        let reader = std::io::BufReader::new(file);
        let mut entries: Vec<ToolAuditEntry> = Vec::new();
        for line in std::io::BufRead::lines(reader) {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            if let Ok(entry) = serde_json::from_str::<ToolAuditEntry>(&line) {
                if let Some(agent) = agent_filter
                    && entry.agent != agent
                {
                    continue;
                }
                entries.push(entry);
            }
        }
        entries.reverse();
        entries.truncate(limit);
        entries
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAuditEntry {
    pub timestamp: String,
    pub agent: String,
    pub tool: String,
    pub params_hash: String,
    pub action: SecurityAction,
    pub reason: String,
    pub matched_rule: Option<String>,
    pub session_id: Option<String>,
}

impl ToolAuditEntry {
    pub fn new(
        agent: &str,
        tool: &str,
        params_hash: &str,
        action: SecurityAction,
        reason: String,
        matched_rule: Option<String>,
        session_id: Option<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            agent: agent.to_string(),
            tool: tool.to_string(),
            params_hash: params_hash.to_string(),
            action,
            reason,
            matched_rule,
            session_id,
        }
    }
}

pub struct ToolAuditLog {
    entries: Mutex<Vec<ToolAuditEntry>>,
    max_entries: usize,
    sink: Mutex<Option<Arc<dyn AuditSink>>>,
}

impl ToolAuditLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Mutex::new(Vec::with_capacity(max_entries.min(1000))),
            max_entries,
            sink: Mutex::new(None),
        }
    }

    pub fn with_sink(self, sink: Arc<dyn AuditSink>) -> Self {
        *self.sink.lock() = Some(sink);
        self
    }

    pub fn set_sink(&self, sink: Arc<dyn AuditSink>) {
        let mut guard = self.entries.lock();
        if !guard.is_empty() {
            sink.flush(&guard);
        }
        guard.clear();
        drop(guard);
        *self.sink.lock() = Some(sink);
    }

    pub fn record(&self, entry: ToolAuditEntry) {
        let sink = self.sink.lock().clone();
        let mut guard = self.entries.lock();
        if guard.len() >= self.max_entries {
            let evicted: Vec<ToolAuditEntry> = guard.drain(0..1).collect();
            if let Some(ref s) = sink {
                s.flush(&evicted);
            }
        }
        guard.push(entry);
    }

    pub fn flush_all(&self) {
        let sink = self.sink.lock().clone();
        if let Some(ref s) = sink {
            let mut guard = self.entries.lock();
            if !guard.is_empty() {
                s.flush(&guard);
                guard.clear();
            }
        }
    }

    pub fn recent_entries(&self, limit: usize) -> Vec<ToolAuditEntry> {
        let guard = self.entries.lock();
        let count = guard.len();
        if count >= limit {
            return guard.iter().rev().take(limit).cloned().collect();
        }
        let mut result: Vec<ToolAuditEntry> = guard.iter().rev().take(limit).cloned().collect();
        drop(guard);

        let sink = self.sink.lock().clone();
        if let Some(ref s) = sink {
            let remaining = limit.saturating_sub(result.len());
            if remaining > 0 {
                let mut historical = s.query(None, remaining);
                result.append(&mut historical);
            }
        }
        result
    }

    pub fn entries_for_agent(&self, agent: &str, limit: usize) -> Vec<ToolAuditEntry> {
        let guard = self.entries.lock();
        let in_mem: Vec<ToolAuditEntry> = guard
            .iter()
            .rev()
            .filter(|e| e.agent == agent)
            .take(limit)
            .cloned()
            .collect();
        drop(guard);

        if in_mem.len() >= limit {
            return in_mem;
        }

        let sink = self.sink.lock().clone();
        if let Some(ref s) = sink {
            let remaining = limit.saturating_sub(in_mem.len());
            if remaining > 0 {
                let mut historical = s.query(Some(agent), remaining);
                let mut combined = in_mem;
                combined.append(&mut historical);
                return combined;
            }
        }
        in_mem
    }

    pub fn blocked_entries(&self, limit: usize) -> Vec<ToolAuditEntry> {
        let guard = self.entries.lock();
        guard
            .iter()
            .rev()
            .filter(|e| e.action != SecurityAction::Allow)
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn total_count(&self) -> usize {
        self.entries.lock().len()
    }
}

pub fn compute_params_hash(params: &serde_json::Value) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let serialized = serde_json::to_string(params).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    serialized.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub type SharedAuditLog = Arc<ToolAuditLog>;

pub fn shared_audit_log(max_entries: usize) -> SharedAuditLog {
    Arc::new(ToolAuditLog::new(max_entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_security::SecurityAction;
    use anyhow::Result;

    #[derive(Serialize)]
    struct KeyValueParams {
        key: &'static str,
    }

    #[derive(Serialize)]
    struct AParams {
        a: i64,
    }

    #[test]
    fn audit_log_records_and_retrieves() -> Result<()> {
        let log = ToolAuditLog::new(100);
        log.record(ToolAuditEntry::new(
            "skopeo",
            "exec",
            "abc123",
            SecurityAction::Allow,
            String::new(),
            None,
            None,
        ));
        log.record(ToolAuditEntry::new(
            "hubris",
            "exec",
            "def456",
            SecurityAction::Block,
            "dangerous".to_string(),
            Some("exec_rmrf_root".to_string()),
            None,
        ));
        assert_eq!(log.total_count(), 2);
        let recent = log.recent_entries(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].agent, "hubris");
        Ok(())
    }

    #[test]
    fn audit_log_trims_at_max() -> Result<()> {
        let log = ToolAuditLog::new(3);
        for i in 0..5 {
            log.record(ToolAuditEntry::new(
                "skopeo",
                "exec",
                &format!("hash{}", i),
                SecurityAction::Allow,
                String::new(),
                None,
                None,
            ));
        }
        assert_eq!(log.total_count(), 3);
        let recent = log.recent_entries(10);
        assert_eq!(recent[0].params_hash, "hash4");
        Ok(())
    }

    #[test]
    fn audit_log_filters_by_agent() -> Result<()> {
        let log = ToolAuditLog::new(100);
        log.record(ToolAuditEntry::new(
            "skopeo",
            "exec",
            "a",
            SecurityAction::Allow,
            String::new(),
            None,
            None,
        ));
        log.record(ToolAuditEntry::new(
            "hubris",
            "exec",
            "b",
            SecurityAction::Allow,
            String::new(),
            None,
            None,
        ));
        log.record(ToolAuditEntry::new(
            "skopeo",
            "file_read",
            "c",
            SecurityAction::Allow,
            String::new(),
            None,
            None,
        ));
        let skopeo = log.entries_for_agent("skopeo", 10);
        assert_eq!(skopeo.len(), 2);
        Ok(())
    }

    #[test]
    fn audit_log_filters_blocked() -> Result<()> {
        let log = ToolAuditLog::new(100);
        log.record(ToolAuditEntry::new(
            "skopeo",
            "exec",
            "a",
            SecurityAction::Allow,
            String::new(),
            None,
            None,
        ));
        log.record(ToolAuditEntry::new(
            "hubris",
            "exec",
            "b",
            SecurityAction::Block,
            "dangerous".to_string(),
            None,
            None,
        ));
        log.record(ToolAuditEntry::new(
            "skopeo",
            "exec",
            "c",
            SecurityAction::RateLimited,
            "rate".to_string(),
            None,
            None,
        ));
        let blocked = log.blocked_entries(10);
        assert_eq!(blocked.len(), 2);
        Ok(())
    }

    #[test]
    fn params_hash_deterministic() -> Result<()> {
        let params = serde_json::to_value(KeyValueParams { key: "value" }).unwrap_or_default();
        let h1 = compute_params_hash(&params);
        let h2 = compute_params_hash(&params);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
        Ok(())
    }

    #[test]
    fn params_hash_different_for_different_params() -> Result<()> {
        let h1 = compute_params_hash(&serde_json::to_value(AParams { a: 1 }).unwrap_or_default());
        let h2 = compute_params_hash(&serde_json::to_value(AParams { a: 2 }).unwrap_or_default());
        assert_ne!(h1, h2);
        Ok(())
    }
}
