use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use super::{LogError, LogParser, LogProducer, LogRecord, LogSource, LogTransport, SeverityText};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AppEventHeader {
    pub(crate) name: String,
    pub(crate) domain: String,
    pub(crate) result: String,
}

impl AppEventHeader {
    pub(crate) fn new(
        name: impl Into<String>,
        domain: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            domain: domain.into(),
            result: result.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StructuredFields {
    fields: Vec<(String, StructuredValue)>,
}

impl StructuredFields {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn append_json_map(&mut self, attributes: BTreeMap<String, Value>) {
        self.fields.extend(
            attributes
                .into_iter()
                .map(|(key, value)| (key, StructuredValue::Json(value))),
        );
    }

    pub(crate) fn insert<V>(&mut self, key: impl Into<String>, value: V)
    where
        V: Into<StructuredValue>,
    {
        self.fields.push((key.into(), value.into()));
    }

    pub(crate) fn insert_optional<V>(&mut self, key: impl Into<String>, value: Option<V>)
    where
        V: Into<StructuredValue>,
    {
        if let Some(value) = value {
            self.insert(key, value);
        }
    }

    pub(crate) fn insert_serialized<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        value: &T,
    ) -> Result<(), LogError> {
        let json_value =
            serde_json::to_value(value).map_err(|err| LogError::Json(err.to_string()))?;
        self.fields
            .push((key.into(), StructuredValue::Json(json_value)));
        Ok(())
    }

    pub(crate) fn into_attributes(self) -> BTreeMap<String, Value> {
        let mut attributes = BTreeMap::new();
        for (key, value) in self.fields {
            attributes.insert(key, value.into_json());
        }
        attributes
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StructuredValue {
    Bool(bool),
    I64(i64),
    U64(u64),
    String(String),
    Json(Value),
}

impl StructuredValue {
    fn into_json(self) -> Value {
        match self {
            Self::Bool(value) => Value::Bool(value),
            Self::I64(value) => Value::Number(value.into()),
            Self::U64(value) => Value::Number(value.into()),
            Self::String(value) => Value::String(value),
            Self::Json(value) => value,
        }
    }
}

impl From<bool> for StructuredValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i16> for StructuredValue {
    fn from(value: i16) -> Self {
        Self::I64(i64::from(value))
    }
}

impl From<i32> for StructuredValue {
    fn from(value: i32) -> Self {
        Self::I64(i64::from(value))
    }
}

impl From<i64> for StructuredValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u16> for StructuredValue {
    fn from(value: u16) -> Self {
        Self::U64(u64::from(value))
    }
}

impl From<u32> for StructuredValue {
    fn from(value: u32) -> Self {
        Self::U64(u64::from(value))
    }
}

impl From<u64> for StructuredValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<usize> for StructuredValue {
    fn from(value: usize) -> Self {
        Self::U64(value as u64)
    }
}

impl From<&str> for StructuredValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for StructuredValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Value> for StructuredValue {
    fn from(value: Value) -> Self {
        Self::Json(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AppEvent {
    header: AppEventHeader,
    severity: SeverityText,
    message: String,
    fields: StructuredFields,
}

impl AppEvent {
    pub(crate) fn new(
        severity: SeverityText,
        message: impl Into<String>,
        header: AppEventHeader,
    ) -> Self {
        Self {
            header,
            severity,
            message: message.into(),
            fields: StructuredFields::new(),
        }
    }

    pub(crate) fn severity(&self) -> SeverityText {
        self.severity
    }

    pub(crate) fn fields_mut(&mut self) -> &mut StructuredFields {
        &mut self.fields
    }

    pub(crate) fn into_record(
        self,
        timestamp_ms: u64,
        hostname: String,
        origin: impl Into<String>,
    ) -> LogRecord {
        let source = LogSource {
            producer: LogProducer::App,
            transport: LogTransport::Internal,
            parser: LogParser::App,
            origin: origin.into(),
        };
        let mut record =
            LogRecord::new(timestamp_ms, hostname, self.severity, self.message, source);
        let mut attributes = self.fields.into_attributes();
        attributes.insert("event.name".to_string(), Value::String(self.header.name));
        attributes.insert(
            "event.domain".to_string(),
            Value::String(self.header.domain),
        );
        attributes.insert(
            "event.result".to_string(),
            Value::String(self.header.result),
        );
        record.attributes = attributes;
        record
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DecodedAppEvent {
    pub(crate) header: AppEventHeader,
    pub(crate) severity: SeverityText,
    pub(crate) message: String,
    pub(crate) origin: String,
    pub(crate) fields: BTreeMap<String, Value>,
}

#[cfg(test)]
pub(crate) fn decode_app_event(record: &LogRecord) -> Result<DecodedAppEvent, LogError> {
    let name = decode_required_string_attribute(record, "event.name")?;
    let domain = decode_required_string_attribute(record, "event.domain")?;
    let result = decode_required_string_attribute(record, "event.result")?;
    let mut fields = record.attributes.clone();
    fields.remove("event.name");
    fields.remove("event.domain");
    fields.remove("event.result");

    Ok(DecodedAppEvent {
        header: AppEventHeader::new(name, domain, result),
        severity: record.severity_text,
        message: record.message.clone(),
        origin: record.source.origin.clone(),
        fields,
    })
}

#[cfg(test)]
fn decode_required_string_attribute(record: &LogRecord, key: &str) -> Result<String, LogError> {
    match record.attributes.get(key) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(other) => Err(LogError::Json(format!(
            "attribute `{key}` should be a string, got {other}"
        ))),
        None => Err(LogError::Json(format!(
            "attribute `{key}` is missing from app event record"
        ))),
    }
}
