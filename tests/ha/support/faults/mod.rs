use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::support::{
    error::{HarnessError, Result},
    files::{create_dir_all, write_text_file},
    topology::{ClusterMember, DcsService},
};

pub const DATABASE_MEMBERS: [ClusterMember; 3] = ClusterMember::ALL;
pub const DCS_SERVICES: [DcsService; 3] = DcsService::COLOCATED_ALL;
pub const IPTABLES_CHAIN: &str = "PGTM_HA_FAULTS";
pub const FAULT_DIR: &str = "/var/lib/pgtuskmaster/faults";
pub const WIPE_DATA_ON_START_MARKER_PATH: &str = "/var/lib/pgtuskmaster/faults/wipe-data-on-start";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrafficPath {
    Dcs,
    Api,
    Postgres,
}

impl TrafficPath {
    pub fn label(self) -> &'static str {
        self.spec().0
    }

    pub fn port(self) -> u16 {
        self.spec().1
    }

    fn spec(self) -> (&'static str, u16) {
        match self {
            Self::Dcs => ("dcs", 2379),
            Self::Api => ("api", 8443),
            Self::Postgres => ("postgres", 5432),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockerKind {
    PgBasebackup,
    PgRewind,
    PostgresStart,
}

impl BlockerKind {
    pub fn parse(raw_value: &str) -> Result<Self> {
        Ok(match raw_value {
            "pg_basebackup" => Self::PgBasebackup,
            "pg_rewind" => Self::PgRewind,
            "postgres_start" => Self::PostgresStart,
            _ => {
                return Err(HarnessError::message(format!(
                    "unsupported blocker `{raw_value}`"
                )))
            }
        })
    }

    pub fn label(self) -> &'static str {
        self.spec().0
    }

    pub fn marker_path(self) -> &'static str {
        self.spec().1
    }

    pub fn clear_on_start_marker_path(self) -> &'static str {
        self.spec().2
    }

    fn spec(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::PgBasebackup => (
                "pg_basebackup",
                "/var/lib/pgtuskmaster/faults/block-pg-basebackup",
                "/var/lib/pgtuskmaster/faults/clear-block-pg-basebackup-on-start",
            ),
            Self::PgRewind => (
                "pg_rewind",
                "/var/lib/pgtuskmaster/faults/fail-pg-rewind",
                "/var/lib/pgtuskmaster/faults/clear-fail-pg-rewind-on-start",
            ),
            Self::PostgresStart => (
                "postgres_start",
                "/var/lib/pgtuskmaster/faults/fail-postgres-start",
                "/var/lib/pgtuskmaster/faults/clear-fail-postgres-start-on-start",
            ),
        }
    }
}

pub fn ensure_fault_plumbing_script() -> String {
    format!(
        "mkdir -p {fault_dir} && \
iptables -w -N {chain} 2>/dev/null || true && \
iptables -w -C OUTPUT -j {chain} 2>/dev/null || iptables -w -I OUTPUT 1 -j {chain} && \
iptables -w -C INPUT -j {chain} 2>/dev/null || iptables -w -I INPUT 1 -j {chain}",
        fault_dir = FAULT_DIR,
        chain = IPTABLES_CHAIN,
    )
}

pub fn clear_fault_rules_script() -> String {
    format!(
        "iptables -w -F {chain} 2>/dev/null || true",
        chain = IPTABLES_CHAIN
    )
}

pub fn append_fault_rule_script(peer_host: &str, port: u16) -> String {
    let peer = shell_quote(peer_host);
    format!(
        "iptables -w -C {chain} -p tcp -d {peer} --dport {port} -j REJECT 2>/dev/null || \
iptables -w -A {chain} -p tcp -d {peer} --dport {port} -j REJECT; \
iptables -w -C {chain} -p tcp -d {peer} --sport {port} -j REJECT 2>/dev/null || \
iptables -w -A {chain} -p tcp -d {peer} --sport {port} -j REJECT; \
iptables -w -C {chain} -p tcp -s {peer} --dport {port} -j REJECT 2>/dev/null || \
iptables -w -A {chain} -p tcp -s {peer} --dport {port} -j REJECT; \
iptables -w -C {chain} -p tcp -s {peer} --sport {port} -j REJECT 2>/dev/null || \
iptables -w -A {chain} -p tcp -s {peer} --sport {port} -j REJECT",
        chain = IPTABLES_CHAIN,
        peer = peer,
        port = port,
    )
}

pub fn remove_fault_rule_script(peer_host: &str, port: u16) -> String {
    let peer = shell_quote(peer_host);
    format!(
        "while iptables -w -D {chain} -p tcp -d {peer} --dport {port} -j REJECT 2>/dev/null; do :; done; \
while iptables -w -D {chain} -p tcp -d {peer} --sport {port} -j REJECT 2>/dev/null; do :; done; \
while iptables -w -D {chain} -p tcp -s {peer} --dport {port} -j REJECT 2>/dev/null; do :; done; \
while iptables -w -D {chain} -p tcp -s {peer} --sport {port} -j REJECT 2>/dev/null; do :; done",
        chain = IPTABLES_CHAIN,
        peer = peer,
        port = port,
    )
}

pub fn write_fault_marker(
    materialized_dir: &Path,
    member: ClusterMember,
    marker_path: &str,
) -> Result<()> {
    let marker_file = materialized_fault_marker_path(materialized_dir, member, marker_path)?;
    if let Some(parent) = marker_file.parent() {
        create_dir_all(parent)?;
    }
    write_text_file(marker_file.as_path(), "")?;
    Ok(())
}

pub fn remove_fault_marker(
    materialized_dir: &Path,
    member: ClusterMember,
    marker_path: &str,
) -> Result<()> {
    let marker_file = materialized_fault_marker_path(materialized_dir, member, marker_path)?;
    match fs::remove_file(marker_file.as_path()) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(HarnessError::Io {
            path: marker_file,
            source,
        }),
    }
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn materialized_fault_dir(materialized_dir: &Path, member: ClusterMember) -> PathBuf {
    materialized_dir.join("faults").join(member.service_name())
}

fn materialized_fault_marker_path(
    materialized_dir: &Path,
    member: ClusterMember,
    marker_path: &str,
) -> Result<PathBuf> {
    let relative_path = Path::new(marker_path)
        .strip_prefix(FAULT_DIR)
        .map_err(|_| {
            HarnessError::message(format!(
                "fault marker `{marker_path}` does not live under `{FAULT_DIR}`"
            ))
        })?;
    Ok(materialized_fault_dir(materialized_dir, member).join(relative_path))
}

#[cfg(test)]
mod tests {
    use crate::support::{error::Result, files::with_temporary_directory, topology::ClusterMember};

    use super::{
        append_fault_rule_script, remove_fault_marker, remove_fault_rule_script,
        write_fault_marker, WIPE_DATA_ON_START_MARKER_PATH,
    };

    #[test]
    fn fault_rule_scripts_quote_peer_hosts_and_cover_remove_shapes() {
        let append = append_fault_rule_script("node-a.example", 8443);
        let remove = remove_fault_rule_script("node-a.example", 8443);
        for fragment in [
            "-d 'node-a.example' --dport 8443 -j REJECT",
            "-d 'node-a.example' --sport 8443 -j REJECT",
            "-s 'node-a.example' --dport 8443 -j REJECT",
            "-s 'node-a.example' --sport 8443 -j REJECT",
        ] {
            assert!(remove.contains(fragment));
        }

        for script in [append, remove] {
            assert!(script.contains("'node-a.example'"));
        }
    }

    #[test]
    fn write_and_remove_fault_marker_manage_member_marker_file() -> Result<()> {
        with_temporary_directory("pgtm-ha-faults", "marker-round-trip", |root| {
            let marker = root.join("faults/node-c/wipe-data-on-start");
            write_fault_marker(root, ClusterMember::NodeC, WIPE_DATA_ON_START_MARKER_PATH)?;
            assert!(marker.is_file());

            remove_fault_marker(root, ClusterMember::NodeC, WIPE_DATA_ON_START_MARKER_PATH)?;
            assert!(!marker.exists());

            remove_fault_marker(root, ClusterMember::NodeC, WIPE_DATA_ON_START_MARKER_PATH)?;
            Ok(())
        })
    }
}
