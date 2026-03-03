use std::{collections::VecDeque, time::Duration};

use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    debug_api::snapshot::{
        build_snapshot, AppLifecycle, DebugChangeEvent, DebugDomain, DebugSnapshotCtx,
        DebugTimelineEntry, SystemSnapshot,
    },
    ha::state::HaState,
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{StatePublisher, StateSubscriber, UnixMillis, Version, WorkerError},
};

const DEFAULT_HISTORY_LIMIT: usize = 300;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DebugObservedState {
    app: AppLifecycle,
    config_version: Version,
    pg_version: Version,
    dcs_version: Version,
    process_version: Version,
    ha_version: Version,
}

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
    pub(crate) history_limit: usize,
    sequence: u64,
    last_observed: Option<DebugObservedState>,
    changes: VecDeque<DebugChangeEvent>,
    timeline: VecDeque<DebugTimelineEntry>,
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
            history_limit: DEFAULT_HISTORY_LIMIT,
            sequence: 0,
            last_observed: None,
            changes: VecDeque::new(),
            timeline: VecDeque::new(),
        }
    }

    fn next_sequence(&mut self) -> Result<u64, WorkerError> {
        let next = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| WorkerError::Message("debug_api sequence overflow".to_string()))?;
        self.sequence = next;
        Ok(next)
    }

    fn trim_history(&mut self) {
        while self.changes.len() > self.history_limit {
            let _ = self.changes.pop_front();
        }
        while self.timeline.len() > self.history_limit {
            let _ = self.timeline.pop_front();
        }
    }

    fn record_change(
        &mut self,
        now: UnixMillis,
        domain: DebugDomain,
        previous_version: Option<Version>,
        current_version: Option<Version>,
        summary: String,
    ) -> Result<(), WorkerError> {
        let sequence = self.next_sequence()?;
        self.changes.push_back(DebugChangeEvent {
            sequence,
            at: now,
            domain: domain.clone(),
            previous_version,
            current_version,
            summary: summary.clone(),
        });
        self.timeline.push_back(DebugTimelineEntry {
            sequence,
            at: now,
            domain,
            message: summary,
        });
        self.trim_history();
        Ok(())
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

    let observed = DebugObservedState {
        app: snapshot_ctx.app.clone(),
        config_version: snapshot_ctx.config.version,
        pg_version: snapshot_ctx.pg.version,
        dcs_version: snapshot_ctx.dcs.version,
        process_version: snapshot_ctx.process.version,
        ha_version: snapshot_ctx.ha.version,
    };

    if let Some(previous) = ctx.last_observed.clone() {
        if previous.app != observed.app {
            ctx.record_change(
                now,
                DebugDomain::App,
                None,
                None,
                summarize_app(&observed.app),
            )?;
        }
        if previous.config_version != observed.config_version {
            ctx.record_change(
                now,
                DebugDomain::Config,
                Some(previous.config_version),
                Some(observed.config_version),
                summarize_config(&snapshot_ctx.config.value),
            )?;
        }
        if previous.pg_version != observed.pg_version {
            ctx.record_change(
                now,
                DebugDomain::PgInfo,
                Some(previous.pg_version),
                Some(observed.pg_version),
                summarize_pg(&snapshot_ctx.pg.value),
            )?;
        }
        if previous.dcs_version != observed.dcs_version {
            ctx.record_change(
                now,
                DebugDomain::Dcs,
                Some(previous.dcs_version),
                Some(observed.dcs_version),
                summarize_dcs(&snapshot_ctx.dcs.value),
            )?;
        }
        if previous.process_version != observed.process_version {
            ctx.record_change(
                now,
                DebugDomain::Process,
                Some(previous.process_version),
                Some(observed.process_version),
                summarize_process(&snapshot_ctx.process.value),
            )?;
        }
        if previous.ha_version != observed.ha_version {
            ctx.record_change(
                now,
                DebugDomain::Ha,
                Some(previous.ha_version),
                Some(observed.ha_version),
                summarize_ha(&snapshot_ctx.ha.value),
            )?;
        }
    } else {
        ctx.record_change(
            now,
            DebugDomain::App,
            None,
            None,
            summarize_app(&observed.app),
        )?;
        ctx.record_change(
            now,
            DebugDomain::Config,
            None,
            Some(observed.config_version),
            summarize_config(&snapshot_ctx.config.value),
        )?;
        ctx.record_change(
            now,
            DebugDomain::PgInfo,
            None,
            Some(observed.pg_version),
            summarize_pg(&snapshot_ctx.pg.value),
        )?;
        ctx.record_change(
            now,
            DebugDomain::Dcs,
            None,
            Some(observed.dcs_version),
            summarize_dcs(&snapshot_ctx.dcs.value),
        )?;
        ctx.record_change(
            now,
            DebugDomain::Process,
            None,
            Some(observed.process_version),
            summarize_process(&snapshot_ctx.process.value),
        )?;
        ctx.record_change(
            now,
            DebugDomain::Ha,
            None,
            Some(observed.ha_version),
            summarize_ha(&snapshot_ctx.ha.value),
        )?;
    }

    ctx.last_observed = Some(observed);

    let changes = ctx.changes.iter().cloned().collect::<Vec<_>>();
    let timeline = ctx.timeline.iter().cloned().collect::<Vec<_>>();
    let snapshot = build_snapshot(&snapshot_ctx, now, ctx.sequence, &changes, &timeline);

    ctx.publisher
        .publish(snapshot, now)
        .map_err(|err| WorkerError::Message(format!("debug_api publish failed: {err}")))?;
    Ok(())
}

fn summarize_app(app: &AppLifecycle) -> String {
    format!("app={app:?}")
}

fn summarize_config(config: &RuntimeConfig) -> String {
    format!(
        "cluster={} member={} scope={} debug_enabled={} tls_enabled={}",
        config.cluster.name,
        config.cluster.member_id,
        config.dcs.scope,
        config.debug.enabled,
        config.security.tls_enabled
    )
}

fn summarize_pg(state: &PgInfoState) -> String {
    match state {
        PgInfoState::Unknown { common } => {
            format!(
                "pg=unknown worker={:?} sql={:?} readiness={:?}",
                common.worker, common.sql, common.readiness
            )
        }
        PgInfoState::Primary {
            common,
            wal_lsn,
            slots,
        } => {
            format!(
                "pg=primary worker={:?} wal_lsn={} slots={}",
                common.worker,
                wal_lsn.0,
                slots.len()
            )
        }
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => {
            format!(
                "pg=replica worker={:?} replay_lsn={} follow_lsn={} upstream={}",
                common.worker,
                replay_lsn.0,
                follow_lsn
                    .map(|value| value.0)
                    .map_or_else(|| "none".to_string(), |value| value.to_string()),
                upstream
                    .as_ref()
                    .map(|value| value.member_id.0.clone())
                    .unwrap_or_else(|| "none".to_string())
            )
        }
    }
}

fn summarize_dcs(state: &DcsState) -> String {
    format!(
        "dcs worker={:?} trust={:?} members={} leader={} switchover={}",
        state.worker,
        state.trust,
        state.cache.members.len(),
        state
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone())
            .unwrap_or_else(|| "none".to_string()),
        state.cache.switchover.is_some()
    )
}

fn summarize_process(state: &ProcessState) -> String {
    match state {
        ProcessState::Idle {
            worker,
            last_outcome,
        } => {
            format!("process=idle worker={worker:?} last_outcome={last_outcome:?}")
        }
        ProcessState::Running { worker, active } => {
            format!(
                "process=running worker={worker:?} job_id={} kind={:?}",
                active.id.0, active.kind
            )
        }
    }
}

fn summarize_ha(state: &HaState) -> String {
    format!(
        "ha worker={:?} phase={:?} tick={} pending={} recent_action_ids={}",
        state.worker,
        state.phase,
        state.tick,
        state.pending.len(),
        state.recent_action_ids.len()
    )
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::{
        config::{schema::ClusterConfig, RuntimeConfig},
        dcs::state::{DcsCache, DcsState, DcsTrust},
        debug_api::snapshot::{AppLifecycle, DebugDomain, SystemSnapshot},
        ha::actions::HaAction,
        ha::state::{HaPhase, HaState},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerError, WorkerStatus},
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
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
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
        assert_eq!(latest.value.sequence, 6);
        assert_eq!(latest.value.changes.len(), 6);
        assert_eq!(latest.value.timeline.len(), 6);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_keeps_history_when_versions_unchanged(
    ) -> Result<(), crate::state::WorkerError> {
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
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ticks = vec![UnixMillis(2), UnixMillis(3)].into_iter();
        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let first = subscriber.latest();
        super::step_once(&mut ctx).await?;
        let second = subscriber.latest();

        assert_eq!(first.value.sequence, second.value.sequence);
        assert_eq!(first.value.changes.len(), second.value.changes.len());
        assert_eq!(first.value.timeline.len(), second.value.timeline.len());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_records_incremental_version_changes() -> Result<(), crate::state::WorkerError>
    {
        let cfg = sample_runtime_config();
        let (cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
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
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ticks = vec![UnixMillis(2), UnixMillis(4)].into_iter();
        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let before = subscriber.latest().value.sequence;

        let mut updated_cfg = cfg.clone();
        updated_cfg.security.tls_enabled = true;
        cfg_publisher
            .publish(updated_cfg, UnixMillis(3))
            .map_err(|err| WorkerError::Message(format!("cfg publish failed: {err}")))?;

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert!(latest.value.sequence > before);

        let config_events = latest
            .value
            .changes
            .iter()
            .filter(|event| matches!(event.domain, DebugDomain::Config))
            .collect::<Vec<_>>();
        assert!(!config_events.is_empty());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_history_retention_trims_old_entries() -> Result<(), crate::state::WorkerError>
    {
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
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
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
        ctx.history_limit = 3;
        ctx.now = Box::new(|| Ok(UnixMillis(2)));

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert_eq!(latest.value.changes.len(), 3);
        assert_eq!(latest.value.timeline.len(), 3);
        Ok(())
    }
}
