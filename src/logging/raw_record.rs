use serde::Serialize;

use super::{
    LogError, LogParser, LogProducer, LogRecord, LogSource, LogTransport, SeverityText,
    StructuredFields,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RawRecordBuilder {
    severity: SeverityText,
    message: String,
    source: LogSource,
    fields: StructuredFields,
}

impl RawRecordBuilder {
    pub(crate) fn new(
        severity: SeverityText,
        message: impl Into<String>,
        source: LogSource,
    ) -> Self {
        Self {
            severity,
            message: message.into(),
            source,
            fields: StructuredFields::new(),
        }
    }

    pub(crate) fn with_fields(mut self, fields: StructuredFields) -> Self {
        self.fields = fields;
        self
    }

    pub(crate) fn into_record(self, timestamp_ms: u64, hostname: String) -> LogRecord {
        let mut record = LogRecord::new(
            timestamp_ms,
            hostname,
            self.severity,
            self.message,
            self.source,
        );
        record.attributes = self.fields.into_attributes();
        record
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SubprocessStream {
    Stdout,
    Stderr,
}

impl SubprocessStream {
    fn severity(self) -> SeverityText {
        match self {
            Self::Stdout => SeverityText::Info,
            Self::Stderr => SeverityText::Warn,
        }
    }

    fn transport(self) -> LogTransport {
        match self {
            Self::Stdout => LogTransport::ChildStdout,
            Self::Stderr => LogTransport::ChildStderr,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SubprocessLineRecord {
    producer: LogProducer,
    origin: String,
    job_id: String,
    job_kind: String,
    binary: String,
    stream: SubprocessStream,
    bytes: Vec<u8>,
}

impl SubprocessLineRecord {
    pub(crate) fn new(
        producer: LogProducer,
        origin: impl Into<String>,
        job_id: impl Into<String>,
        job_kind: impl Into<String>,
        binary: impl Into<String>,
        stream: SubprocessStream,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            producer,
            origin: origin.into(),
            job_id: job_id.into(),
            job_kind: job_kind.into(),
            binary: binary.into(),
            stream,
            bytes,
        }
    }

    pub(crate) fn into_raw_record(self) -> Result<RawRecordBuilder, LogError> {
        let (message, raw_bytes_hex) = decode_bytes(self.bytes);
        let source = LogSource {
            producer: self.producer,
            transport: self.stream.transport(),
            parser: LogParser::Raw,
            origin: self.origin,
        };
        let mut fields = StructuredFields::new();
        fields.insert("job_id", self.job_id);
        fields.insert("job_kind", self.job_kind);
        fields.insert("binary", self.binary);
        fields.insert_serialized("stream", &self.stream)?;
        fields.insert_optional("raw_bytes_hex", raw_bytes_hex);
        Ok(RawRecordBuilder::new(self.stream.severity(), message, source).with_fields(fields))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PostgresLineRecordBuilder {
    producer: LogProducer,
    transport: LogTransport,
    origin: String,
}

impl PostgresLineRecordBuilder {
    pub(crate) fn new(
        producer: LogProducer,
        transport: LogTransport,
        origin: impl Into<String>,
    ) -> Self {
        Self {
            producer,
            transport,
            origin: origin.into(),
        }
    }

    pub(crate) fn build(
        self,
        parser: LogParser,
        severity: SeverityText,
        message: impl Into<String>,
        fields: StructuredFields,
    ) -> RawRecordBuilder {
        RawRecordBuilder::new(
            severity,
            message,
            LogSource {
                producer: self.producer,
                transport: self.transport,
                parser,
                origin: self.origin,
            },
        )
        .with_fields(fields)
    }
}

fn decode_bytes(bytes: Vec<u8>) -> (String, Option<String>) {
    match String::from_utf8(bytes) {
        Ok(message) => (message, None),
        Err(err) => {
            let raw_bytes = err.into_bytes();
            let hex = hex_encode(raw_bytes.as_slice());
            (format!("non_utf8_bytes_hex={hex}"), Some(hex))
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        out.push(TABLE[(byte >> 4) as usize] as char);
        out.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    out
}
