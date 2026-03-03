use std::time::Duration;

use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    debug_api::snapshot::{build_snapshot, AppLifecycle, DebugSnapshotCtx, SystemSnapshot},
    ha::state::HaState,
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{StatePublisher, StateSubscriber, UnixMillis, WorkerError},
};

pub(crate) struct DebugApiCtx {
    pub(crate) app: AppLifecycle,
    pub(crate) publisher: StatePublisher<SystemSnapshot>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) ha_subscriber: StateSubscriber<HaState>,
    pub(crate) poll_interval: Duration,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}

pub(crate) struct DebugApiContractStubInputs {
    pub(crate) publisher: StatePublisher<SystemSnapshot>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) ha_subscriber: StateSubscriber<HaState>,
}

impl DebugApiCtx {
    pub(crate) fn contract_stub(inputs: DebugApiContractStubInputs) -> Self {
        let DebugApiContractStubInputs {
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        } = inputs;

        Self {
            app: AppLifecycle::Starting,
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
            poll_interval: Duration::from_millis(10),
            now: Box::new(|| Ok(UnixMillis(0))),
        }
    }
}

pub(crate) async fn run(mut ctx: DebugApiCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut DebugApiCtx) -> Result<(), WorkerError> {
    let now = (ctx.now)()?;
    let snapshot_ctx = DebugSnapshotCtx {
        app: ctx.app.clone(),
        config: ctx.config_subscriber.latest(),
        pg: ctx.pg_subscriber.latest(),
        dcs: ctx.dcs_subscriber.latest(),
        process: ctx.process_subscriber.latest(),
        ha: ctx.ha_subscriber.latest(),
    };
    let snapshot = build_snapshot(&snapshot_ctx, now);
    ctx.publisher
        .publish(snapshot, now)
        .map_err(|err| WorkerError::Message(format!("debug_api publish failed: {err}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::{
        config::{schema::ClusterConfig, RuntimeConfig},
        dcs::state::{DcsCache, DcsState, DcsTrust},
        debug_api::snapshot::{AppLifecycle, SystemSnapshot},
        ha::actions::HaAction,
        ha::state::{HaPhase, HaState},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerStatus},
    };

    use super::{DebugApiContractStubInputs, DebugApiCtx};

    fn sample_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: crate::config::schema::PostgresConfig {
                data_dir: "/tmp/pgdata".into(),
                connect_timeout_s: 5,
            },
            dcs: crate::config::schema::DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
            },
            ha: crate::config::schema::HaConfig {
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: crate::config::ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                binaries: crate::config::BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            api: crate::config::schema::ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
            },
            debug: crate::config::schema::DebugConfig { enabled: true },
            security: crate::config::schema::SecurityConfig {
                tls_enabled: false,
                auth_token: None,
            },
        }
    }

    fn sample_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Starting,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: None,
            },
        }
    }

    fn sample_dcs_state(cfg: RuntimeConfig) -> DcsState {
        DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config: cfg,
                init_lock: None,
            },
            last_refresh_at: None,
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        }
    }

    fn sample_ha_state() -> HaState {
        HaState {
            worker: WorkerStatus::Starting,
            phase: HaPhase::Init,
            tick: 0,
            pending: vec![HaAction::SignalFailSafe],
            recent_action_ids: BTreeSet::new(),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_publishes_snapshot() -> Result<(), crate::state::WorkerError> {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));

        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
            },
            UnixMillis(1),
        );

        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(|| Ok(UnixMillis(2)));
        ctx.app = AppLifecycle::Running;

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert_eq!(latest.updated_at, UnixMillis(2));
        assert_eq!(latest.value.app, AppLifecycle::Running);
        Ok(())
    }
}
