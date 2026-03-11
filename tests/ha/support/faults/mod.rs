pub const ALL_CLUSTER_MEMBERS: [&str; 3] = ["node-a", "node-b", "node-c"];
pub const OBSERVER_SERVICE: &str = "observer";
pub const ETCD_SERVICE: &str = "etcd";
pub const IPTABLES_CHAIN: &str = "PGTM_HA_FAULTS";
pub const FAULT_DIR: &str = "/var/lib/pgtuskmaster/faults";

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
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

pub fn touch_file_script(path: &str) -> String {
    format!(
        "mkdir -p {dir} && : > {path}",
        dir = FAULT_DIR,
        path = shell_quote(path)
    )
}

pub fn remove_file_script(path: &str) -> String {
    format!("rm -f {path}", path = shell_quote(path))
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
