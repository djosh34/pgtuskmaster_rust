use std::collections::BTreeMap;

use serde_json::{Number, Value};

use crate::logging::event::{LogEventDto, LogFieldValue};

use super::runtime::{LogContext, LogRecord};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct QueuedRecord {
    pub(crate) timestamp_ns: i64,
    pub(crate) context: LogContext,
    pub(crate) event: LogEventDto,
}

impl QueuedRecord {
    pub(crate) fn from_event(timestamp_ns: i64, context: LogContext, event: LogEventDto) -> Self {
        Self {
            timestamp_ns,
            context,
            event,
        }
    }

    pub(crate) fn into_record(self) -> LogRecord {
        let attributes = self
            .event
            .fields
            .into_iter()
            .map(|field| {
                let value = match field.value {
                    LogFieldValue::String(value) => Value::String(value),
                    LogFieldValue::Bool(value) => Value::Bool(value),
                    LogFieldValue::I64(value) => Value::Number(Number::from(value)),
                    LogFieldValue::U64(value) => Value::Number(Number::from(value)),
                    LogFieldValue::Json(value) => value,
                };
                (field.key.to_string(), value)
            })
            .collect::<BTreeMap<_, _>>();

        LogRecord {
            timestamp_ns: self.timestamp_ns,
            hostname: self.context.hostname,
            cluster_name: self.context.cluster_name,
            scope: self.context.scope,
            member_id: self.context.member_id,
            severity_text: self.event.severity,
            severity_number: self.event.severity.number(),
            message: self.event.message.into_owned(),
            event_name: self.event.event_name,
            event_result: self.event.result,
            producer: self.event.source.producer,
            transport: self.event.source.transport,
            parser: self.event.source.parser,
            attributes,
        }
    }
}
