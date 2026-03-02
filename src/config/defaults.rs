use super::schema::{
    ApiConfig, DebugConfig, PartialRuntimeConfig, PostgresConfig, ProcessConfig, RuntimeConfig,
    SecurityConfig,
};

const DEFAULT_PG_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_API_LISTEN_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_DEBUG_ENABLED: bool = false;
const DEFAULT_SECURITY_TLS_ENABLED: bool = false;

pub fn apply_defaults(raw: PartialRuntimeConfig) -> RuntimeConfig {
    let postgres = PostgresConfig {
        data_dir: raw.postgres.data_dir,
        connect_timeout_s: raw
            .postgres
            .connect_timeout_s
            .unwrap_or(DEFAULT_PG_CONNECT_TIMEOUT_S),
    };

    let process = ProcessConfig {
        pg_rewind_timeout_ms: raw
            .process
            .pg_rewind_timeout_ms
            .unwrap_or(DEFAULT_PG_REWIND_TIMEOUT_MS),
        bootstrap_timeout_ms: raw
            .process
            .bootstrap_timeout_ms
            .unwrap_or(DEFAULT_BOOTSTRAP_TIMEOUT_MS),
        fencing_timeout_ms: raw
            .process
            .fencing_timeout_ms
            .unwrap_or(DEFAULT_FENCING_TIMEOUT_MS),
        binaries: raw.process.binaries,
    };

    let api = ApiConfig {
        listen_addr: raw
            .api
            .and_then(|cfg| cfg.listen_addr)
            .unwrap_or_else(|| DEFAULT_API_LISTEN_ADDR.to_string()),
    };

    let debug = DebugConfig {
        enabled: raw
            .debug
            .and_then(|cfg| cfg.enabled)
            .unwrap_or(DEFAULT_DEBUG_ENABLED),
    };

    let security = SecurityConfig {
        tls_enabled: raw
            .security
            .as_ref()
            .and_then(|cfg| cfg.tls_enabled)
            .unwrap_or(DEFAULT_SECURITY_TLS_ENABLED),
        auth_token: raw.security.and_then(|cfg| cfg.auth_token),
    };

    RuntimeConfig {
        cluster: raw.cluster,
        postgres,
        dcs: raw.dcs,
        ha: raw.ha,
        process,
        api,
        debug,
        security,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::schema::{
        BinaryPaths, ClusterConfig, DcsConfig, HaConfig, PartialApiConfig, PartialDebugConfig,
        PartialPostgresConfig, PartialProcessConfig, PartialSecurityConfig,
    };

    fn base_partial() -> PartialRuntimeConfig {
        PartialRuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "member-a".to_string(),
            },
            postgres: PartialPostgresConfig {
                data_dir: PathBuf::from("/var/lib/postgresql/data"),
                connect_timeout_s: None,
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "demo".to_string(),
            },
            ha: HaConfig {
                loop_interval_ms: 1_000,
                lease_ttl_ms: 10_000,
            },
            process: PartialProcessConfig {
                pg_rewind_timeout_ms: None,
                bootstrap_timeout_ms: None,
                fencing_timeout_ms: None,
                binaries: BinaryPaths {
                    postgres: PathBuf::from("/usr/bin/postgres"),
                    pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                    pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                    initdb: PathBuf::from("/usr/bin/initdb"),
                    psql: PathBuf::from("/usr/bin/psql"),
                },
            },
            api: None,
            debug: None,
            security: None,
        }
    }

    #[test]
    fn apply_defaults_fills_optional_fields() {
        let cfg = apply_defaults(base_partial());

        assert_eq!(cfg.postgres.connect_timeout_s, DEFAULT_PG_CONNECT_TIMEOUT_S);
        assert_eq!(cfg.process.pg_rewind_timeout_ms, DEFAULT_PG_REWIND_TIMEOUT_MS);
        assert_eq!(cfg.process.bootstrap_timeout_ms, DEFAULT_BOOTSTRAP_TIMEOUT_MS);
        assert_eq!(cfg.process.fencing_timeout_ms, DEFAULT_FENCING_TIMEOUT_MS);
        assert_eq!(cfg.api.listen_addr, DEFAULT_API_LISTEN_ADDR);
        assert!(!cfg.debug.enabled);
        assert!(!cfg.security.tls_enabled);
        assert_eq!(cfg.security.auth_token, None);
    }

    #[test]
    fn apply_defaults_preserves_caller_values() {
        let mut raw = base_partial();
        raw.postgres.connect_timeout_s = Some(42);
        raw.process.pg_rewind_timeout_ms = Some(2_000);
        raw.process.bootstrap_timeout_ms = Some(3_000);
        raw.process.fencing_timeout_ms = Some(4_000);
        raw.api = Some(PartialApiConfig {
            listen_addr: Some("0.0.0.0:9999".to_string()),
        });
        raw.debug = Some(PartialDebugConfig {
            enabled: Some(true),
        });
        raw.security = Some(PartialSecurityConfig {
            tls_enabled: Some(true),
            auth_token: Some("token-123".to_string()),
        });

        let cfg = apply_defaults(raw);

        assert_eq!(cfg.postgres.connect_timeout_s, 42);
        assert_eq!(cfg.process.pg_rewind_timeout_ms, 2_000);
        assert_eq!(cfg.process.bootstrap_timeout_ms, 3_000);
        assert_eq!(cfg.process.fencing_timeout_ms, 4_000);
        assert_eq!(cfg.api.listen_addr, "0.0.0.0:9999");
        assert!(cfg.debug.enabled);
        assert!(cfg.security.tls_enabled);
        assert_eq!(cfg.security.auth_token.as_deref(), Some("token-123"));
    }
}
