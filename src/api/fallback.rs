use serde::{Deserialize, Serialize};

use crate::{
    api::{AcceptedResponse, ApiError, ApiResult},
    config::RuntimeConfig,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct FallbackClusterView {
    pub(crate) name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FallbackHeartbeatInput {
    pub(crate) source: String,
}

pub(crate) fn get_fallback_cluster(cfg: &RuntimeConfig) -> FallbackClusterView {
    FallbackClusterView {
        name: cfg.cluster.name.clone(),
    }
}

pub(crate) fn post_fallback_heartbeat(
    input: FallbackHeartbeatInput,
) -> ApiResult<AcceptedResponse> {
    if input.source.trim().is_empty() {
        return Err(ApiError::bad_request("source must be non-empty"));
    }
    Ok(AcceptedResponse { accepted: true })
}

#[cfg(test)]
mod tests {

    use crate::{
        api::fallback::{get_fallback_cluster, post_fallback_heartbeat, FallbackHeartbeatInput},
        config::RuntimeConfig,
    };

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    #[test]
    fn heartbeat_denies_unknown_fields() {
        let raw = r#"{"source":"x","extra":1}"#;
        let parsed = serde_json::from_str::<FallbackHeartbeatInput>(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn get_cluster_returns_config_name() {
        let cfg = sample_runtime_config();
        let view = get_fallback_cluster(&cfg);
        assert_eq!(view.name, "cluster-a");
    }

    #[test]
    fn heartbeat_rejects_empty_source() {
        let result = post_fallback_heartbeat(FallbackHeartbeatInput {
            source: "   ".to_string(),
        });
        assert!(matches!(result, Err(crate::api::ApiError::BadRequest(_))));
    }
}
