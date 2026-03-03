use serde::{Deserialize, Serialize};

use crate::{
    api::{AcceptedResponse, ApiError, ApiResult, HaStateResponse},
    dcs::{
        state::SwitchoverRequest,
        store::{DcsHaWriter, DcsStore},
    },
    debug_api::snapshot::SystemSnapshot,
    state::{MemberId, Versioned},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    pub(crate) requested_by: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SetLeaderRequestInput {
    pub(crate) member_id: MemberId,
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

pub(crate) fn post_set_leader(
    scope: &str,
    store: &mut dyn DcsStore,
    input: SetLeaderRequestInput,
) -> ApiResult<AcceptedResponse> {
    if input.member_id.0.trim().is_empty() {
        return Err(ApiError::bad_request("member_id must be non-empty"));
    }

    DcsHaWriter::write_leader_lease(store, scope, &input.member_id)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;

    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn delete_leader(scope: &str, store: &mut dyn DcsStore) -> ApiResult<AcceptedResponse> {
    DcsHaWriter::delete_leader(store, scope).map_err(|err| ApiError::DcsStore(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn delete_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
) -> ApiResult<AcceptedResponse> {
    DcsHaWriter::clear_switchover(store, scope)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
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

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{
        api::controller::{
            delete_leader, delete_switchover, post_set_leader, post_switchover,
            SetLeaderRequestInput, SwitchoverRequestInput,
        },
        dcs::{
            state::{LeaderRecord, SwitchoverRequest},
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

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            self.writes.push_back((path.to_string(), value));
            Ok(())
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
    fn set_leader_input_denies_unknown_fields() {
        let raw = r#"{"member_id":"node-a","extra":1}"#;
        let parsed = serde_json::from_str::<SetLeaderRequestInput>(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn post_set_leader_writes_typed_record_to_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let response = post_set_leader(
            "scope-a",
            &mut store,
            SetLeaderRequestInput {
                member_id: MemberId("node-a".to_string()),
            },
        )?;
        assert!(response.accepted);

        let (path, raw) = store
            .pop_write()
            .ok_or_else(|| crate::api::ApiError::internal("expected one DCS write".to_string()))?;
        assert_eq!(path, "/scope-a/leader");
        let decoded = serde_json::from_str::<LeaderRecord>(&raw)
            .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        assert_eq!(decoded.member_id, MemberId("node-a".to_string()));
        Ok(())
    }

    #[test]
    fn post_set_leader_rejects_empty_member_id() {
        let mut store = RecordingStore::default();
        let result = post_set_leader(
            "scope-a",
            &mut store,
            SetLeaderRequestInput {
                member_id: MemberId("".to_string()),
            },
        );
        assert!(matches!(result, Err(crate::api::ApiError::BadRequest(_))));
    }

    #[test]
    fn delete_leader_deletes_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let response = delete_leader("scope-a", &mut store)?;
        assert!(response.accepted);
        assert_eq!(store.pop_delete().as_deref(), Some("/scope-a/leader"));
        Ok(())
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
