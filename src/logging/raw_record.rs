use std::collections::BTreeMap;

use serde_json::Value;

use super::{
    event::{DomainLogEvent, LogEventMetadata, LogFieldVisitor},
    LogRecord, LogSource,
};

#[derive(Clone, Debug, PartialEq)]
enum QueuedField {
    String { key: &'static str, value: String },
    Bool { key: &'static str, value: bool },
    U64 { key: &'static str, value: u64 },
    Usize { key: &'static str, value: usize },
    Json { key: &'static str, value: Value },
}

#[derive(Default)]
struct QueuedFieldRecorder {
    fields: Vec<QueuedField>,
}

impl LogFieldVisitor for QueuedFieldRecorder {
    fn string(&mut self, key: &'static str, value: String) {
        self.fields.push(QueuedField::String { key, value });
    }

    fn str(&mut self, key: &'static str, value: &'static str) {
        self.fields.push(QueuedField::String {
            key,
            value: value.to_string(),
        });
    }

    fn bool(&mut self, key: &'static str, value: bool) {
        self.fields.push(QueuedField::Bool { key, value });
    }

    fn u64(&mut self, key: &'static str, value: u64) {
        self.fields.push(QueuedField::U64 { key, value });
    }

    fn usize(&mut self, key: &'static str, value: usize) {
        self.fields.push(QueuedField::Usize { key, value });
    }

    fn json(&mut self, key: &'static str, value: Value) {
        self.fields.push(QueuedField::Json { key, value });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct QueuedRecord {
    pub(crate) timestamp_ms: u64,
    pub(crate) hostname: String,
    metadata: LogEventMetadata,
    fields: Vec<QueuedField>,
}

impl QueuedRecord {
    pub(crate) fn from_event<E: DomainLogEvent>(
        timestamp_ms: u64,
        hostname: String,
        event: E,
    ) -> Self {
        let metadata = event.metadata();
        let mut recorder = QueuedFieldRecorder::default();
        event.write_fields(&mut recorder);
        Self {
            timestamp_ms,
            hostname,
            metadata,
            fields: recorder.fields,
        }
    }

    pub(crate) fn into_record(self) -> LogRecord {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            "event.name".to_string(),
            Value::String(self.metadata.event_name.to_string()),
        );
        attributes.insert(
            "event.domain".to_string(),
            Value::String(self.metadata.event_domain.to_string()),
        );
        attributes.insert(
            "event.result".to_string(),
            Value::String(self.metadata.event_result.label().to_string()),
        );

        for field in self.fields {
            match field {
                QueuedField::String { key, value } => {
                    attributes.insert(key.to_string(), Value::String(value));
                }
                QueuedField::Bool { key, value } => {
                    attributes.insert(key.to_string(), Value::Bool(value));
                }
                QueuedField::U64 { key, value } => {
                    attributes.insert(key.to_string(), Value::Number(value.into()));
                }
                QueuedField::Usize { key, value } => {
                    attributes.insert(key.to_string(), Value::Number(value.into()));
                }
                QueuedField::Json { key, value } => {
                    attributes.insert(key.to_string(), value);
                }
            }
        }

        LogRecord {
            timestamp_ms: self.timestamp_ms,
            hostname: self.hostname,
            severity_text: self.metadata.severity,
            severity_number: self.metadata.severity.number(),
            message: self.metadata.message.into_owned(),
            source: LogSource {
                producer: self.metadata.source.producer,
                transport: self.metadata.source.transport,
                parser: self.metadata.source.parser,
                origin: self.metadata.source.origin,
            },
            attributes,
        }
    }
}
