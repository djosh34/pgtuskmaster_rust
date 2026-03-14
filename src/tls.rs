use std::{collections::BTreeSet, fs, io::Cursor, sync::Arc};

use axum_server::tls_rustls::RustlsConfig;
use rustls::{
    client::danger::HandshakeSignatureValid,
    self,
    pki_types::{CertificateDer, PrivateKeyDer, UnixTime},
    server::danger::{ClientCertVerified, ClientCertVerifier},
    DigitallySignedStruct, DistinguishedName, Error as RustlsError, SignatureScheme,
};
use thiserror::Error;
use x509_parser::parse_x509_certificate;

use crate::{
    api::worker::{ApiServerTransport, ApiTlsRuntime},
    config::{
        ApiClientAuthConfig, ApiTlsConfig, ApiTransportConfig, ClientCertificateMode,
        ClientCommonName, InlineOrPath, TlsClientAuthConfig,
    },
};
#[cfg(test)]
use crate::config::TlsServerConfig;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum TlsConfigError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("pem parse error: {message}")]
    PemParse { message: String },
    #[error("rustls error: {message}")]
    Rustls { message: String },
}

pub(crate) fn build_api_server_transport(
    transport: &ApiTransportConfig,
) -> Result<ApiServerTransport, TlsConfigError> {
    match transport {
        ApiTransportConfig::Http => Ok(ApiServerTransport::Http),
        ApiTransportConfig::Https { tls } => Ok(ApiServerTransport::Https(ApiTlsRuntime {
            server_config: build_api_rustls_config(tls)?,
        })),
    }
}

pub(crate) fn build_api_server_config(
    tls: &ApiTlsConfig,
) -> Result<Arc<rustls::ServerConfig>, TlsConfigError> {
    let verifier = api_client_verifier(&tls.client_auth)?;
    let mut config = build_server_config(
        "api.security.transport.https.tls.identity",
        &tls.identity.cert_chain,
        &tls.identity.private_key,
        verifier,
    )?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}

#[cfg(test)]
pub(crate) fn build_rustls_server_config(
    tls: &TlsServerConfig,
) -> Result<Option<Arc<rustls::ServerConfig>>, TlsConfigError> {
    match tls {
        TlsServerConfig::Disabled => Ok(None),
        TlsServerConfig::Enabled {
            identity,
            client_auth,
        } => {
            let verifier = plain_client_verifier(client_auth.as_ref())?;
            let config = build_server_config(
                "tls.identity",
                &identity.cert_chain,
                &identity.private_key,
                verifier,
            )?;
            Ok(Some(Arc::new(config)))
        }
    }
}

fn build_api_rustls_config(tls: &ApiTlsConfig) -> Result<RustlsConfig, TlsConfigError> {
    Ok(RustlsConfig::from_config(build_api_server_config(tls)?))
}

fn build_server_config(
    identity_field_prefix: &'static str,
    cert_chain: &InlineOrPath,
    private_key: &InlineOrPath,
    verifier: Arc<dyn ClientCertVerifier>,
) -> Result<rustls::ServerConfig, TlsConfigError> {
    let cert_pem = load_inline_or_path_bytes(
        format!("{identity_field_prefix}.cert_chain").as_str(),
        cert_chain,
    )?;
    let key_pem = load_inline_or_path_bytes(
        format!("{identity_field_prefix}.private_key").as_str(),
        private_key,
    )?;

    let cert_chain = parse_pem_cert_chain(cert_pem.as_slice())?;
    let key = parse_pem_private_key(key_pem.as_slice())?;

    let provider = rustls::crypto::ring::default_provider();
    rustls::ServerConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .map_err(|err| TlsConfigError::Rustls {
            message: format!("build server config failed: {err}"),
        })?
        .with_client_cert_verifier(verifier)
        .with_single_cert(cert_chain, key)
        .map_err(|err| TlsConfigError::Rustls {
            message: format!("configure server cert/key failed: {err}"),
        })
}

#[cfg(test)]
fn plain_client_verifier(
    client_auth: Option<&TlsClientAuthConfig>,
) -> Result<Arc<dyn ClientCertVerifier>, TlsConfigError> {
    match client_auth {
        Some(client_auth) => build_client_verifier(
            client_auth,
            CommonNamePolicy::Disabled,
            "tls.client_auth.client_ca",
        ),
        None => Ok(Arc::new(rustls::server::NoClientAuth)),
    }
}

fn api_client_verifier(
    client_auth: &ApiClientAuthConfig,
) -> Result<Arc<dyn ClientCertVerifier>, TlsConfigError> {
    match client_auth {
        ApiClientAuthConfig::Disabled => Ok(Arc::new(rustls::server::NoClientAuth)),
        ApiClientAuthConfig::Optional { client_ca } => build_client_verifier(
            &TlsClientAuthConfig {
                client_ca: client_ca.clone(),
                client_certificate: ClientCertificateMode::Optional,
            },
            CommonNamePolicy::Disabled,
            "api.security.transport.https.tls.client_auth.client_ca",
        ),
        ApiClientAuthConfig::Required {
            client_ca,
            allowed_common_names,
        } => build_client_verifier(
            &TlsClientAuthConfig {
                client_ca: client_ca.clone(),
                client_certificate: ClientCertificateMode::Required,
            },
            CommonNamePolicy::AllowList(client_common_names(allowed_common_names)),
            "api.security.transport.https.tls.client_auth.client_ca",
        ),
    }
}

fn build_client_verifier(
    client_auth: &TlsClientAuthConfig,
    common_name_policy: CommonNamePolicy,
    field: &'static str,
) -> Result<Arc<dyn ClientCertVerifier>, TlsConfigError> {
    let ca_pem = load_inline_or_path_bytes(field, &client_auth.client_ca)?;
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
    if matches!(
        client_auth.client_certificate,
        ClientCertificateMode::Optional
    ) {
        verifier_builder = verifier_builder.allow_unauthenticated();
    }

    let verifier = verifier_builder
        .build()
        .map_err(|err| TlsConfigError::Rustls {
            message: format!("build client cert verifier failed: {err}"),
        })?;

    match common_name_policy {
        CommonNamePolicy::Disabled => Ok(verifier),
        CommonNamePolicy::AllowList(allowed_common_names) => {
            if allowed_common_names.is_empty() {
                Ok(verifier)
            } else {
                Ok(Arc::new(AllowedCommonNamesClientCertVerifier {
                    inner: verifier,
                    allowed_common_names,
                }))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CommonNamePolicy {
    Disabled,
    AllowList(BTreeSet<String>),
}

#[derive(Debug)]
struct AllowedCommonNamesClientCertVerifier {
    inner: Arc<dyn ClientCertVerifier>,
    allowed_common_names: BTreeSet<String>,
}

impl ClientCertVerifier for AllowedCommonNamesClientCertVerifier {
    fn offer_client_auth(&self) -> bool {
        self.inner.offer_client_auth()
    }

    fn client_auth_mandatory(&self) -> bool {
        self.inner.client_auth_mandatory()
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        self.inner.root_hint_subjects()
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        now: UnixTime,
    ) -> Result<ClientCertVerified, RustlsError> {
        self.inner
            .verify_client_cert(end_entity, intermediates, now)?;

        let common_names = certificate_common_names(end_entity)?;
        let matches_allow_list = common_names
            .iter()
            .any(|value| self.allowed_common_names.contains(value));

        if matches_allow_list {
            Ok(ClientCertVerified::assertion())
        } else {
            Err(RustlsError::General(format!(
                "client certificate common name is not allowed: expected one of {:?}, got {:?}",
                self.allowed_common_names, common_names
            )))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

fn certificate_common_names(end_entity: &CertificateDer<'_>) -> Result<BTreeSet<String>, RustlsError> {
    let (_remaining, certificate) =
        parse_x509_certificate(end_entity.as_ref()).map_err(|err| {
            RustlsError::General(format!(
                "parse client certificate for common-name validation failed: {err}"
            ))
        })?;
    let values = certificate
        .subject()
        .iter_common_name()
        .map(|value| {
            value
                .as_str()
                .map(|common_name| common_name.trim().to_string())
                .map_err(|err| {
                    RustlsError::General(format!(
                        "client certificate common name was not valid UTF-8: {err}"
                    ))
                })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;

    if values.is_empty() || values.iter().any(|value| value.is_empty()) {
        return Err(RustlsError::General(
            "client certificate common name allow-list requires a non-empty common name"
                .to_string(),
        ));
    }

    Ok(values)
}

fn client_common_names(values: &[ClientCommonName]) -> BTreeSet<String> {
    values
        .iter()
        .map(|value| value.0.trim().to_string())
        .collect()
}

fn parse_pem_cert_chain(pem: &[u8]) -> Result<Vec<CertificateDer<'static>>, TlsConfigError> {
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

fn parse_pem_private_key(pem: &[u8]) -> Result<PrivateKeyDer<'static>, TlsConfigError> {
    let mut reader = std::io::BufReader::new(Cursor::new(pem));
    rustls_pemfile::private_key(&mut reader)
        .map_err(|err| TlsConfigError::PemParse {
            message: format!("parse private key failed: {err}"),
        })?
        .ok_or_else(|| TlsConfigError::PemParse {
            message: "no private key found in PEM input".to_string(),
        })
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
    use std::time::Duration;

    use rustls::pki_types::UnixTime;

    use std::path::PathBuf;

    use crate::{
        config::{
            ApiClientAuthConfig, ClientCommonName, InlineOrPath, TlsClientAuthConfig,
            TlsServerConfig, TlsServerIdentityConfig,
        },
        dev_support::tls::build_adversarial_tls_fixture,
    };

    use super::{api_client_verifier, build_rustls_server_config, TlsConfigError};

    fn sample_validation_time() -> UnixTime {
        UnixTime::since_unix_epoch(Duration::from_secs(1_735_689_600))
    }

    #[test]
    fn build_rustls_server_config_accepts_inline_identity_and_optional_client_auth(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let cfg = TlsServerConfig::Enabled {
            identity: TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            },
            client_auth: Some(TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: fixture.trusted_client_ca.cert.cert_pem.clone(),
                },
                client_certificate: crate::config::ClientCertificateMode::Optional,
            }),
        };

        let built = build_rustls_server_config(&cfg)?;
        assert!(built.is_some());
        Ok(())
    }

    #[test]
    fn build_rustls_server_config_reports_io_error_when_cert_path_missing() {
        let cfg = TlsServerConfig::Enabled {
            identity: TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Path(PathBuf::from(
                    "/tmp/pgtuskmaster-missing-cert-chain.pem",
                )),
                private_key: InlineOrPath::Path(PathBuf::from(
                    "/tmp/pgtuskmaster-missing-private-key.pem",
                )),
            },
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(result, Err(TlsConfigError::Io { .. })));
    }

    #[test]
    fn build_rustls_server_config_reports_pem_error_for_invalid_cert_chain() {
        let cfg = TlsServerConfig::Enabled {
            identity: TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: "not-a-cert".to_string(),
                },
                private_key: InlineOrPath::Inline {
                    content: "not-a-key".to_string(),
                },
            },
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(result, Err(TlsConfigError::PemParse { .. })));
    }

    #[test]
    fn build_rustls_server_config_reports_pem_error_for_invalid_private_key(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let cfg = TlsServerConfig::Enabled {
            identity: TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: "not-a-private-key".to_string(),
                },
            },
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(result, Err(TlsConfigError::PemParse { .. })));
        Ok(())
    }

    #[test]
    fn api_client_verifier_rejects_client_signed_by_unconfigured_ca(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let verifier = api_client_verifier(&ApiClientAuthConfig::Required {
            client_ca: InlineOrPath::Inline {
                content: fixture.trusted_client_ca.cert.cert_pem.clone(),
            },
            allowed_common_names: vec![ClientCommonName("trusted-client".to_string())],
        })?;

        let result = verifier.verify_client_cert(
            &fixture.untrusted_client.cert_der(),
            &[],
            sample_validation_time(),
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn api_client_verifier_rejects_client_common_name_outside_allow_list(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let verifier = api_client_verifier(&ApiClientAuthConfig::Required {
            client_ca: InlineOrPath::Inline {
                content: fixture.trusted_client_ca.cert.cert_pem.clone(),
            },
            allowed_common_names: vec![ClientCommonName("ops-admin".to_string())],
        })?;

        let result = verifier.verify_client_cert(
            &fixture.trusted_client.cert_der(),
            &[],
            sample_validation_time(),
        );

        assert!(result.is_err());
        Ok(())
    }
}
