use std::fs;

const NEXTTEST_CONFIG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.config/nextest.toml");
const PARALLEL_POLICY_COMMENT_START: &str = "If a scenario only passes in serial,";
const PARALLEL_POLICY_COMMENT_END: &str =
    "must be fixed instead of protected by serialization.";
const HA_BINARY_LAYOUT_RULE: &str = "binary(ha_*)";

#[test]
fn nextest_profiles_use_ha_binary_split_without_serial_cap() {
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

    assert!(
        default_filter.contains(HA_BINARY_LAYOUT_RULE),
        "default profile must exclude long HA binaries via {}: {default_filter}",
        HA_BINARY_LAYOUT_RULE
    );
    assert!(
        ultra_long_filter.contains(HA_BINARY_LAYOUT_RULE),
        "ultra-long profile must select long HA binaries via {}: {ultra_long_filter}",
        HA_BINARY_LAYOUT_RULE
    );
    assert!(
        !default_filter.contains("test(="),
        "default profile must not use exact test-name filters: {default_filter}"
    );
    assert!(
        !ultra_long_filter.contains("test(="),
        "ultra-long profile must not use exact test-name filters: {ultra_long_filter}"
    );
    assert_ne!(
        ultra_long_threads,
        Some(1),
        "ultra-long profile must not reintroduce suite-wide serial execution"
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
        config_text.contains(HA_BINARY_LAYOUT_RULE),
        "nextest config must document the durable HA binary layout rule {}",
        HA_BINARY_LAYOUT_RULE
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
