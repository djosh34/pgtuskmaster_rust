use std::{
    fs,
    path::{Path, PathBuf},
};

use pgtuskmaster_test_support::config_v2::render_ha_member_runtime_config_toml;

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
const SHARED_ETCD_SERVICES: [DcsService; 1] = [DcsService::SharedEtcd];
const THREE_ETCD_QUORUM_SERVICES: [DcsService; 2] = [DcsService::EtcdA, DcsService::EtcdB];
const PLAIN_GIVEN: HaGivenDefinition = HaGivenDefinition {
    name: "three_node_plain",
    local_dcs_service_for: shared_dcs_service_for,
    replicator_role: "replicator",
    rewinder_role: "rewinder",
    dcs_services: &SHARED_ETCD_SERVICES,
    quorum_majority_dcs_services: &SHARED_ETCD_SERVICES,
    compose_variant_relative_path: "compose/three_node_shared_single.yml",
    shared_fixture_relative_paths: &SHARED_FIXTURE_RELATIVE_PATHS,
};
const CUSTOM_ROLES_GIVEN: HaGivenDefinition = HaGivenDefinition {
    name: "three_node_custom_roles",
    local_dcs_service_for: shared_dcs_service_for,
    replicator_role: "mirrorbot",
    rewinder_role: "rewindbot",
    dcs_services: &SHARED_ETCD_SERVICES,
    quorum_majority_dcs_services: &SHARED_ETCD_SERVICES,
    compose_variant_relative_path: "compose/three_node_shared_single.yml",
    shared_fixture_relative_paths: &SHARED_FIXTURE_RELATIVE_PATHS,
};
const THREE_ETCD_GIVEN: HaGivenDefinition = HaGivenDefinition {
    name: "three_node_three_etcd",
    local_dcs_service_for: member_local_dcs_service_for,
    replicator_role: "replicator",
    rewinder_role: "rewinder",
    dcs_services: &DCS_SERVICES,
    quorum_majority_dcs_services: &THREE_ETCD_QUORUM_SERVICES,
    compose_variant_relative_path: "compose/three_node_three_etcd.yml",
    shared_fixture_relative_paths: &SHARED_FIXTURE_RELATIVE_PATHS,
};

struct HaGivenDefinition {
    name: &'static str,
    local_dcs_service_for: fn(ClusterMember) -> DcsService,
    replicator_role: &'static str,
    rewinder_role: &'static str,
    dcs_services: &'static [DcsService],
    quorum_majority_dcs_services: &'static [DcsService],
    compose_variant_relative_path: &'static str,
    shared_fixture_relative_paths: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HaGivenId {
    Plain,
    CustomRoles,
    ThreeEtcd,
}

impl HaGivenId {
    fn definition(self) -> &'static HaGivenDefinition {
        match self {
            Self::Plain => &PLAIN_GIVEN,
            Self::CustomRoles => &CUSTOM_ROLES_GIVEN,
            Self::ThreeEtcd => &THREE_ETCD_GIVEN,
        }
    }

    pub fn local_dcs_service_for(self, member: ClusterMember) -> DcsService {
        (self.definition().local_dcs_service_for)(member)
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
        self.definition().name
    }

    pub fn replicator_role(self) -> &'static str {
        self.definition().replicator_role
    }

    pub fn rewinder_role(self) -> &'static str {
        self.definition().rewinder_role
    }

    pub fn dcs_services(self) -> &'static [DcsService] {
        self.definition().dcs_services
    }

    pub fn quorum_majority_dcs_services(self) -> &'static [DcsService] {
        self.definition().quorum_majority_dcs_services
    }

    pub fn compose_variant_absolute_path(self, repo_root: &Path) -> Result<PathBuf> {
        let absolute = repo_root
            .join("tests/ha/givens")
            .join(self.definition().compose_variant_relative_path);
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
        render_ha_member_runtime_config_toml(
            member.service_name(),
            self.local_dcs_service_for(member).client_url(),
            self.replicator_role(),
            self.rewinder_role(),
        )
        .map_err(|err| HarnessError::message(err.to_string()))
    }

    pub fn materialize_fixture(self, repo_root: &Path, materialized_root: &Path) -> Result<()> {
        let shared_root = repo_root.join("tests/ha/givens/three_node_shared");
        for relative_path in self.definition().shared_fixture_relative_paths {
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
}

fn shared_dcs_service_for(_: ClusterMember) -> DcsService {
    DcsService::SharedEtcd
}

fn member_local_dcs_service_for(member: ClusterMember) -> DcsService {
    member.local_dcs_service()
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
    use pgtuskmaster_test_support::config_v2::validate_runtime_document_contents;

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
                r#"endpoints = ["http://etcd:2379"]"#,
                r#"username = "replicator""#,
                r#"username = "rewinder""#,
            ),
            (
                HaGivenId::CustomRoles,
                ClusterMember::NodeB,
                r#"endpoints = ["http://etcd:2379"]"#,
                r#"username = "mirrorbot""#,
                r#"username = "rewindbot""#,
            ),
            (
                HaGivenId::ThreeEtcd,
                ClusterMember::NodeC,
                r#"endpoints = ["http://etcd-c:2379"]"#,
                r#"username = "replicator""#,
                r#"username = "rewinder""#,
            ),
        ] {
            let rendered = given.render_runtime_config(member)?;
            validate_runtime_document_contents(rendered.as_str())
                .map_err(|source| HarnessError::message(source.to_string()))?;
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
                r#"endpoints = ["http://etcd:2379"]"#,
            ),
            (
                "custom-roles",
                HaGivenId::CustomRoles,
                ClusterMember::NodeB,
                r#"username = "mirrorbot""#,
                r#"username = "rewindbot""#,
                r#"endpoints = ["http://etcd:2379"]"#,
            ),
            (
                "three-etcd",
                HaGivenId::ThreeEtcd,
                ClusterMember::NodeA,
                r#"username = "replicator""#,
                r#"username = "rewinder""#,
                r#"endpoints = ["http://etcd-a:2379"]"#,
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
                validate_runtime_document_contents(runtime.as_str()).map_err(|source| {
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
