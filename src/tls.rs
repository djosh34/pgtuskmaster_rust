use std::{fs, io::Cursor, sync::Arc};

use thiserror::Error;

use crate::config::{ApiTlsMode, InlineOrPath, TlsClientAuthConfig, TlsServerConfig};
use rustls::server::danger::ClientCertVerifier;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum TlsConfigError {
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("pem parse error: {message}")]
    PemParse { message: String },
    #[error("rustls error: {message}")]
    Rustls { message: String },
}

pub(crate) fn build_rustls_server_config(
    tls: &TlsServerConfig,
) -> Result<Option<Arc<rustls::ServerConfig>>, TlsConfigError> {
    if matches!(tls.mode, ApiTlsMode::Disabled) {
        return Ok(None);
    }

    let identity = tls
        .identity
        .as_ref()
        .ok_or_else(|| TlsConfigError::InvalidConfig {
            message: "tls.identity must be configured when tls.mode is optional or required"
                .to_string(),
        })?;

    let cert_pem = load_inline_or_path_bytes("tls.identity.cert_chain", &identity.cert_chain)?;
    let key_pem = load_inline_or_path_bytes("tls.identity.private_key", &identity.private_key)?;

    let cert_chain = parse_pem_cert_chain(cert_pem.as_slice())?;
    let key = parse_pem_private_key(key_pem.as_slice())?;

    let provider = rustls::crypto::ring::default_provider();
    let builder = rustls::ServerConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .map_err(|err| TlsConfigError::Rustls {
            message: format!("build server config failed: {err}"),
        })?;

    let config = match tls.client_auth.as_ref() {
        None => builder
            .with_no_client_auth()
            .with_single_cert(cert_chain, key)
            .map_err(|err| TlsConfigError::Rustls {
                message: format!("configure server cert/key failed: {err}"),
            })?,
        Some(client_auth) => {
            let verifier = build_client_verifier(client_auth)?;
            builder
                .with_client_cert_verifier(verifier)
                .with_single_cert(cert_chain, key)
                .map_err(|err| TlsConfigError::Rustls {
                    message: format!("configure server cert/key failed: {err}"),
                })?
        }
    };

    Ok(Some(Arc::new(config)))
}

fn build_client_verifier(
    client_auth: &TlsClientAuthConfig,
) -> Result<Arc<dyn ClientCertVerifier>, TlsConfigError> {
    let ca_pem = load_inline_or_path_bytes("tls.client_auth.client_ca", &client_auth.client_ca)?;
    let ca_certs = parse_pem_cert_chain(ca_pem.as_slice())?;

    let mut roots = rustls::RootCertStore::empty();
    for cert in ca_certs {
        roots.add(cert).map_err(|err| TlsConfigError::Rustls {
            message: format!("add client ca cert failed: {err}"),
        })?;
    }

    let provider = rustls::crypto::ring::default_provider();
    let mut verifier_builder = rustls::server::WebPkiClientVerifier::builder_with_provider(
        Arc::new(roots),
        provider.into(),
    );
    if !client_auth.require_client_cert {
        verifier_builder = verifier_builder.allow_unauthenticated();
    }

    verifier_builder
        .build()
        .map_err(|err| TlsConfigError::Rustls {
            message: format!("build client cert verifier failed: {err}"),
        })
}

fn parse_pem_cert_chain(
    pem: &[u8],
) -> Result<Vec<rustls::pki_types::CertificateDer<'static>>, TlsConfigError> {
    let mut reader = std::io::BufReader::new(Cursor::new(pem));
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| TlsConfigError::PemParse {
            message: format!("parse certs failed: {err}"),
        })?;
    if certs.is_empty() {
        return Err(TlsConfigError::PemParse {
            message: "no certificates found in PEM input".to_string(),
        });
    }
    Ok(certs)
}

fn parse_pem_private_key(
    pem: &[u8],
) -> Result<rustls::pki_types::PrivateKeyDer<'static>, TlsConfigError> {
    let mut reader = std::io::BufReader::new(Cursor::new(pem));
    let key = rustls_pemfile::private_key(&mut reader)
        .map_err(|err| TlsConfigError::PemParse {
            message: format!("parse private key failed: {err}"),
        })?
        .ok_or_else(|| TlsConfigError::PemParse {
            message: "no private key found in PEM input".to_string(),
        })?;
    Ok(key)
}

fn load_inline_or_path_bytes(
    field: &str,
    source: &InlineOrPath,
) -> Result<Vec<u8>, TlsConfigError> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => {
            fs::read(path).map_err(|err| TlsConfigError::Io {
                message: format!("failed to read `{field}` from {}: {err}", path.display()),
            })
        }
        InlineOrPath::Inline { content } => Ok(content.as_bytes().to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        config::{
            ApiTlsMode, InlineOrPath, TlsClientAuthConfig, TlsServerConfig, TlsServerIdentityConfig,
        },
        test_harness::tls::build_adversarial_tls_fixture,
    };

    use super::build_rustls_server_config;

    #[test]
    fn build_rustls_server_config_rejects_optional_without_identity() {
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Optional,
            identity: None,
            client_auth: None,
        };
        let result = build_rustls_server_config(&cfg);
        assert!(result.is_err());
    }

    #[test]
    fn build_rustls_server_config_accepts_inline_identity_and_optional_client_auth(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            }),
            client_auth: Some(TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: fixture.trusted_client_ca.cert.cert_pem.clone(),
                },
                require_client_cert: false,
            }),
        };

        let built = build_rustls_server_config(&cfg)?;
        assert!(built.is_some());
        Ok(())
    }

    #[test]
    fn build_rustls_server_config_reports_io_error_when_cert_path_missing() {
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Path(PathBuf::from(
                    "/tmp/pgtuskmaster-missing-cert-chain.pem",
                )),
                private_key: InlineOrPath::Path(PathBuf::from(
                    "/tmp/pgtuskmaster-missing-private-key.pem",
                )),
            }),
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(result, Err(super::TlsConfigError::Io { .. })));
    }

    #[test]
    fn build_rustls_server_config_reports_pem_error_for_invalid_cert_chain() {
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: "not-a-cert".to_string(),
                },
                private_key: InlineOrPath::Inline {
                    content: "not-a-key".to_string(),
                },
            }),
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(
            result,
            Err(super::TlsConfigError::PemParse { .. })
        ));
    }

    #[test]
    fn build_rustls_server_config_reports_pem_error_for_invalid_private_key(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: "not-a-private-key".to_string(),
                },
            }),
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(
            result,
            Err(super::TlsConfigError::PemParse { .. })
        ));
        Ok(())
    }
}
