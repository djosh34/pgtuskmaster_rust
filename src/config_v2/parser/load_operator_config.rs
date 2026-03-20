use std::{path::Path, path::PathBuf};

use crate::config_v2::types::{
    ApiClientTokens, OperatorClientTlsConfig, OperatorConfigV2, PgtmApiTransportExpectation,
    TlsConfig,
};

use super::{
    load_config::{
        normalize_optional_string, parse_error, read_config_file, resolve_path_only,
        resolve_secret_optional, resolve_secret_path, take_token_sources, token_auth_mode,
        validation_error, validate_non_empty, TokenAuthMode,
    },
    private_schema as raw,
};

pub fn load_operator_config(path: &Path) -> Result<OperatorConfigV2, crate::config_v2::ConfigErrorV2> {
    let contents = read_config_file(path)?;
    let document: raw::OperatorConfigDocument =
        toml::from_str(&contents).map_err(|source| parse_error(path, source))?;

    let operator = match document {
        raw::OperatorConfigDocument::Operator(operator) => *operator,
        raw::OperatorConfigDocument::Runtime(runtime) => runtime.pgtm.ok_or_else(|| {
            validation_error(
                "pgtm",
                "missing operator config block in runtime document",
            )
        })?,
    };

    if let Some(base_url) = operator.api.base_url.as_ref() {
        validate_non_empty("pgtm.api.base_url", base_url)?;
    }
    if operator.api.advertised_url.is_some() {
        return Err(validation_error(
            "pgtm.api.advertised_url",
            "is not supported by config_v2",
        ));
    }

    Ok(OperatorConfigV2 {
        api_base_url: normalize_optional_string(operator.api.base_url),
        expected_transport: map_expected_transport(operator.api.expected_transport),
        api_resolve_to: operator.api.resolve_to,
        client_tls: merge_client_tls(
            map_operator_client_tls("pgtm.api.tls", operator.api.tls)?,
            map_operator_client_tls("pgtm.postgres.tls", operator.postgres.tls)?,
        )?,
        api_auth: map_operator_auth(operator.api.auth)?,
    })
}

fn map_operator_auth(auth: raw::TokenAuthConfig) -> Result<ApiClientTokens, crate::config_v2::ConfigErrorV2> {
    let mode = token_auth_mode(&auth);
    let (read_token, admin_token) = take_token_sources(auth);
    match mode {
        TokenAuthMode::Disabled => Ok(ApiClientTokens::default()),
        TokenAuthMode::RoleTokens => Ok(ApiClientTokens {
            read_token: resolve_secret_optional("pgtm.api.auth.read_token", read_token)?,
            admin_token: resolve_secret_optional("pgtm.api.auth.admin_token", admin_token)?,
        }),
    }
}

fn map_expected_transport(
    expected_transport: Option<raw::PgtmApiTransportExpectation>,
) -> Option<PgtmApiTransportExpectation> {
    expected_transport.map(|expected_transport| match expected_transport {
        raw::PgtmApiTransportExpectation::Http => PgtmApiTransportExpectation::Http,
        raw::PgtmApiTransportExpectation::Https => PgtmApiTransportExpectation::Https,
    })
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
                    key: resolve_secret_path(operator_key_field(field_prefix), identity.key)?,
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
            format!(
                "`{left_field}` and `{right_field}` must match when both are configured"
            ),
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
    use crate::config_v2::PgtmApiTransportExpectation;
    use std::{path::PathBuf, time::SystemTime};

    #[test]
    fn load_operator_config_preserves_expected_transport() -> Result<(), String> {
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

    fn write_temp_config(contents: &str) -> Result<PathBuf, String> {
        let path = std::env::temp_dir().join(format!(
            "pgtm-operator-config-{}-{}.toml",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        std::fs::write(&path, contents).map_err(|err| err.to_string())?;
        Ok(path)
    }
}
