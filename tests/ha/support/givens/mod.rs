use std::{
    fs,
    path::{Path, PathBuf},
};

use pgtuskmaster_test_support::config_v2::{
    load_runtime_config_contents, render_runtime_test_config_toml_with_overrides, toml_path_source,
    toml_string, toml_string_secret,
};

use crate::support::{
    error::{HarnessError, Result},
    faults::DCS_SERVICES,
    files::{create_dir_all, write_text_file},
    topology::{ClusterMember, DcsService},
};

const SHARED_FIXTURE_RELATIVE_PATHS: [&str; 4] = [
    "configs/tls",
    "secrets",
    "configs/pg_hba.conf",
    "configs/pg_ident.conf",
];
const HA_SUPERUSER_PASSWORD: &str = "ha-cucumber-superuser-password";
const HA_REPLICATOR_PASSWORD: &str = "ha-cucumber-superuser-password";
const HA_REWINDER_PASSWORD: &str = "ha-cucumber-superuser-password";
const HA_API_READ_TOKEN: &str = "ha-cucumber-read-token";
const HA_API_ADMIN_TOKEN: &str = "ha-cucumber-admin-token";
const CONTAINER_PROCESS_BINARY_OVERRIDES: [&str; 4] = [
    "/usr/lib/postgresql/16/bin/pg_ctl",
    "/usr/local/lib/pgtuskmaster/wrappers/pg_rewind",
    "/usr/lib/postgresql/16/bin/initdb",
    "/usr/local/lib/pgtuskmaster/wrappers/pg_basebackup",
];
const HOST_VALIDATION_PROCESS_BINARY_OVERRIDES: [&str; 4] =
    ["/bin/true", "/bin/true", "/bin/true", "/bin/true"];
const HA_POSTGRES_HBA_CONTENTS: &str = r#"local   all             all                                     peer
hostnossl all           all             0.0.0.0/0               reject
hostnossl replication   all             0.0.0.0/0               reject
hostssl all            postgres        0.0.0.0/0               cert clientname=CN map=observer_as_postgres
hostssl all            all             127.0.0.1/32            scram-sha-256
hostssl all            all             ::1/128                 scram-sha-256
hostssl all            all             0.0.0.0/0               scram-sha-256
hostssl replication    all             127.0.0.1/32            scram-sha-256
hostssl replication    all             0.0.0.0/0               scram-sha-256"#;
const HA_POSTGRES_IDENT_CONTENTS: &str = "observer_as_postgres    observer        postgres";
const SHARED_ETCD_SERVICES: [DcsService; 1] = [DcsService::SharedEtcd];
const THREE_ETCD_QUORUM_SERVICES: [DcsService; 2] = [DcsService::EtcdA, DcsService::EtcdB];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HaGivenId {
    Plain,
    CustomRoles,
    ThreeEtcd,
}

impl HaGivenId {
    pub fn local_dcs_service_for(self, member: ClusterMember) -> DcsService {
        match self {
            Self::Plain | Self::CustomRoles => DcsService::SharedEtcd,
            Self::ThreeEtcd => member.local_dcs_service(),
        }
    }

    pub fn artifact_service_names(self) -> Vec<&'static str> {
        self.dcs_services()
            .iter()
            .copied()
            .map(DcsService::service_name)
            .chain(
                ClusterMember::ALL
                    .into_iter()
                    .map(ClusterMember::service_name),
            )
            .collect()
    }

    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "three_node_plain" => Ok(Self::Plain),
            "three_node_custom_roles" => Ok(Self::CustomRoles),
            "three_node_three_etcd" => Ok(Self::ThreeEtcd),
            _ => Err(HarnessError::message(format!(
                "unsupported HA given `{raw}`"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "three_node_plain",
            Self::CustomRoles => "three_node_custom_roles",
            Self::ThreeEtcd => "three_node_three_etcd",
        }
    }

    pub fn replicator_role(self) -> &'static str {
        match self {
            Self::Plain | Self::ThreeEtcd => "replicator",
            Self::CustomRoles => "mirrorbot",
        }
    }

    pub fn rewinder_role(self) -> &'static str {
        match self {
            Self::Plain | Self::ThreeEtcd => "rewinder",
            Self::CustomRoles => "rewindbot",
        }
    }

    pub fn dcs_services(self) -> &'static [DcsService] {
        match self {
            Self::Plain | Self::CustomRoles => &SHARED_ETCD_SERVICES,
            Self::ThreeEtcd => &DCS_SERVICES,
        }
    }

    pub fn quorum_majority_dcs_services(self) -> &'static [DcsService] {
        match self {
            Self::Plain | Self::CustomRoles => &SHARED_ETCD_SERVICES,
            Self::ThreeEtcd => &THREE_ETCD_QUORUM_SERVICES,
        }
    }

    pub fn compose_variant_absolute_path(self, repo_root: &Path) -> Result<PathBuf> {
        let absolute = repo_root
            .join("tests/ha/givens")
            .join(self.compose_variant_relative_path());
        if absolute.is_file() {
            Ok(absolute)
        } else {
            Err(HarnessError::message(format!(
                "static compose variant is missing: {}",
                absolute.display()
            )))
        }
    }

    pub fn render_compose_include_file(
        self,
        repo_root: &Path,
        materialized_root: &Path,
    ) -> Result<String> {
        let compose_variant_path = self.compose_variant_absolute_path(repo_root)?;
        Ok(format!(
            "include:\n  - path: {:?}\n    project_directory: {:?}\n",
            compose_variant_path.display().to_string(),
            materialized_root.display().to_string(),
        ))
    }

    pub fn render_runtime_config(self, member: ClusterMember) -> Result<String> {
        let rendered = self.render_runtime_config_with_process_binaries(
            member,
            CONTAINER_PROCESS_BINARY_OVERRIDES,
        );
        self.validate_runtime_config_for_host(member)?;
        Ok(rendered)
    }

    pub fn materialize_fixture(self, repo_root: &Path, materialized_root: &Path) -> Result<()> {
        let shared_root = repo_root.join("tests/ha/givens/three_node_shared");
        for relative_path in self.shared_fixture_relative_paths() {
            copy_shared_fixture_path(
                shared_root.as_path(),
                materialized_root,
                Path::new(relative_path),
            )?;
        }
        for member in ClusterMember::ALL {
            let target_path = materialized_root.join(member.runtime_config_relative_path());
            if let Some(parent) = target_path.parent() {
                create_dir_all(parent)?;
            }
            write_text_file(
                target_path.as_path(),
                self.render_runtime_config(member)?.as_str(),
            )?;
        }
        write_text_file(
            materialized_root.join("compose.yml").as_path(),
            self.render_compose_include_file(repo_root, materialized_root)?
                .as_str(),
        )?;
        Ok(())
    }

    fn render_runtime_config_with_process_binaries(
        self,
        member: ClusterMember,
        process_binary_overrides: [&str; 4],
    ) -> String {
        render_ha_member_runtime_test_config_toml(
            member.service_name(),
            self.local_dcs_service_for(member).client_url(),
            self.replicator_role(),
            self.rewinder_role(),
            process_binary_overrides,
        )
    }

    fn compose_variant_relative_path(self) -> &'static str {
        match self {
            Self::Plain | Self::CustomRoles => "compose/three_node_shared_single.yml",
            Self::ThreeEtcd => "compose/three_node_three_etcd.yml",
        }
    }

    fn shared_fixture_relative_paths(self) -> &'static [&'static str] {
        match self {
            Self::Plain | Self::CustomRoles | Self::ThreeEtcd => &SHARED_FIXTURE_RELATIVE_PATHS,
        }
    }

    fn validate_runtime_config_for_host(self, member: ClusterMember) -> Result<()> {
        let rendered = self.render_runtime_config_with_process_binaries(
            member,
            HOST_VALIDATION_PROCESS_BINARY_OVERRIDES,
        );
        load_runtime_config_contents(rendered.as_str())
            .map(|_| ())
            .map_err(|source| HarnessError::message(source.to_string()))
    }
}

fn render_ha_member_runtime_test_config_toml(
    member_name: &str,
    dcs_endpoint: &str,
    replicator: &str,
    rewinder: &str,
    process_binary_overrides: [&str; 4],
) -> String {
    let ca_cert_path = Path::new("/etc/pgtuskmaster/tls/ca.crt");
    let member_cert_path = PathBuf::from(format!("/etc/pgtuskmaster/tls/{member_name}.crt"));
    let member_key_path = PathBuf::from(format!("/etc/pgtuskmaster/tls/{member_name}.key"));
    let [pg_ctl_path, pg_rewind_path, initdb_path, pg_basebackup_path] = process_binary_overrides;
    let ca_cert = toml_path_source(ca_cert_path);
    let member_cert_path = toml_path_source(member_cert_path.as_path());
    let member_key_path = toml_path_source(member_key_path.as_path());
    let api_read_token = toml_string_secret(HA_API_READ_TOKEN);
    let api_admin_token = toml_string_secret(HA_API_ADMIN_TOKEN);
    let api_base_url = toml_string(format!("https://{member_name}:8443").as_str());
    render_runtime_test_config_toml_with_overrides(
        ("ha-cucumber-cluster", "ha-cucumber-cluster", member_name),
        (
            Path::new("/var/lib/postgresql/data"),
            Some(Path::new("/var/lib/pgtuskmaster/socket")),
            Some(Path::new("/var/log/pgtuskmaster/postgres.log")),
        ),
        [dcs_endpoint],
        [
            ("postgres", HA_SUPERUSER_PASSWORD),
            (replicator, HA_REPLICATOR_PASSWORD),
            (rewinder, HA_REWINDER_PASSWORD),
        ],
        (HA_POSTGRES_HBA_CONTENTS, HA_POSTGRES_IDENT_CONTENTS),
        [
            format!(
                r#"[postgres.network]
listen_host = {member_name}
listen_port = 5432"#,
                member_name = toml_string(member_name),
            ),
            format!(
                r#"[postgres.rewind.transport]
ssl_mode = "verify-full"
ca_cert = {ca_cert}"#,
            ),
            format!(
                r#"[postgres.tls]
mode = "enabled"
identity = {{ cert_chain = {member_cert_path}, private_key = {member_key_path} }}
client_auth = {{ client_ca = {ca_cert}, client_certificate = "optional" }}"#,
            ),
            r#"[postgres.extra_gucs]
wal_keep_size = "128MB""#
                .to_string(),
            format!(
                r#"[process.binaries.overrides]
pg_ctl = {pg_ctl_path}
pg_rewind = {pg_rewind_path}
initdb = {initdb_path}
pg_basebackup = {pg_basebackup_path}"#,
                pg_ctl_path = toml_string(pg_ctl_path),
                pg_rewind_path = toml_string(pg_rewind_path),
                initdb_path = toml_string(initdb_path),
                pg_basebackup_path = toml_string(pg_basebackup_path),
            ),
            r#"[logging]
capture_subprocess_output = true

[logging.postgres]
enabled = true
poll_interval_ms = 200

[logging.postgres.cleanup]
enabled = true
max_files = 20
max_age_seconds = 86400
protect_recent_seconds = 300

[logging.sinks.file]
enabled = true
path = "/var/log/pgtuskmaster/runtime.jsonl"
mode = "append""#
                .to_string(),
            format!(
                r#"[api]
listen_addr = "0.0.0.0:8443"
transport = {{ transport = "https", tls = {{ identity = {{ cert_chain = {member_cert_path}, private_key = {member_key_path} }}, client_auth = {{ client_certificate = "disabled" }} }} }}
auth = {{ type = "role_tokens", tokens = {{ read_token = {api_read_token}, admin_token = {api_admin_token} }} }}"#,
            ),
            format!(
                r#"[pgtm.api]
base_url = {api_base_url}
auth = {{ type = "role_tokens", read_token = {api_read_token}, admin_token = {api_admin_token} }}
tls = {{ ca_cert = {ca_cert} }}

[pgtm.postgres]
tls = {{ ca_cert = {ca_cert} }}"#,
            ),
            r#"[debug]
enabled = true"#
                .to_string(),
        ],
    )
}

fn copy_file(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|source| HarnessError::Io {
            path: to.to_path_buf(),
            source,
        })?;
    apply_private_key_permissions(to)
}

fn apply_private_key_permissions(path: &Path) -> Result<()> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("key") {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions).map_err(|source| HarnessError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }

    Ok(())
}

fn copy_shared_fixture_path(
    shared_root: &Path,
    materialized_root: &Path,
    relative_path: &Path,
) -> Result<()> {
    let source_path = shared_root.join(relative_path);
    let target_path = materialized_root.join(relative_path);
    if source_path.is_dir() {
        return copy_directory(source_path.as_path(), target_path.as_path());
    }

    if let Some(parent) = target_path.parent() {
        create_dir_all(parent)?;
    }
    copy_file(source_path.as_path(), target_path.as_path())
}

fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Err(HarnessError::message(format!(
            "source directory does not exist: {}",
            from.display()
        )));
    }

    let mut directories = vec![(from.to_path_buf(), to.to_path_buf())];
    while let Some((current_from, current_to)) = directories.pop() {
        create_dir_all(current_to.as_path())?;
        for entry in fs::read_dir(current_from.as_path()).map_err(|source| HarnessError::Io {
            path: current_from.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| HarnessError::Io {
                path: current_from.clone(),
                source,
            })?;
            let source_path = entry.path();
            let destination_path = current_to.join(entry.file_name());
            if source_path.is_dir() {
                directories.push((source_path, destination_path));
            } else {
                copy_file(source_path.as_path(), destination_path.as_path())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use super::HaGivenId;
    use crate::support::{
        error::{HarnessError, Result},
        files::with_temporary_directory,
        topology::{ClusterMember, DcsService},
    };
    #[test]
    fn givens_expose_expected_static_topology_metadata() -> Result<()> {
        for (
            given,
            replicator_role,
            rewinder_role,
            dcs_services,
            quorum_majority,
            compose_variant,
        ) in [
            (
                HaGivenId::Plain,
                "replicator",
                "rewinder",
                &[DcsService::SharedEtcd][..],
                &[DcsService::SharedEtcd][..],
                "compose/three_node_shared_single.yml",
            ),
            (
                HaGivenId::CustomRoles,
                "mirrorbot",
                "rewindbot",
                &[DcsService::SharedEtcd][..],
                &[DcsService::SharedEtcd][..],
                "compose/three_node_shared_single.yml",
            ),
            (
                HaGivenId::ThreeEtcd,
                "replicator",
                "rewinder",
                &[DcsService::EtcdA, DcsService::EtcdB, DcsService::EtcdC][..],
                &[DcsService::EtcdA, DcsService::EtcdB][..],
                "compose/three_node_three_etcd.yml",
            ),
        ] {
            assert_eq!(given.replicator_role(), replicator_role);
            assert_eq!(given.rewinder_role(), rewinder_role);
            assert_eq!(given.dcs_services(), dcs_services);
            assert_eq!(given.quorum_majority_dcs_services(), quorum_majority);
            assert!(given
                .compose_variant_absolute_path(Path::new(env!("CARGO_MANIFEST_DIR")))?
                .ends_with(compose_variant));
        }
        Ok(())
    }

    #[test]
    fn givens_route_local_dcs_services_consistently() {
        for given in [HaGivenId::Plain, HaGivenId::CustomRoles] {
            assert_eq!(
                given.local_dcs_service_for(ClusterMember::NodeA),
                DcsService::SharedEtcd
            );
        }
        assert_eq!(
            HaGivenId::ThreeEtcd.local_dcs_service_for(ClusterMember::NodeA),
            DcsService::EtcdA
        );
        assert_eq!(
            HaGivenId::ThreeEtcd.local_dcs_service_for(ClusterMember::NodeB),
            DcsService::EtcdB
        );
    }

    #[test]
    fn renders_runtime_configs_from_given_owned_fixture_definition() -> Result<()> {
        for (given, member, expected_endpoint, expected_replicator, expected_rewinder) in [
            (
                HaGivenId::Plain,
                ClusterMember::NodeA,
                "http://etcd:2379",
                r#"username = "replicator""#,
                r#"username = "rewinder""#,
            ),
            (
                HaGivenId::CustomRoles,
                ClusterMember::NodeB,
                "http://etcd:2379",
                r#"username = "mirrorbot""#,
                r#"username = "rewindbot""#,
            ),
            (
                HaGivenId::ThreeEtcd,
                ClusterMember::NodeC,
                "http://etcd-c:2379",
                r#"username = "replicator""#,
                r#"username = "rewinder""#,
            ),
        ] {
            let rendered = given.render_runtime_config(member)?;
            given.validate_runtime_config_for_host(member)?;
            assert!(rendered.contains(expected_endpoint));
            assert!(rendered.contains(expected_replicator));
            assert!(rendered.contains(expected_rewinder));
        }
        Ok(())
    }

    #[test]
    fn materializes_expected_fixture_assets_and_rendered_configs() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        for (name, given, member, expected_replicator, expected_rewinder, expected_endpoint) in [
            (
                "plain",
                HaGivenId::Plain,
                ClusterMember::NodeA,
                r#"username = "replicator""#,
                r#"username = "rewinder""#,
                "http://etcd:2379",
            ),
            (
                "custom-roles",
                HaGivenId::CustomRoles,
                ClusterMember::NodeB,
                r#"username = "mirrorbot""#,
                r#"username = "rewindbot""#,
                "http://etcd:2379",
            ),
            (
                "three-etcd",
                HaGivenId::ThreeEtcd,
                ClusterMember::NodeA,
                r#"username = "replicator""#,
                r#"username = "rewinder""#,
                "http://etcd-a:2379",
            ),
        ] {
            with_temporary_directory("pgtm-ha-givens", name, |output_root| {
                given.materialize_fixture(repo_root.as_path(), output_root)?;

                let compose_path = output_root.join("compose.yml");
                let compose = fs::read_to_string(compose_path.as_path()).map_err(|source| {
                    HarnessError::Io {
                        path: compose_path.clone(),
                        source,
                    }
                })?;
                let expected_variant = given.compose_variant_absolute_path(repo_root.as_path())?;
                assert!(compose.contains("include:"));
                assert!(compose.contains(expected_variant.display().to_string().as_str()));
                assert!(compose.contains(output_root.display().to_string().as_str()));

                let runtime_path = output_root.join(member.runtime_config_relative_path());
                let runtime = fs::read_to_string(runtime_path.as_path()).map_err(|source| {
                    HarnessError::Io {
                        path: runtime_path.clone(),
                        source,
                    }
                })?;
                given
                    .validate_runtime_config_for_host(member)
                    .map_err(|source| {
                        HarnessError::message(format!(
                            "materialized node runtime config failed validation: {source}"
                        ))
                    })?;
                assert!(runtime.contains(expected_replicator));
                assert!(runtime.contains(expected_rewinder));
                assert!(runtime.contains(expected_endpoint));
                assert!(output_root.join("configs/tls/ca.crt").is_file());
                assert!(output_root.join("secrets/replicator-password").is_file());
                Ok(())
            })?;
        }

        Ok(())
    }
}
