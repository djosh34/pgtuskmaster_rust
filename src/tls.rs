use std::{collections::BTreeSet, io::Cursor, sync::Arc};

use rustls::{
    self,
    client::danger::HandshakeSignatureValid,
    pki_types::{CertificateDer, PrivateKeyDer, UnixTime},
    server::danger::{ClientCertVerified, ClientCertVerifier},
    DigitallySignedStruct, DistinguishedName, Error as RustlsError, SignatureScheme,
};
use thiserror::Error;
use x509_parser::parse_x509_certificate;

use crate::config_v2::types::ApiTransport as ApiTransportV2;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum TlsConfigError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("pem parse error: {message}")]
    PemParse { message: String },
    #[error("rustls error: {message}")]
    Rustls { message: String },
}

pub(crate) fn build_api_server_config_v2(
    transport: &ApiTransportV2,
) -> Result<Arc<rustls::ServerConfig>, TlsConfigError> {
    let ApiTransportV2::Https {
        tls,
        client_ca,
        client_cert_required,
        allowed_client_common_names,
    } = transport
    else {
        return Err(TlsConfigError::Rustls {
            message: "https transport required for rustls server config".to_string(),
        });
    };
    let verifier = build_client_verifier_from_paths(
        client_ca.as_deref(),
        *client_cert_required,
        allowed_client_common_names,
    )?;
    let mut config = build_server_config_from_paths(&tls.cert, &tls.key, verifier)?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}

fn build_server_config_from_paths(
    cert_path: &std::path::Path,
    private_key_path: &std::path::Path,
    verifier: Arc<dyn ClientCertVerifier>,
) -> Result<rustls::ServerConfig, TlsConfigError> {
    let cert_chain = load_pem_cert_chain_from_path(cert_path)?;
    let key = load_pem_private_key_from_path(private_key_path)?;

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

fn build_client_verifier_from_paths(
    client_ca: Option<&std::path::Path>,
    client_cert_required: bool,
    allowed_client_common_names: &[String],
) -> Result<Arc<dyn ClientCertVerifier>, TlsConfigError> {
    let Some(client_ca) = client_ca else {
        return Ok(Arc::new(rustls::server::NoClientAuth));
    };
    let ca_certs = load_pem_cert_chain_from_path(client_ca)?;

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
    if !client_cert_required {
        verifier_builder = verifier_builder.allow_unauthenticated();
    }

    let verifier = verifier_builder
        .build()
        .map_err(|err| TlsConfigError::Rustls {
            message: format!("build client cert verifier failed: {err}"),
        })?;

    if allowed_client_common_names.is_empty() {
        Ok(verifier)
    } else {
        Ok(Arc::new(AllowedCommonNamesClientCertVerifier {
            inner: verifier,
            allowed_common_names: allowed_client_common_names.iter().cloned().collect(),
        }))
    }
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

fn certificate_common_names(
    end_entity: &CertificateDer<'_>,
) -> Result<BTreeSet<String>, RustlsError> {
    let (_remaining, certificate) = parse_x509_certificate(end_entity.as_ref()).map_err(|err| {
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

pub fn load_pem_cert_chain(path: &std::path::Path) -> Result<Vec<CertificateDer<'static>>, String> {
    load_pem_cert_chain_from_path(path).map_err(|err| err.to_string())
}

pub fn load_pem_private_key(path: &std::path::Path) -> Result<PrivateKeyDer<'static>, String> {
    load_pem_private_key_from_path(path).map_err(|err| err.to_string())
}

fn load_pem_cert_chain_from_path(
    path: &std::path::Path,
) -> Result<Vec<CertificateDer<'static>>, TlsConfigError> {
    std::fs::read(path)
        .map_err(|err| TlsConfigError::Io {
            message: format!("read {} failed: {err}", path.display()),
        })
        .and_then(|pem| {
            parse_pem_cert_chain(pem.as_slice()).map_err(|err| TlsConfigError::PemParse {
                message: format!("parse certificate `{}` failed: {}", path.display(), err),
            })
        })
}

fn load_pem_private_key_from_path(
    path: &std::path::Path,
) -> Result<PrivateKeyDer<'static>, TlsConfigError> {
    std::fs::read(path)
        .map_err(|err| TlsConfigError::Io {
            message: format!("read {} failed: {err}", path.display()),
        })
        .and_then(|pem| {
            parse_pem_private_key(pem.as_slice()).map_err(|err| TlsConfigError::PemParse {
                message: format!("parse private key `{}` failed: {}", path.display(), err),
            })
        })
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use rustls::pki_types::UnixTime;

    use crate::{
        config_v2::types::{ApiTransport as ApiTransportV2, TlsConfig},
        dev_support::{
            test_fs::{unique_test_dir, write_text_file},
            tls::build_adversarial_tls_fixture,
        },
    };

    use super::{build_api_server_config_v2, build_client_verifier_from_paths, TlsConfigError};
    use super::{load_pem_cert_chain, load_pem_private_key};

    fn sample_validation_time() -> UnixTime {
        UnixTime::since_unix_epoch(Duration::from_secs(1_735_689_600))
    }

    #[test]
    fn build_api_server_config_accepts_path_identity_and_optional_client_auth(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let dir = unique_test_dir("tls", "server-config-ok")?;
        let cfg = ApiTransportV2::Https {
            tls: TlsConfig {
                cert: write_text_file(
                    dir.as_path(),
                    "server.crt",
                    fixture.valid_server.cert_pem.as_str(),
                )?,
                key: write_text_file(
                    dir.as_path(),
                    "server.key",
                    fixture.valid_server.key_pem.as_str(),
                )?,
                ca_cert: None,
            },
            client_ca: Some(write_text_file(
                dir.as_path(),
                "client-ca.crt",
                fixture.trusted_client_ca.cert.cert_pem.as_str(),
            )?),
            client_cert_required: false,
            allowed_client_common_names: Vec::new(),
        };

        let built = build_api_server_config_v2(&cfg)?;
        assert_eq!(
            built.alpn_protocols,
            vec![b"h2".to_vec(), b"http/1.1".to_vec()]
        );
        Ok(())
    }

    #[test]
    fn build_api_server_config_reports_io_error_when_cert_path_missing() {
        let cfg = ApiTransportV2::Https {
            tls: TlsConfig {
                cert: PathBuf::from("/tmp/pgtuskmaster-missing-cert-chain.pem"),
                key: PathBuf::from("/tmp/pgtuskmaster-missing-private-key.pem"),
                ca_cert: None,
            },
            client_ca: None,
            client_cert_required: false,
            allowed_client_common_names: Vec::new(),
        };

        let result = build_api_server_config_v2(&cfg);
        assert!(matches!(result, Err(TlsConfigError::Io { .. })));
    }

    #[test]
    fn build_api_server_config_reports_pem_error_for_invalid_cert_chain() -> Result<(), String> {
        let dir = unique_test_dir("tls", "invalid-cert")?;
        let cfg = ApiTransportV2::Https {
            tls: TlsConfig {
                cert: write_text_file(dir.as_path(), "server.crt", "not-a-cert")?,
                key: write_text_file(dir.as_path(), "server.key", "not-a-key")?,
                ca_cert: None,
            },
            client_ca: None,
            client_cert_required: false,
            allowed_client_common_names: Vec::new(),
        };

        let result = build_api_server_config_v2(&cfg);
        assert!(matches!(result, Err(TlsConfigError::PemParse { .. })));
        Ok(())
    }

    #[test]
    fn build_api_server_config_reports_pem_error_for_invalid_private_key(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let dir = unique_test_dir("tls", "invalid-key")?;
        let cfg = ApiTransportV2::Https {
            tls: TlsConfig {
                cert: write_text_file(
                    dir.as_path(),
                    "server.crt",
                    fixture.valid_server.cert_pem.as_str(),
                )?,
                key: write_text_file(dir.as_path(), "server.key", "not-a-private-key")?,
                ca_cert: None,
            },
            client_ca: None,
            client_cert_required: false,
            allowed_client_common_names: Vec::new(),
        };

        let result = build_api_server_config_v2(&cfg);
        assert!(matches!(result, Err(TlsConfigError::PemParse { .. })));
        Ok(())
    }

    #[test]
    fn api_client_verifier_rejects_client_signed_by_unconfigured_ca(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let dir = unique_test_dir("tls", "verifier-untrusted")?;
        let ca_path = write_text_file(
            dir.as_path(),
            "client-ca.crt",
            fixture.trusted_client_ca.cert.cert_pem.as_str(),
        )?;
        let verifier = build_client_verifier_from_paths(
            Some(ca_path.as_path()),
            true,
            &["trusted-client".to_string()],
        )?;

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
        let dir = unique_test_dir("tls", "verifier-cn")?;
        let ca_path = write_text_file(
            dir.as_path(),
            "client-ca.crt",
            fixture.trusted_client_ca.cert.cert_pem.as_str(),
        )?;
        let verifier = build_client_verifier_from_paths(
            Some(ca_path.as_path()),
            true,
            &["ops-admin".to_string()],
        )?;

        let result = verifier.verify_client_cert(
            &fixture.trusted_client.cert_der(),
            &[],
            sample_validation_time(),
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn load_pem_cert_chain_reads_valid_chain_from_path() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let dir = unique_test_dir("tls", "load-cert-chain")?;
        let cert_path = write_text_file(
            dir.as_path(),
            "client-ca.crt",
            fixture.trusted_client_ca.cert.cert_pem.as_str(),
        )?;

        let certs = load_pem_cert_chain(cert_path.as_path())?;

        assert_eq!(certs.len(), 1);
        Ok(())
    }

    #[test]
    fn load_pem_private_key_reads_valid_key_from_path() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let dir = unique_test_dir("tls", "load-private-key")?;
        let key_path = write_text_file(
            dir.as_path(),
            "client.key",
            fixture.trusted_client.key_pem.as_str(),
        )?;

        let _key = load_pem_private_key(key_path.as_path())?;

        Ok(())
    }

    #[test]
    fn load_pem_cert_chain_reports_path_in_parse_error() -> Result<(), Box<dyn std::error::Error>> {
        let dir = unique_test_dir("tls", "load-cert-chain-invalid")?;
        let cert_path = write_text_file(dir.as_path(), "broken.crt", "not-a-cert")?;

        let err = match load_pem_cert_chain(cert_path.as_path()) {
            Ok(_) => return Err("expected certificate parse to fail".into()),
            Err(err) => err,
        };

        assert!(err.contains("broken.crt"));
        assert!(err.contains("parse certificate"));
        Ok(())
    }
}
