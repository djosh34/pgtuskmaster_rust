pub fn run_feature_test(feature_name: &str, feature_path: &str) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to build tokio runtime: {err}"))?;
    runtime.block_on(crate::support::run_feature(feature_name, feature_path))
}
