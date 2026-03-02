use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiError, ApiResult, AcceptedResponse},
    dcs::{state::SwitchoverRequest, store::DcsStore},
    state::MemberId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    pub(crate) requested_by: MemberId,
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

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{
        api::controller::{post_switchover, SwitchoverRequestInput},
        dcs::{state::SwitchoverRequest, store::{DcsStore, DcsStoreError, WatchEvent}},
        state::MemberId,
    };

    #[derive(Default)]
    struct RecordingStore {
        writes: VecDeque<(String, String)>,
    }

    impl RecordingStore {
        fn pop_write(&mut self) -> Option<(String, String)> {
            self.writes.pop_front()
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

        fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
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

        let (path, raw) = store.pop_write().ok_or_else(|| {
            crate::api::ApiError::internal("expected one DCS write".to_string())
        })?;
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
}
