use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::mpsc;

use tracing::{Event, Level, field};
use tracing_subscriber::{Layer, layer::Context};

use crate::pg_log_writer::LogEntry;

const FIELD_MESSAGE: &str = "message";

pub struct PgLogLayer {
    tx: mpsc::UnboundedSender<LogEntry>,
    source: String,
}

impl PgLogLayer {
    pub fn new(tx: mpsc::UnboundedSender<LogEntry>, source: &str) -> Self {
        Self {
            tx,
            source: source.to_string(),
        }
    }
}

impl<S> Layer<S> for PgLogLayer
where
    S: tracing::Subscriber,
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = LogEntryVisitor::new();
        event.record(&mut visitor);

        let metadata = event.metadata();
        let entry = LogEntry {
            source: self.source.clone(),
            instance_uuid: None,
            level: level_str(*metadata.level()),
            target: Some(metadata.target().to_string()),
            message: visitor.message.unwrap_or_default(),
            fields: serde_json::to_value(&visitor.fields)
                .unwrap_or(serde_json::Value::Object(Default::default())),
            created_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        };

        let _ = self.tx.send(entry);
    }
}

fn level_str(level: Level) -> String {
    match level {
        Level::ERROR => "error".to_string(),
        Level::WARN => "warn".to_string(),
        Level::INFO => "info".to_string(),
        Level::DEBUG => "debug".to_string(),
        Level::TRACE => "trace".to_string(),
    }
}

struct LogEntryVisitor {
    message: Option<String>,
    fields: HashMap<String, String>,
}

impl LogEntryVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: HashMap::new(),
        }
    }
}

impl field::Visit for LogEntryVisitor {
    fn record_debug(&mut self, field: &field::Field, value: &dyn std::fmt::Debug) {
        let value_str = format!("{:?}", value);
        if field.name() == FIELD_MESSAGE {
            self.message = Some(value_str);
        } else {
            self.fields.insert(field.name().to_string(), value_str);
        }
    }

    fn record_str(&mut self, field: &field::Field, value: &str) {
        if field.name() == FIELD_MESSAGE {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .insert(field.name().to_string(), value.to_string());
        }
    }

    fn record_error(&mut self, field: &field::Field, value: &(dyn std::error::Error + 'static)) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &field::Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &field::Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_f64(&mut self, field: &field::Field, value: f64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_str() {
        assert_eq!(level_str(Level::ERROR), "error");
        assert_eq!(level_str(Level::WARN), "warn");
        assert_eq!(level_str(Level::INFO), "info");
        assert_eq!(level_str(Level::DEBUG), "debug");
        assert_eq!(level_str(Level::TRACE), "trace");
    }
}
