use std::collections::BTreeMap;

use serde_json::Value;

use super::{LogRecord, LogSource, SeverityText};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EncodedRecord {
    pub(crate) severity: SeverityText,
    pub(crate) message: String,
    pub(crate) source: LogSource,
    pub(crate) attributes: BTreeMap<String, Value>,
}

impl EncodedRecord {
    pub(crate) fn into_record(self, timestamp_ms: u64, hostname: String) -> LogRecord {
        let mut record = LogRecord::new(
            timestamp_ms,
            hostname,
            self.severity,
            self.message,
            self.source,
        );
        record.attributes = self.attributes;
        record
    }
}
