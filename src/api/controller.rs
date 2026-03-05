use serde::{Deserialize, Serialize};

use crate::{
    api::{AcceptedResponse, ApiError, ApiResult, HaStateResponse},
    dcs::{
        state::{RestorePhase, RestoreRequestRecord, RestoreStatusRecord, SwitchoverRequest},
        store::{DcsHaWriter, DcsStore},
    },
    debug_api::snapshot::SystemSnapshot,
    state::{MemberId, UnixMillis, Versioned},
};

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    pub(crate) requested_by: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClusterRestoreRequestInput {
    pub(crate) requested_by: MemberId,
    pub(crate) executor_member_id: MemberId,
    pub(crate) reason: Option<String>,
    pub(crate) idempotency_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClusterRestoreAcceptedResponse {
    pub(crate) accepted: bool,
    pub(crate) restore_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClusterRestoreDerivedView {
    pub(crate) is_executor: bool,
    pub(crate) heartbeat_stale: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClusterRestoreStatusResponse {
    pub(crate) request: Option<RestoreRequestRecord>,
    pub(crate) status: Option<RestoreStatusRecord>,
    pub(crate) derived: ClusterRestoreDerivedView,
}

pub(crate) fn post_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
    input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    if input.requested_by.0.trim().is_empty() {
        return Err(ApiError::bad_request("requested_by must be non-empty"));
    }

    let request = SwitchoverRequest {
        requested_by: input.requested_by,
    };
    let encoded = serde_json::to_string(&request)
        .map_err(|err| ApiError::internal(format!("switchover encode failed: {err}")))?;

    let path = format!("/{}/switchover", scope.trim_matches('/'));
    store
        .write_path(&path, encoded)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;

    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn post_restore(
    scope: &str,
    store: &mut dyn DcsStore,
    input: ClusterRestoreRequestInput,
) -> ApiResult<ClusterRestoreAcceptedResponse> {
    if input.requested_by.0.trim().is_empty() {
        return Err(ApiError::bad_request("requested_by must be non-empty"));
    }
    if input.executor_member_id.0.trim().is_empty() {
        return Err(ApiError::bad_request("executor_member_id must be non-empty"));
    }
    if let Some(reason) = &input.reason {
        if reason.trim().is_empty() {
            return Err(ApiError::bad_request("reason must be non-empty when provided"));
        }
    }
    if let Some(token) = &input.idempotency_token {
        if token.trim().is_empty() {
            return Err(ApiError::bad_request(
                "idempotency_token must be non-empty when provided",
            ));
        }
    }

    let now = now_unix_millis().map_err(ApiError::internal)?;
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let restore_id = format!("restore-{}-{}-{}", now.0, std::process::id(), counter);

    let request = RestoreRequestRecord {
        restore_id: restore_id.clone(),
        requested_by: input.requested_by,
        requested_at_ms: now,
        executor_member_id: input.executor_member_id.clone(),
        reason: input.reason,
        idempotency_token: input.idempotency_token,
    };
    let encoded_request = serde_json::to_string(&request)
        .map_err(|err| ApiError::internal(format!("restore request encode failed: {err}")))?;

    let status = RestoreStatusRecord {
        restore_id: restore_id.clone(),
        phase: RestorePhase::Requested,
        heartbeat_at_ms: now,
        running_job_id: None,
        last_error: None,
        updated_at_ms: now,
    };
    let encoded_status = serde_json::to_string(&status)
        .map_err(|err| ApiError::internal(format!("restore status encode failed: {err}")))?;

    let request_path = restore_request_path(scope);
    let status_path = restore_status_path(scope);

    let created = store
        .put_path_if_absent(request_path.as_str(), encoded_request)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;
    if !created {
        return Err(ApiError::conflict("restore request already exists"));
    }

    store
        .write_path(status_path.as_str(), encoded_status)
        .map_err(|err| {
            let _ = store.delete_path(request_path.as_str());
            ApiError::DcsStore(err.to_string())
        })?;

    Ok(ClusterRestoreAcceptedResponse {
        accepted: true,
        restore_id,
    })
}

pub(crate) fn delete_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
) -> ApiResult<AcceptedResponse> {
    DcsHaWriter::clear_switchover(store, scope)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn delete_restore(scope: &str, store: &mut dyn DcsStore) -> ApiResult<AcceptedResponse> {
    let request_path = restore_request_path(scope);
    let status_path = restore_status_path(scope);

    store
        .delete_path(request_path.as_str())
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;
    store
        .delete_path(status_path.as_str())
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;

    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn get_restore_status(
    scope: &str,
    store: &mut dyn DcsStore,
    self_member_id: &MemberId,
) -> ApiResult<ClusterRestoreStatusResponse> {
    let request_path = restore_request_path(scope);
    let status_path = restore_status_path(scope);

    let request = store
        .read_path(request_path.as_str())
        .map_err(|err| ApiError::DcsStore(err.to_string()))?
        .map(|raw| {
            serde_json::from_str::<RestoreRequestRecord>(raw.as_str()).map_err(|err| {
                ApiError::internal(format!("decode restore request failed: {err}"))
            })
        })
        .transpose()?;

    let status = store
        .read_path(status_path.as_str())
        .map_err(|err| ApiError::DcsStore(err.to_string()))?
        .map(|raw| {
            serde_json::from_str::<RestoreStatusRecord>(raw.as_str())
                .map_err(|err| ApiError::internal(format!("decode restore status failed: {err}")))
        })
        .transpose()?;

    let is_executor = request
        .as_ref()
        .map(|req| req.executor_member_id == *self_member_id)
        .unwrap_or(false);

    let heartbeat_stale = match (&status, now_unix_millis()) {
        (Some(status), Ok(now)) => now.0.saturating_sub(status.heartbeat_at_ms.0) > 30_000,
        _ => false,
    };

    Ok(ClusterRestoreStatusResponse {
        request,
        status,
        derived: ClusterRestoreDerivedView {
            is_executor,
            heartbeat_stale,
        },
    })
}

pub(crate) fn get_ha_state(snapshot: &Versioned<SystemSnapshot>) -> HaStateResponse {
    HaStateResponse {
        cluster_name: snapshot.value.config.value.cluster.name.clone(),
        scope: snapshot.value.config.value.dcs.scope.clone(),
        self_member_id: snapshot.value.config.value.cluster.member_id.clone(),
        leader: snapshot
            .value
            .dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone()),
        switchover_requested_by: snapshot
            .value
            .dcs
            .value
            .cache
            .switchover
            .as_ref()
            .map(|switchover| switchover.requested_by.0.clone()),
        member_count: snapshot.value.dcs.value.cache.members.len(),
        dcs_trust: format!("{:?}", snapshot.value.dcs.value.trust),
        ha_phase: format!("{:?}", snapshot.value.ha.value.phase),
        ha_tick: snapshot.value.ha.value.tick,
        pending_actions: snapshot.value.ha.value.pending.len(),
        snapshot_sequence: snapshot.value.sequence,
    }
}

fn restore_request_path(scope: &str) -> String {
    format!("/{}/restore/request", scope.trim_matches('/'))
}

fn restore_status_path(scope: &str) -> String {
    format!("/{}/restore/status", scope.trim_matches('/'))
}

fn now_unix_millis() -> Result<UnixMillis, String> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system clock before unix epoch: {err}"))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| format!("unix millis conversion failed: {err}"))?;
    Ok(UnixMillis(millis))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{
        api::controller::{delete_switchover, post_switchover, SwitchoverRequestInput},
        dcs::{
            state::SwitchoverRequest,
            store::{DcsStore, DcsStoreError, WatchEvent},
        },
        state::MemberId,
    };

    #[derive(Default)]
    struct RecordingStore {
        writes: VecDeque<(String, String)>,
        deletes: VecDeque<String>,
    }

    impl RecordingStore {
        fn pop_write(&mut self) -> Option<(String, String)> {
            self.writes.pop_front()
        }

        fn pop_delete(&mut self) -> Option<String> {
            self.deletes.pop_front()
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            self.writes.push_back((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            self.writes.push_back((path.to_string(), value));
            Ok(true)
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            self.deletes.push_back(path.to_string());
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn switchover_input_denies_unknown_fields() {
        let raw = r#"{"requested_by":"node-a","extra":1}"#;
        let parsed = serde_json::from_str::<SwitchoverRequestInput>(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn post_switchover_writes_typed_record_to_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let response = post_switchover(
            "scope-a",
            &mut store,
            SwitchoverRequestInput {
                requested_by: MemberId("node-a".to_string()),
            },
        )?;
        assert!(response.accepted);

        let (path, raw) = store
            .pop_write()
            .ok_or_else(|| crate::api::ApiError::internal("expected one DCS write".to_string()))?;
        assert_eq!(path, "/scope-a/switchover");
        let decoded = serde_json::from_str::<SwitchoverRequest>(&raw)
            .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        assert_eq!(decoded.requested_by, MemberId("node-a".to_string()));
        Ok(())
    }

    #[test]
    fn post_switchover_rejects_empty_requested_by() {
        let mut store = RecordingStore::default();
        let result = post_switchover(
            "scope-a",
            &mut store,
            SwitchoverRequestInput {
                requested_by: MemberId("".to_string()),
            },
        );
        assert!(matches!(result, Err(crate::api::ApiError::BadRequest(_))));
    }

    #[test]
    fn delete_switchover_deletes_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let response = delete_switchover("scope-a", &mut store)?;
        assert!(response.accepted);
        assert_eq!(store.pop_delete().as_deref(), Some("/scope-a/switchover"));
        Ok(())
    }
}
