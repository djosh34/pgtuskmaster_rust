use std::fs;

const NEXTTEST_CONFIG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.config/nextest.toml");
const HA_FEATURES_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ha/features");
const PARALLEL_POLICY_COMMENT_START: &str = "If a scenario only passes in serial,";
const PARALLEL_POLICY_COMMENT_END: &str = "must be fixed instead of protected by serialization.";
const GREENFIELD_HA_BINARY_RULE: &str = "binary(ha)";
const GREENFIELD_HA_TEST_RULE: &str = "test(/^ha_/)";
const SETUP_SCRIPTS_EXPERIMENT: &str = "setup-scripts";
const HA_CUCUMBER_SETUP_SCRIPT: &str = "ha-cucumber-image";

#[test]
fn nextest_profiles_route_greenfield_ha_binaries_without_serial_cap() {
    let config_text_result = fs::read_to_string(NEXTTEST_CONFIG_PATH);
    assert!(
        config_text_result.is_ok(),
        "failed to read nextest config {}: {:?}",
        NEXTTEST_CONFIG_PATH,
        config_text_result.as_ref().err()
    );
    let config_text = config_text_result.unwrap_or_default();

    let parsed_result = toml::from_str::<toml::Value>(&config_text);
    assert!(
        parsed_result.is_ok(),
        "failed to parse nextest config {}: {:?}",
        NEXTTEST_CONFIG_PATH,
        parsed_result.as_ref().err()
    );
    let parsed = parsed_result.unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

    let default_filter = profile_filter(&parsed, "default");
    let ultra_long_filter = profile_filter(&parsed, "ultra-long");
    let ultra_long_threads = profile_test_threads(&parsed, "ultra-long");
    let ultra_long_setup_scripts = profile_setup_scripts(&parsed, "ultra-long");
    let experimental_features = experimental_features(&parsed);
    let ha_feature_count = count_ha_feature_files();

    assert!(
        default_filter.contains(GREENFIELD_HA_BINARY_RULE),
        "default profile must exclude long greenfield HA binaries via {}: {default_filter}",
        GREENFIELD_HA_BINARY_RULE
    );
    assert!(
        default_filter.contains(GREENFIELD_HA_TEST_RULE),
        "default profile must exclude only greenfield HA scenario tests via {}: {default_filter}",
        GREENFIELD_HA_TEST_RULE
    );
    assert!(
        ultra_long_filter.contains(GREENFIELD_HA_BINARY_RULE),
        "ultra-long profile must select long greenfield HA binaries via {}: {ultra_long_filter}",
        GREENFIELD_HA_BINARY_RULE
    );
    assert!(
        ultra_long_filter.contains(GREENFIELD_HA_TEST_RULE),
        "ultra-long profile must select only greenfield HA scenario tests via {}: {ultra_long_filter}",
        GREENFIELD_HA_TEST_RULE
    );
    assert!(
        !default_filter.contains("test(="),
        "default profile must not use exact test-name filters: {default_filter}"
    );
    assert!(
        !ultra_long_filter.contains("test(="),
        "ultra-long profile must not use exact test-name filters: {ultra_long_filter}"
    );
    assert!(
        ultra_long_threads.is_some(),
        "ultra-long profile must set test-threads high enough to fan out every HA feature"
    );
    assert!(
        ultra_long_threads
            .and_then(|value| usize::try_from(value).ok())
            .is_some_and(|threads| threads >= ha_feature_count),
        "ultra-long profile must allow at least one worker per HA feature: threads={ultra_long_threads:?} features={ha_feature_count}"
    );
    assert!(
        experimental_features
            .iter()
            .any(|feature| feature == SETUP_SCRIPTS_EXPERIMENT),
        "nextest config must enable the {} experiment",
        SETUP_SCRIPTS_EXPERIMENT
    );
    assert!(
        ultra_long_setup_scripts
            .iter()
            .any(|setup| setup.iter().any(|name| name == HA_CUCUMBER_SETUP_SCRIPT)),
        "ultra-long profile must register the shared HA cucumber setup script {}",
        HA_CUCUMBER_SETUP_SCRIPT
    );
}

#[test]
fn nextest_config_keeps_parallel_policy_comment_visible() {
    let config_text_result = fs::read_to_string(NEXTTEST_CONFIG_PATH);
    assert!(
        config_text_result.is_ok(),
        "failed to read nextest config {}: {:?}",
        NEXTTEST_CONFIG_PATH,
        config_text_result.as_ref().err()
    );
    let config_text = config_text_result.unwrap_or_default();

    assert!(
        config_text.contains(GREENFIELD_HA_BINARY_RULE),
        "nextest config must document the durable greenfield HA binary rule {}",
        GREENFIELD_HA_BINARY_RULE
    );
    assert!(
        config_text.contains(GREENFIELD_HA_TEST_RULE),
        "nextest config must document the durable greenfield HA test-name rule {}",
        GREENFIELD_HA_TEST_RULE
    );
    assert!(
        config_text.contains("[scripts.setup.ha-cucumber-image]"),
        "nextest config must define the shared HA cucumber setup script section"
    );
    assert!(
        config_text.contains(PARALLEL_POLICY_COMMENT_START),
        "nextest config must keep the serial-only policy comment opening"
    );
    assert!(
        config_text.contains(PARALLEL_POLICY_COMMENT_END),
        "nextest config must keep the serial-only policy comment conclusion"
    );
}

fn profile_filter(config: &toml::Value, profile_name: &str) -> String {
    config
        .get("profile")
        .and_then(|profiles| profiles.get(profile_name))
        .and_then(|profile| profile.get("default-filter"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .unwrap_or_default()
}

fn profile_test_threads(config: &toml::Value, profile_name: &str) -> Option<i64> {
    config
        .get("profile")
        .and_then(|profiles| profiles.get(profile_name))
        .and_then(|profile| profile.get("test-threads"))
        .and_then(toml::Value::as_integer)
}

fn count_ha_feature_files() -> usize {
    let read_dir_result = fs::read_dir(HA_FEATURES_ROOT);
    assert!(
        read_dir_result.is_ok(),
        "failed to read HA features root {}: {:?}",
        HA_FEATURES_ROOT,
        read_dir_result.as_ref().err()
    );
    let read_dir = match read_dir_result {
        Ok(value) => value,
        Err(_) => return 0,
    };
    read_dir
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.join(
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| format!("{name}.feature"))
                    .unwrap_or_default(),
            )
            .is_file()
        })
        .count()
}

fn experimental_features(config: &toml::Value) -> Vec<String> {
    config
        .get("experimental")
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn profile_setup_scripts(config: &toml::Value, profile_name: &str) -> Vec<Vec<String>> {
    config
        .get("profile")
        .and_then(|profiles| profiles.get(profile_name))
        .and_then(|profile| profile.get("scripts"))
        .and_then(toml::Value::as_array)
        .map(|scripts| {
            scripts
                .iter()
                .filter_map(|script| script.get("setup"))
                .filter_map(setup_names)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn setup_names(value: &toml::Value) -> Option<Vec<String>> {
    match value {
        toml::Value::String(name) => Some(vec![name.clone()]),
        toml::Value::Array(values) => Some(
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        ),
        _ => None,
    }
}
