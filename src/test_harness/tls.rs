use std::path::PathBuf;
use std::{fs, path::Path};

use super::{namespace::TestNamespace, HarnessError};
#[cfg(test)]
use rcgen::{
    date_time_ymd, BasicConstraints, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair, KeyUsagePurpose,
};
#[cfg(test)]
use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
    ClientConfig, RootCertStore, ServerConfig,
};
#[cfg(test)]
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TlsMaterial {
    pub(crate) ca_cert: Option<PathBuf>,
    pub(crate) cert: Option<PathBuf>,
    pub(crate) key: Option<PathBuf>,
}

pub(crate) fn write_tls_material(
    namespace: &TestNamespace,
    profile: &str,
    ca_pem: Option<&[u8]>,
    cert_pem: Option<&[u8]>,
    key_pem: Option<&[u8]>,
) -> Result<TlsMaterial, HarnessError> {
    if profile.trim().is_empty() {
        return Err(HarnessError::InvalidInput(
            "tls profile must not be empty".to_string(),
        ));
    }

    let base = namespace.child_dir(format!("security/tls/{}", sanitize_profile(profile)));
    fs::create_dir_all(&base)?;

    let ca_cert = write_optional_file(&base, "ca.crt", ca_pem)?;
    let cert = write_optional_file(&base, "server.crt", cert_pem)?;
    let key = write_optional_file(&base, "server.key", key_pem)?;

    Ok(TlsMaterial { ca_cert, cert, key })
}

fn write_optional_file(
    base: &Path,
    file_name: &str,
    contents: Option<&[u8]>,
) -> Result<Option<PathBuf>, HarnessError> {
    match contents {
        Some(bytes) => {
            if bytes.is_empty() {
                return Err(HarnessError::InvalidInput(format!(
                    "tls file {file_name} contents must not be empty"
                )));
            }

            let path = base.join(file_name);
            fs::write(&path, bytes)?;
            Ok(Some(path))
        }
        None => Ok(None),
    }
}

fn sanitize_profile(profile: &str) -> String {
    let mut out = String::with_capacity(profile.len());
    for ch in profile.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }

    if out.is_empty() {
        "tls-profile".to_string()
    } else {
        out
    }
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct GeneratedCert {
    pub(crate) cert_der: Vec<u8>,
    pub(crate) key_der: Vec<u8>,
    pub(crate) cert_pem: String,
    pub(crate) key_pem: String,
}

#[cfg(test)]
impl GeneratedCert {
    pub(crate) fn cert_der(&self) -> CertificateDer<'static> {
        CertificateDer::from(self.cert_der.clone())
    }

    pub(crate) fn key_der(&self) -> PrivateKeyDer<'static> {
        PrivateKeyDer::from(PrivatePkcs8KeyDer::from(self.key_der.clone()))
    }
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct GeneratedCa {
    pub(crate) cert: GeneratedCert,
    issuer: Issuer<'static, KeyPair>,
}

#[cfg(test)]
impl GeneratedCa {
    pub(crate) fn issuer(&self) -> &Issuer<'static, KeyPair> {
        &self.issuer
    }
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct AdversarialTlsFixture {
    pub(crate) valid_server_ca: GeneratedCa,
    pub(crate) wrong_server_ca: GeneratedCa,
    pub(crate) valid_server: GeneratedCert,
    pub(crate) expired_server: GeneratedCert,
    pub(crate) trusted_client_ca: GeneratedCa,
    pub(crate) trusted_client: GeneratedCert,
    pub(crate) untrusted_client_ca: GeneratedCa,
    pub(crate) untrusted_client: GeneratedCert,
}

#[cfg(test)]
pub(crate) fn build_adversarial_tls_fixture() -> Result<AdversarialTlsFixture, HarnessError> {
    let valid_server_ca = generate_ca("server-valid-ca")?;
    let wrong_server_ca = generate_ca("server-wrong-ca")?;
    let trusted_client_ca = generate_ca("trusted-client-ca")?;
    let untrusted_client_ca = generate_ca("untrusted-client-ca")?;

    let valid_server = generate_leaf_cert(
        "server-valid",
        "localhost",
        false,
        valid_server_ca.issuer(),
        false,
    )?;
    let expired_server = generate_leaf_cert(
        "server-expired",
        "localhost",
        true,
        valid_server_ca.issuer(),
        false,
    )?;
    let trusted_client = generate_leaf_cert(
        "trusted-client",
        "localhost",
        false,
        trusted_client_ca.issuer(),
        true,
    )?;
    let untrusted_client = generate_leaf_cert(
        "untrusted-client",
        "localhost",
        false,
        untrusted_client_ca.issuer(),
        true,
    )?;

    Ok(AdversarialTlsFixture {
        valid_server_ca,
        wrong_server_ca,
        valid_server,
        expired_server,
        trusted_client_ca,
        trusted_client,
        untrusted_client_ca,
        untrusted_client,
    })
}

#[cfg(test)]
pub(crate) fn generate_ca(common_name: &str) -> Result<GeneratedCa, HarnessError> {
    let mut params = CertificateParams::new(Vec::new())
        .map_err(|err| HarnessError::InvalidInput(format!("create ca params failed: {err}")))?;
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, common_name.to_string());
    params.distinguished_name = dn;
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);
    params.not_before = date_time_ymd(2024, 1, 1);
    params.not_after = date_time_ymd(2034, 1, 1);

    let key_pair = KeyPair::generate()
        .map_err(|err| HarnessError::InvalidInput(format!("generate ca key failed: {err}")))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|err| HarnessError::InvalidInput(format!("self-sign ca failed: {err}")))?;

    Ok(GeneratedCa {
        cert: GeneratedCert {
            cert_der: cert.der().to_vec(),
            key_der: key_pair.serialize_der(),
            cert_pem: cert.pem(),
            key_pem: key_pair.serialize_pem(),
        },
        issuer: Issuer::new(params, key_pair),
    })
}

#[cfg(test)]
pub(crate) fn generate_leaf_cert(
    common_name: &str,
    dns_name: &str,
    expired: bool,
    issuer: &Issuer<'static, KeyPair>,
    is_client_cert: bool,
) -> Result<GeneratedCert, HarnessError> {
    let mut params = CertificateParams::new(vec![dns_name.to_string()])
        .map_err(|err| HarnessError::InvalidInput(format!("create leaf params failed: {err}")))?;
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, common_name.to_string());
    params.distinguished_name = dn;
    params.is_ca = IsCa::NoCa;
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    if is_client_cert {
        params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ClientAuth);
    } else {
        params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ServerAuth);
    }
    if expired {
        params.not_before = date_time_ymd(2018, 1, 1);
        params.not_after = date_time_ymd(2019, 1, 1);
    } else {
        params.not_before = date_time_ymd(2024, 1, 1);
        params.not_after = date_time_ymd(2034, 1, 1);
    }

    let key_pair = KeyPair::generate()
        .map_err(|err| HarnessError::InvalidInput(format!("generate leaf key failed: {err}")))?;
    let cert = params
        .signed_by(&key_pair, issuer)
        .map_err(|err| HarnessError::InvalidInput(format!("sign leaf cert failed: {err}")))?;

    Ok(GeneratedCert {
        cert_der: cert.der().to_vec(),
        key_der: key_pair.serialize_der(),
        cert_pem: cert.pem(),
        key_pem: key_pair.serialize_pem(),
    })
}

#[cfg(test)]
pub(crate) fn build_server_config(
    server: &GeneratedCert,
    server_ca: &GeneratedCert,
) -> Result<Arc<ServerConfig>, HarnessError> {
    let provider = rustls::crypto::ring::default_provider();
    let builder = ServerConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .map_err(|err| HarnessError::InvalidInput(format!("build server config failed: {err}")))?;

    let config = builder
        .with_no_client_auth()
        .with_single_cert(
            vec![server.cert_der(), server_ca.cert_der()],
            server.key_der(),
        )
        .map_err(|err| HarnessError::InvalidInput(format!("build server config failed: {err}")))?;
    Ok(Arc::new(config))
}

#[cfg(test)]
pub(crate) fn build_server_config_with_client_auth(
    server: &GeneratedCert,
    server_ca: &GeneratedCert,
    trusted_client_ca: &GeneratedCert,
) -> Result<Arc<ServerConfig>, HarnessError> {
    let mut roots = RootCertStore::empty();
    roots.add(trusted_client_ca.cert_der()).map_err(|err| {
        HarnessError::InvalidInput(format!("add trusted client root failed: {err}"))
    })?;

    let provider = rustls::crypto::ring::default_provider();
    let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
        Arc::new(roots),
        provider.into(),
    )
        .build()
        .map_err(|err| {
            HarnessError::InvalidInput(format!("build client cert verifier failed: {err}"))
        })?;

    let provider = rustls::crypto::ring::default_provider();
    let builder = ServerConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .map_err(|err| {
            HarnessError::InvalidInput(format!("build mTLS server config failed: {err}"))
        })?;

    let config = builder
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![server.cert_der(), server_ca.cert_der()],
            server.key_der(),
        )
        .map_err(|err| {
            HarnessError::InvalidInput(format!("build mTLS server config failed: {err}"))
        })?;

    Ok(Arc::new(config))
}

#[cfg(test)]
pub(crate) fn build_client_config(
    trusted_server_ca: &GeneratedCert,
    identity: Option<&GeneratedCert>,
    identity_ca: Option<&GeneratedCert>,
) -> Result<Arc<ClientConfig>, HarnessError> {
    let mut roots = RootCertStore::empty();
    roots.add(trusted_server_ca.cert_der()).map_err(|err| {
        HarnessError::InvalidInput(format!("add trusted server root failed: {err}"))
    })?;

    let provider = rustls::crypto::ring::default_provider();
    let builder = ClientConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .map_err(|err| HarnessError::InvalidInput(format!("build client config failed: {err}")))?
        .with_root_certificates(roots);
    let config = match identity {
        Some(cert) => builder
            .with_client_auth_cert(
                vec![
                    cert.cert_der(),
                    identity_ca.map(GeneratedCert::cert_der).ok_or_else(|| {
                        HarnessError::InvalidInput(
                            "identity_ca is required when identity is configured".to_string(),
                        )
                    })?,
                ],
                cert.key_der(),
            )
            .map_err(|err| {
                HarnessError::InvalidInput(format!("build mTLS client config failed: {err}"))
            })?,
        None => builder.with_no_client_auth(),
    };

    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use crate::test_harness::{namespace::NamespaceGuard, HarnessError};

    use super::{
        build_adversarial_tls_fixture, build_client_config, build_server_config,
        build_server_config_with_client_auth, write_tls_material,
    };

    #[test]
    fn write_tls_material_writes_files_under_namespace() -> Result<(), HarnessError> {
        let guard = NamespaceGuard::new("tls-material")?;
        let namespace = guard.namespace()?;
        let material = write_tls_material(
            namespace,
            "node-a",
            Some(b"ca-bytes"),
            Some(b"cert-bytes"),
            Some(b"key-bytes"),
        )?;

        let ca = material
            .ca_cert
            .ok_or_else(|| HarnessError::InvalidInput("expected ca cert path".to_string()))?;
        let cert = material
            .cert
            .ok_or_else(|| HarnessError::InvalidInput("expected cert path".to_string()))?;
        let key = material
            .key
            .ok_or_else(|| HarnessError::InvalidInput("expected key path".to_string()))?;

        assert!(ca.exists());
        assert!(cert.exists());
        assert!(key.exists());
        Ok(())
    }

    #[test]
    fn adversarial_fixture_builds_distinct_material() -> Result<(), HarnessError> {
        let fixture = build_adversarial_tls_fixture()?;

        assert_ne!(
            fixture.valid_server_ca.cert.cert_pem,
            fixture.wrong_server_ca.cert.cert_pem
        );
        assert_ne!(
            fixture.valid_server.cert_pem,
            fixture.expired_server.cert_pem
        );
        assert_ne!(
            fixture.trusted_client_ca.cert.cert_pem,
            fixture.untrusted_client_ca.cert.cert_pem
        );
        assert_ne!(
            fixture.trusted_client.cert_pem,
            fixture.untrusted_client.cert_pem
        );
        assert!(!fixture.valid_server.key_pem.is_empty());
        assert!(!fixture.expired_server.key_pem.is_empty());
        Ok(())
    }

    #[test]
    fn tls_builders_accept_fixture_material() -> Result<(), HarnessError> {
        let fixture = build_adversarial_tls_fixture()?;

        let _server_cfg =
            build_server_config(&fixture.valid_server, &fixture.valid_server_ca.cert)?;
        let _mtls_server_cfg = build_server_config_with_client_auth(
            &fixture.valid_server,
            &fixture.valid_server_ca.cert,
            &fixture.trusted_client_ca.cert,
        )?;
        let _client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let _trusted_mtls_client = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.trusted_client),
            Some(&fixture.trusted_client_ca.cert),
        )?;
        Ok(())
    }

    #[test]
    fn client_identity_requires_identity_ca() -> Result<(), HarnessError> {
        let fixture = build_adversarial_tls_fixture()?;
        let result = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.trusted_client),
            None,
        );
        match result {
            Err(HarnessError::InvalidInput(message)) => {
                assert!(message.contains("identity_ca is required"));
                Ok(())
            }
            Err(other) => Err(HarnessError::InvalidInput(format!(
                "unexpected error variant: {other}"
            ))),
            Ok(_) => Err(HarnessError::InvalidInput(
                "expected missing identity_ca failure".to_string(),
            )),
        }
    }
}
