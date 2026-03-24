use std::{path::Path, path::PathBuf};

use crate::config_v2::types::{
    OperatorClientTlsConfig, OperatorConfigV2, PgtmApiTransportExpectation, Secret, TlsConfig,
};
use crate::state::ApiRoute;
use reqwest::Url;

use super::{
    load_config::{
        normalize_optional_string, parse_error, read_config_file, resolve_path_only,
        resolve_secret_optional, take_token_sources, token_auth_mode, validate_non_empty,
        validation_error, TokenAuthMode,
    },
    private_schema as raw,
};

pub fn load_operator_config(
    path: &Path,
) -> Result<OperatorConfigV2, crate::config_v2::ConfigErrorV2> {
    let contents = read_config_file(path)?;
    let document: raw::OperatorConfigDocument =
        toml::from_str(&contents).map_err(|source| parse_error(path, source))?;

    let operator = match document {
        raw::OperatorConfigDocument::Operator(operator) => *operator,
        raw::OperatorConfigDocument::Runtime(runtime) => runtime.pgtm.ok_or_else(|| {
            validation_error("pgtm", "missing operator config block in runtime document")
        })?,
    };

    if let Some(base_url) = operator.api.base_url.as_ref() {
        validate_non_empty("pgtm.api.base_url", base_url)?;
    }
    if let Some(advertised_url) = operator.api.advertised_url.as_ref() {
        validate_non_empty("pgtm.api.advertised_url", advertised_url)?;
    }
    let expected_transport =
        operator
            .api
            .expected_transport
            .map(|expected_transport| match expected_transport {
                raw::PgtmApiTransportExpectation::Http => PgtmApiTransportExpectation::Http,
                raw::PgtmApiTransportExpectation::Https => PgtmApiTransportExpectation::Https,
            });
    let (read_token, admin_token) = map_operator_auth(operator.api.auth)?;

    Ok(OperatorConfigV2 {
        base_url: parse_operator_base_url(
            normalize_optional_string(operator.api.base_url),
            expected_transport,
        )?,
        advertised_url: parse_operator_api_route(
            normalize_optional_string(operator.api.advertised_url),
            expected_transport,
        )?,
        expected_transport,
        resolve_to: operator.api.resolve_to,
        client_tls: merge_client_tls(
            map_operator_client_tls("pgtm.api.tls", operator.api.tls)?,
            map_operator_client_tls("pgtm.postgres.tls", operator.postgres.tls)?,
        )?,
        read_token,
        admin_token,
    })
}

fn parse_operator_base_url(
    base_url: Option<String>,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<Option<Url>, crate::config_v2::ConfigErrorV2> {
    base_url
        .map(|base_url| {
            let url = Url::parse(base_url.as_str()).map_err(|err| {
                validation_error("pgtm.api.base_url", format!("must be a valid URL: {err}"))
            })?;
            validate_expected_transport(&url, expected_transport)?;
            Ok(url)
        })
        .transpose()
}

fn parse_operator_api_route(
    advertised_url: Option<String>,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<Option<ApiRoute>, crate::config_v2::ConfigErrorV2> {
    advertised_url
        .map(|advertised_url| {
            let url = Url::parse(advertised_url.as_str()).map_err(|err| {
                validation_error(
                    "pgtm.api.advertised_url",
                    format!("must be a valid URL: {err}"),
                )
            })?;
            validate_expected_transport(&url, expected_transport).map_err(|err| match err {
                crate::config_v2::ConfigErrorV2::Validation { message, .. } => {
                    validation_error("pgtm.api.advertised_url", message)
                }
                other => other,
            })?;
            ApiRoute::from_url(url).map_err(|err| validation_error("pgtm.api.advertised_url", err))
        })
        .transpose()
}

fn validate_expected_transport(
    url: &Url,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<(), crate::config_v2::ConfigErrorV2> {
    let Some(expected_transport) = expected_transport else {
        return Ok(());
    };

    if expected_transport.matches_url(url) {
        return Ok(());
    }

    Err(validation_error(
        "pgtm.api.base_url",
        format!(
            "operator config expects `{}` API transport, but resolved base URL uses `{}`",
            expected_transport.scheme(),
            url.scheme()
        ),
    ))
}

fn map_operator_auth(
    auth: raw::TokenAuthConfig,
) -> Result<(Option<Secret>, Option<Secret>), crate::config_v2::ConfigErrorV2> {
    let mode = token_auth_mode(&auth);
    let (read_token, admin_token) = take_token_sources(auth);
    match mode {
        TokenAuthMode::Disabled => Ok((None, None)),
        TokenAuthMode::RoleTokens => Ok((
            resolve_secret_optional("pgtm.api.auth.read_token", read_token)?,
            resolve_secret_optional("pgtm.api.auth.admin_token", admin_token)?,
        )),
    }
}

fn map_operator_client_tls(
    field_prefix: &'static str,
    tls: raw::OperatorClientTlsConfig,
) -> Result<OperatorClientTlsConfig, crate::config_v2::ConfigErrorV2> {
    Ok(OperatorClientTlsConfig {
        ca_cert: tls
            .ca_cert
            .map(|ca_cert| resolve_path_only(operator_ca_field(field_prefix), ca_cert))
            .transpose()?,
        identity: tls
            .identity
            .map(|identity| {
                Ok(TlsConfig {
                    cert: resolve_path_only(operator_cert_field(field_prefix), identity.cert)?,
                    key: resolve_path_only(operator_key_field(field_prefix), identity.key)?,
                    ca_cert: None,
                })
            })
            .transpose()?,
    })
}

fn merge_client_tls(
    api_tls: OperatorClientTlsConfig,
    postgres_tls: OperatorClientTlsConfig,
) -> Result<OperatorClientTlsConfig, crate::config_v2::ConfigErrorV2> {
    Ok(OperatorClientTlsConfig {
        ca_cert: merge_optional_path(
            "pgtm.api.tls.ca_cert",
            api_tls.ca_cert,
            "pgtm.postgres.tls.ca_cert",
            postgres_tls.ca_cert,
            "pgtm.client_tls.ca_cert",
        )?,
        identity: merge_optional_identity(api_tls.identity, postgres_tls.identity)?,
    })
}

fn merge_optional_path(
    left_field: &'static str,
    left: Option<PathBuf>,
    right_field: &'static str,
    right: Option<PathBuf>,
    merged_field: &'static str,
) -> Result<Option<PathBuf>, crate::config_v2::ConfigErrorV2> {
    match (left, right) {
        (Some(left), Some(right)) if left != right => Err(validation_error(
            merged_field,
            format!("`{left_field}` and `{right_field}` must match when both are configured"),
        )),
        (Some(path), Some(_)) | (Some(path), None) | (None, Some(path)) => Ok(Some(path)),
        (None, None) => Ok(None),
    }
}

fn merge_optional_identity(
    left: Option<TlsConfig>,
    right: Option<TlsConfig>,
) -> Result<Option<TlsConfig>, crate::config_v2::ConfigErrorV2> {
    match (left, right) {
        (Some(left), Some(right))
            if left.cert != right.cert || left.key != right.key || left.ca_cert != right.ca_cert =>
        {
            Err(validation_error(
                "pgtm.client_tls.identity",
                "`pgtm.api.tls.identity` and `pgtm.postgres.tls.identity` must match when both are configured",
            ))
        }
        (Some(identity), Some(_)) | (Some(identity), None) | (None, Some(identity)) => {
            Ok(Some(identity))
        }
        (None, None) => Ok(None),
    }
}

fn operator_ca_field(field_prefix: &'static str) -> &'static str {
    match field_prefix {
        "pgtm.api.tls" => "pgtm.api.tls.ca_cert",
        "pgtm.postgres.tls" => "pgtm.postgres.tls.ca_cert",
        _ => field_prefix,
    }
}

fn operator_cert_field(field_prefix: &'static str) -> &'static str {
    match field_prefix {
        "pgtm.api.tls" => "pgtm.api.tls.identity.cert",
        "pgtm.postgres.tls" => "pgtm.postgres.tls.identity.cert",
        _ => field_prefix,
    }
}

fn operator_key_field(field_prefix: &'static str) -> &'static str {
    match field_prefix {
        "pgtm.api.tls" => "pgtm.api.tls.identity.key",
        "pgtm.postgres.tls" => "pgtm.postgres.tls.identity.key",
        _ => field_prefix,
    }
}

#[cfg(test)]
mod tests {
    use super::load_operator_config;
    use crate::config_v2::{ConfigErrorV2, PgtmApiTransportExpectation};
    use std::{net::SocketAddr, path::PathBuf, time::SystemTime};

    #[test]
    fn load_operator_config_preserves_expected_transport_for_operator_documents(
    ) -> Result<(), String> {
        let path = write_temp_config(
            r#"
[api]
base_url = "https://127.0.0.1:8443"
expected_transport = "https"
"#,
        )?;

        let config = load_operator_config(path.as_path()).map_err(|err| err.to_string())?;

        let _ = std::fs::remove_file(path);

        if config.expected_transport != Some(PgtmApiTransportExpectation::Https) {
            return Err(format!(
                "expected https transport expectation, got {:?}",
                config.expected_transport
            ));
        }

        Ok(())
    }

    #[test]
    fn load_operator_config_preserves_expected_transport_for_runtime_documents(
    ) -> Result<(), String> {
        let path = write_temp_config(
            r#"
[cluster]
name = "cluster-a"
scope = "scope-a"
member_id = "node-a"

[postgres.paths]
data_dir = "/tmp/data"

[postgres.roles.mandatory.superuser]
username = "postgres"
auth = { type = "password", password = { type = "string", value = "postgres" } }

[postgres.roles.mandatory.replicator]
username = "replicator"
auth = { type = "password", password = { type = "string", value = "replicator" } }

[postgres.roles.mandatory.rewinder]
username = "rewinder"
auth = { type = "password", password = { type = "string", value = "rewinder" } }

[postgres.access]
hba = { content = "host all all 127.0.0.1/32 trust" }
ident = { content = "" }

[dcs]
endpoints = ["http://127.0.0.1:2379"]

[pgtm.api]
base_url = "https://127.0.0.1:8443"
expected_transport = "https"
"#,
        )?;

        let config = load_operator_config(path.as_path()).map_err(|err| err.to_string())?;

        let _ = std::fs::remove_file(path);

        if config.expected_transport != Some(PgtmApiTransportExpectation::Https) {
            return Err(format!(
                "expected https transport expectation from runtime document, got {:?}",
                config.expected_transport
            ));
        }

        Ok(())
    }

    #[test]
    fn load_operator_config_keeps_resolve_to_on_validated_api_endpoint() -> Result<(), String> {
        let path = write_temp_config(
            r#"
[api]
base_url = "https://node-b:8443"
expected_transport = "https"
resolve_to = "127.0.0.1:18443"
"#,
        )?;

        let config = load_operator_config(path.as_path()).map_err(|err| err.to_string())?;

        let _ = std::fs::remove_file(path);

        if config.base_url.as_ref().map(reqwest::Url::as_str) != Some("https://node-b:8443/") {
            return Err(format!("unexpected api base url: {:?}", config.base_url));
        }
        if config.expected_transport != Some(PgtmApiTransportExpectation::Https) {
            return Err(format!(
                "unexpected transport expectation: {:?}",
                config.expected_transport
            ));
        }
        if config.resolve_to != Some(SocketAddr::from(([127, 0, 0, 1], 18443))) {
            return Err(format!("unexpected resolve_to: {:?}", config.resolve_to));
        }

        Ok(())
    }

    #[test]
    fn load_operator_config_preserves_advertised_url() -> Result<(), String> {
        let path = write_temp_config(
            r#"
[api]
base_url = "https://node-a:8443"
advertised_url = "https://127.0.0.1:18081"
expected_transport = "https"
"#,
        )?;

        let config = load_operator_config(path.as_path()).map_err(|err| err.to_string())?;

        let _ = std::fs::remove_file(path);

        if config
            .advertised_url
            .as_ref()
            .map(crate::state::ApiRoute::as_str)
            != Some("https://127.0.0.1:18081/")
        {
            return Err(format!(
                "unexpected advertised API route: {:?}",
                config.advertised_url
            ));
        }

        Ok(())
    }

    #[test]
    fn load_operator_config_flattens_tokens_and_merges_client_tls() -> Result<(), String> {
        let dir = temp_test_dir("pgtm-operator-config-merge")?;
        let api_ca_path = dir.join("api-ca.pem");
        let identity_cert_path = dir.join("client.crt");
        let identity_key_path = dir.join("client.key");
        let path = write_temp_config(format!(
            r#"
[api]
base_url = "https://127.0.0.1:8443"

[api.auth]
type = "role_tokens"
read_token = {{ type = "string", value = "read-token" }}
admin_token = {{ type = "string", value = "admin-token" }}

[api.tls]
ca_cert = {{ path = "{}" }}
identity = {{ cert = {{ path = "{}" }}, key = {{ path = "{}" }} }}

[postgres.tls]
ca_cert = {{ path = "{}" }}
identity = {{ cert = {{ path = "{}" }}, key = {{ path = "{}" }} }}
"#,
            api_ca_path.display(),
            identity_cert_path.display(),
            identity_key_path.display(),
            api_ca_path.display(),
            identity_cert_path.display(),
            identity_key_path.display(),
        ))?;

        let config = load_operator_config(path.as_path()).map_err(|err| err.to_string())?;

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir_all(dir);

        if config.read_token.as_ref().map(|token| token.as_str()) != Some("read-token") {
            return Err(format!("unexpected read token: {:?}", config.read_token));
        }
        if config.admin_token.as_ref().map(|token| token.as_str()) != Some("admin-token") {
            return Err(format!("unexpected admin token: {:?}", config.admin_token));
        }
        if config.client_tls.ca_cert.as_ref() != Some(&api_ca_path) {
            return Err(format!(
                "unexpected merged CA cert: {:?}",
                config.client_tls.ca_cert
            ));
        }
        if config.client_tls.identity.as_ref().map(|tls| &tls.cert) != Some(&identity_cert_path) {
            return Err(format!(
                "unexpected merged client cert: {:?}",
                config.client_tls.identity
            ));
        }
        if config.client_tls.identity.as_ref().map(|tls| &tls.key) != Some(&identity_key_path) {
            return Err(format!(
                "unexpected merged client key: {:?}",
                config.client_tls.identity
            ));
        }

        Ok(())
    }

    #[test]
    fn load_operator_config_rejects_non_path_tls_identity_sources_at_parse_boundary(
    ) -> Result<(), String> {
        let path = write_temp_config(
            r#"
[api]
base_url = "https://127.0.0.1:8443"

[api.tls]
identity = { cert = { path = "/tmp/client.crt" }, key = { type = "env", env = "CLIENT_KEY" } }
"#,
        )?;

        let result = load_operator_config(path.as_path());
        let _ = std::fs::remove_file(path);

        match result {
            Err(ConfigErrorV2::Parse { .. }) => Ok(()),
            Err(err) => Err(format!("expected parse error, got {err}")),
            Ok(_) => Err("expected non-path TLS identity rejection".to_string()),
        }
    }

    fn write_temp_config(contents: impl AsRef<str>) -> Result<PathBuf, String> {
        let path = std::env::temp_dir().join(format!(
            "pgtm-operator-config-{}-{}.toml",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        std::fs::write(&path, contents.as_ref()).map_err(|err| err.to_string())?;
        Ok(path)
    }

    fn temp_test_dir(prefix: &str) -> Result<PathBuf, String> {
        let dir = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        Ok(dir)
    }
}
