use serde::Serialize;

use crate::cli::{
    client::{CliApiClient, DebugVerboseResponse},
    config::OperatorContext,
    error::CliError,
    output,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct DebugVerboseView {
    pub(crate) api_url: String,
    pub(crate) since: Option<u64>,
    pub(crate) payload: DebugVerboseResponse,
}

pub(crate) async fn run_debug_verbose(
    context: &OperatorContext,
    json: bool,
    since: Option<u64>,
) -> Result<String, CliError> {
    let client = CliApiClient::from_config(context.api_client.clone())?;
    let payload = client.get_debug_verbose_since(since).await?;
    let view = DebugVerboseView {
        api_url: client.base_url().to_string(),
        since,
        payload,
    };
    output::render_debug_verbose_view(&view, json)
}
