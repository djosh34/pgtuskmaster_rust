use crate::support::topology::{ClusterMember, DcsMember};

pub const DATABASE_MEMBERS: [ClusterMember; 3] = ClusterMember::ALL;
pub const DCS_MEMBERS: [DcsMember; 3] = DcsMember::ALL;
pub const OBSERVER_SERVICE_NAME: &str = "observer";
pub const IPTABLES_CHAIN: &str = "PGTM_HA_FAULTS";
pub const FAULT_DIR: &str = "/var/lib/pgtuskmaster/faults";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrafficPath {
    Dcs,
    Api,
    Postgres,
}

impl TrafficPath {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dcs => "dcs",
            Self::Api => "api",
            Self::Postgres => "postgres",
        }
    }

    pub fn port(self) -> u16 {
        match self {
            Self::Dcs => 2379,
            Self::Api => 8443,
            Self::Postgres => 5432,
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
    pub fn label(self) -> &'static str {
        match self {
            Self::PgBasebackup => "pg_basebackup",
            Self::PgRewind => "pg_rewind",
            Self::PostgresStart => "postgres_start",
        }
    }

    pub fn marker_path(self) -> &'static str {
        match self {
            Self::PgBasebackup => "/var/lib/pgtuskmaster/faults/block-pg-basebackup",
            Self::PgRewind => "/var/lib/pgtuskmaster/faults/fail-pg-rewind",
            Self::PostgresStart => "/var/lib/pgtuskmaster/faults/fail-postgres-start",
        }
    }

    pub fn clear_on_start_marker_path(self) -> &'static str {
        match self {
            Self::PgBasebackup => "/var/lib/pgtuskmaster/faults/clear-block-pg-basebackup-on-start",
            Self::PgRewind => "/var/lib/pgtuskmaster/faults/clear-fail-pg-rewind-on-start",
            Self::PostgresStart => {
                "/var/lib/pgtuskmaster/faults/clear-fail-postgres-start-on-start"
            }
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

pub fn signal_named_process_script(signal: &str, process_name: &str) -> String {
    let quoted_signal = shell_quote(signal);
    let quoted_name = shell_quote(process_name);
    format!(
        "pkill --signal {signal} --exact {process_name}",
        signal = quoted_signal,
        process_name = quoted_name
    )
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::{append_fault_rule_script, remove_fault_rule_script};

    #[test]
    fn remove_fault_rule_script_targets_all_rule_shapes_for_peer_and_port() {
        let script = remove_fault_rule_script("10.0.0.7", 5432);

        for fragment in [
            "-d '10.0.0.7' --dport 5432 -j REJECT",
            "-d '10.0.0.7' --sport 5432 -j REJECT",
            "-s '10.0.0.7' --dport 5432 -j REJECT",
            "-s '10.0.0.7' --sport 5432 -j REJECT",
        ] {
            assert!(
                script.contains(fragment),
                "expected remove script to contain `{fragment}`, got: {script}"
            );
        }
    }

    #[test]
    fn append_and_remove_scripts_quote_peer_hosts_consistently() {
        let append = append_fault_rule_script("node-a.example", 8443);
        let remove = remove_fault_rule_script("node-a.example", 8443);

        for script in [append, remove] {
            assert!(
                script.contains("'node-a.example'"),
                "expected quoted peer host in script: {script}"
            );
        }
    }
}
