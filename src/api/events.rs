use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    api::{AcceptedResponse, ApiError, ApiResult},
    logging::{EventMeta, LogHandle, SeverityText},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WalEventIngestInput {
    pub(crate) provider: String,
    pub(crate) event_kind: String,
    pub(crate) invocation_id: String,
    pub(crate) status_code: i32,
    pub(crate) success: bool,
    pub(crate) started_at_ms: u64,
    pub(crate) duration_ms: u64,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) stdout_truncated: bool,
    pub(crate) stderr_truncated: bool,
    pub(crate) wal_path: Option<String>,
    pub(crate) wal_segment: Option<String>,
    pub(crate) destination_path: Option<String>,
    pub(crate) command_program: String,
    pub(crate) command_args: Vec<String>,
}

impl WalEventIngestInput {
    fn validate(&self) -> ApiResult<()> {
        if self.provider.trim().is_empty() {
            return Err(ApiError::bad_request("provider must be non-empty"));
        }
        if self.event_kind.trim().is_empty() {
            return Err(ApiError::bad_request("event_kind must be non-empty"));
        }
        if self.invocation_id.trim().is_empty() {
            return Err(ApiError::bad_request("invocation_id must be non-empty"));
        }
        if !(0..=255).contains(&self.status_code) {
            return Err(ApiError::bad_request(
                "status_code must be in range 0..=255",
            ));
        }
        if self.command_program.trim().is_empty() {
            return Err(ApiError::bad_request("command_program must be non-empty"));
        }
        if self.duration_ms == 0 {
            return Err(ApiError::bad_request("duration_ms must be greater than zero"));
        }

        let wal_path = self.wal_path.as_ref().map(|v| v.trim()).unwrap_or("");
        let wal_segment = self
            .wal_segment
            .as_ref()
            .map(|v| v.trim())
            .unwrap_or("");
        let destination_path = self
            .destination_path
            .as_ref()
            .map(|v| v.trim())
            .unwrap_or("");

        let has_push = !wal_path.is_empty();
        let has_get = !wal_segment.is_empty() || !destination_path.is_empty();
        if has_push && has_get {
            return Err(ApiError::bad_request(
                "wal_path is mutually exclusive with wal_segment/destination_path",
            ));
        }
        if !has_push && !has_get {
            return Err(ApiError::bad_request(
                "must include wal_path (push) or wal_segment+destination_path (get)",
            ));
        }
        if has_get && (wal_segment.is_empty() || destination_path.is_empty()) {
            return Err(ApiError::bad_request(
                "wal_segment and destination_path must both be present for archive-get",
            ));
        }

        Ok(())
    }
}

pub(crate) fn ingest_wal_event(
    log: &LogHandle,
    peer: std::net::SocketAddr,
    input: WalEventIngestInput,
) -> ApiResult<AcceptedResponse> {
    input.validate()?;

    let result = if input.success { "ok" } else { "error" };
    let mut attrs = BTreeMap::new();
    attrs.insert(
        "provider".to_string(),
        serde_json::Value::String(input.provider),
    );
    attrs.insert(
        "event_kind".to_string(),
        serde_json::Value::String(input.event_kind),
    );
    attrs.insert(
        "invocation_id".to_string(),
        serde_json::Value::String(input.invocation_id),
    );
    attrs.insert(
        "status_code".to_string(),
        serde_json::Value::Number(serde_json::Number::from(input.status_code as i64)),
    );
    attrs.insert(
        "success".to_string(),
        serde_json::Value::Bool(input.success),
    );
    attrs.insert(
        "started_at_ms".to_string(),
        serde_json::Value::Number(serde_json::Number::from(input.started_at_ms)),
    );
    attrs.insert(
        "duration_ms".to_string(),
        serde_json::Value::Number(serde_json::Number::from(input.duration_ms)),
    );
    attrs.insert(
        "stdout".to_string(),
        serde_json::Value::String(input.stdout),
    );
    attrs.insert(
        "stderr".to_string(),
        serde_json::Value::String(input.stderr),
    );
    attrs.insert(
        "stdout_truncated".to_string(),
        serde_json::Value::Bool(input.stdout_truncated),
    );
    attrs.insert(
        "stderr_truncated".to_string(),
        serde_json::Value::Bool(input.stderr_truncated),
    );
    if let Some(value) = input.wal_path {
        attrs.insert("wal_path".to_string(), serde_json::Value::String(value));
    }
    if let Some(value) = input.wal_segment {
        attrs.insert("wal_segment".to_string(), serde_json::Value::String(value));
    }
    if let Some(value) = input.destination_path {
        attrs.insert(
            "destination_path".to_string(),
            serde_json::Value::String(value),
        );
    }
    attrs.insert(
        "command.program".to_string(),
        serde_json::Value::String(input.command_program),
    );
    attrs.insert(
        "command.args".to_string(),
        serde_json::Value::Array(input.command_args.into_iter().map(serde_json::Value::String).collect()),
    );
    attrs.insert(
        "api.peer_addr".to_string(),
        serde_json::Value::String(peer.to_string()),
    );

    log.emit_event(
        SeverityText::Info,
        "wal passthrough invocation",
        "api_worker::events_wal",
        EventMeta::new("backup.wal_passthrough", "backup", result),
        attrs,
    )
    .map_err(|err| ApiError::internal(format!("wal event log emit failed: {err}")))?;

    Ok(AcceptedResponse { accepted: true })
}

