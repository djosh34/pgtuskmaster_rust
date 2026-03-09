#[cfg(test)]
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
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
pub struct TlsMaterial {
    pub ca_cert: Option<PathBuf>,
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
}

pub fn write_tls_material(
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
pub struct GeneratedCert {
    pub cert_der: Vec<u8>,
    pub key_der: Vec<u8>,
    pub cert_pem: String,
    pub key_pem: String,
}

#[cfg(test)]
impl GeneratedCert {
    pub fn cert_der(&self) -> CertificateDer<'static> {
        CertificateDer::from(self.cert_der.clone())
    }

    pub fn key_der(&self) -> PrivateKeyDer<'static> {
        PrivateKeyDer::from(PrivatePkcs8KeyDer::from(self.key_der.clone()))
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestSubjectAltName {
    Dns(String),
    Ip(IpAddr),
}

#[cfg(test)]
impl TestSubjectAltName {
    fn to_certificate_param(&self) -> String {
        match self {
            Self::Dns(name) => name.clone(),
            Self::Ip(addr) => addr.to_string(),
        }
    }
}

#[cfg(test)]
fn localhost_server_sans() -> Vec<TestSubjectAltName> {
    vec![TestSubjectAltName::Dns("localhost".to_string())]
}

#[cfg(test)]
fn ipv4_loopback_server_sans() -> Vec<TestSubjectAltName> {
    vec![TestSubjectAltName::Ip(IpAddr::V4(Ipv4Addr::LOCALHOST))]
}

#[cfg(test)]
fn mixed_loopback_server_sans() -> Vec<TestSubjectAltName> {
    vec![
        TestSubjectAltName::Dns("localhost".to_string()),
        TestSubjectAltName::Ip(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        TestSubjectAltName::Ip(IpAddr::V6(Ipv6Addr::LOCALHOST)),
    ]
}

#[cfg(test)]
#[derive(Debug)]
pub struct GeneratedCa {
    pub cert: GeneratedCert,
    issuer: Issuer<'static, KeyPair>,
}

#[cfg(test)]
impl GeneratedCa {
    pub fn issuer(&self) -> &Issuer<'static, KeyPair> {
        &self.issuer
    }
}

#[cfg(test)]
#[derive(Debug)]
pub struct AdversarialTlsFixture {
    pub valid_server_ca: GeneratedCa,
    pub wrong_server_ca: GeneratedCa,
    pub valid_server: GeneratedCert,
    pub ipv4_loopback_server: GeneratedCert,
    pub mixed_loopback_server: GeneratedCert,
    pub expired_server: GeneratedCert,
    pub trusted_client_ca: GeneratedCa,
    pub trusted_client: GeneratedCert,
    pub untrusted_client_ca: GeneratedCa,
    pub untrusted_client: GeneratedCert,
}

#[cfg(test)]
pub fn build_adversarial_tls_fixture() -> Result<AdversarialTlsFixture, HarnessError> {
    let valid_server_ca = generate_ca("server-valid-ca")?;
    let wrong_server_ca = generate_ca("server-wrong-ca")?;
    let trusted_client_ca = generate_ca("trusted-client-ca")?;
    let untrusted_client_ca = generate_ca("untrusted-client-ca")?;

    let valid_server = generate_leaf_cert(
        "server-valid",
        &localhost_server_sans(),
        false,
        valid_server_ca.issuer(),
        false,
    )?;
    let ipv4_loopback_server = generate_leaf_cert(
        "server-ipv4-loopback",
        &ipv4_loopback_server_sans(),
        false,
        valid_server_ca.issuer(),
        false,
    )?;
    let mixed_loopback_server = generate_leaf_cert(
        "server-mixed-loopback",
        &mixed_loopback_server_sans(),
        false,
        valid_server_ca.issuer(),
        false,
    )?;
    let expired_server = generate_leaf_cert(
        "server-expired",
        &localhost_server_sans(),
        true,
        valid_server_ca.issuer(),
        false,
    )?;
    let trusted_client = generate_leaf_cert(
        "trusted-client",
        &localhost_server_sans(),
        false,
        trusted_client_ca.issuer(),
        true,
    )?;
    let untrusted_client = generate_leaf_cert(
        "untrusted-client",
        &localhost_server_sans(),
        false,
        untrusted_client_ca.issuer(),
        true,
    )?;

    Ok(AdversarialTlsFixture {
        valid_server_ca,
        wrong_server_ca,
        valid_server,
        ipv4_loopback_server,
        mixed_loopback_server,
        expired_server,
        trusted_client_ca,
        trusted_client,
        untrusted_client_ca,
        untrusted_client,
    })
}

#[cfg(test)]
pub fn generate_ca(common_name: &str) -> Result<GeneratedCa, HarnessError> {
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
pub fn generate_leaf_cert(
    common_name: &str,
    subject_alt_names: &[TestSubjectAltName],
    expired: bool,
    issuer: &Issuer<'static, KeyPair>,
    is_client_cert: bool,
) -> Result<GeneratedCert, HarnessError> {
    let san_values = subject_alt_names
        .iter()
        .map(TestSubjectAltName::to_certificate_param)
        .collect::<Vec<_>>();
    let mut params = CertificateParams::new(san_values)
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
pub fn build_server_config(
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
pub fn build_server_config_with_client_auth(
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
pub fn build_client_config(
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
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use x509_parser::{extensions::GeneralName, parse_x509_certificate};

    use crate::test_harness::{namespace::NamespaceGuard, HarnessError};

    use super::{
        build_adversarial_tls_fixture, build_client_config, build_server_config,
        build_server_config_with_client_auth, write_tls_material, GeneratedCert,
        TestSubjectAltName,
    };

    fn certificate_subject_alt_names(
        cert: &GeneratedCert,
    ) -> Result<Vec<TestSubjectAltName>, HarnessError> {
        let (_remaining, parsed) =
            parse_x509_certificate(cert.cert_der.as_slice()).map_err(|err| {
                HarnessError::InvalidInput(format!("parse certificate failed: {err}"))
            })?;
        let extension = parsed
            .subject_alternative_name()
            .map_err(|err| HarnessError::InvalidInput(format!("read SAN extension failed: {err}")))?
            .ok_or_else(|| {
                HarnessError::InvalidInput("certificate missing SAN extension".to_string())
            })?;

        extension
            .value
            .general_names
            .iter()
            .map(|name| match name {
                GeneralName::DNSName(value) => Ok(TestSubjectAltName::Dns(value.to_string())),
                GeneralName::IPAddress(bytes) => match bytes.len() {
                    4 => Ok(TestSubjectAltName::Ip(IpAddr::V4(Ipv4Addr::new(
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    )))),
                    16 => {
                        let octets = <[u8; 16]>::try_from(*bytes).map_err(|_| {
                            HarnessError::InvalidInput("invalid IPv6 SAN octet length".to_string())
                        })?;
                        Ok(TestSubjectAltName::Ip(IpAddr::V6(Ipv6Addr::from(octets))))
                    }
                    len => Err(HarnessError::InvalidInput(format!(
                        "unsupported IP SAN octet length: {len}"
                    ))),
                },
                other => Err(HarnessError::InvalidInput(format!(
                    "unsupported SAN entry in fixture certificate: {other:?}"
                ))),
            })
            .collect()
    }

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
    fn adversarial_fixture_publishes_expected_server_san_variants() -> Result<(), HarnessError> {
        let fixture = build_adversarial_tls_fixture()?;

        assert_eq!(
            certificate_subject_alt_names(&fixture.valid_server)?,
            vec![TestSubjectAltName::Dns("localhost".to_string())]
        );
        assert_eq!(
            certificate_subject_alt_names(&fixture.ipv4_loopback_server)?,
            vec![TestSubjectAltName::Ip(IpAddr::V4(Ipv4Addr::LOCALHOST))]
        );
        assert_eq!(
            certificate_subject_alt_names(&fixture.mixed_loopback_server)?,
            vec![
                TestSubjectAltName::Dns("localhost".to_string()),
                TestSubjectAltName::Ip(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                TestSubjectAltName::Ip(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            ]
        );
        assert_eq!(
            certificate_subject_alt_names(&fixture.expired_server)?,
            vec![TestSubjectAltName::Dns("localhost".to_string())]
        );
        assert_ne!(
            fixture.valid_server_ca.cert.cert_pem,
            fixture.wrong_server_ca.cert.cert_pem
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
        let _ipv4_server_cfg =
            build_server_config(&fixture.ipv4_loopback_server, &fixture.valid_server_ca.cert)?;
        let _mixed_server_cfg = build_server_config(
            &fixture.mixed_loopback_server,
            &fixture.valid_server_ca.cert,
        )?;
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
