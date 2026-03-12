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
    config_revision: Version,
    config_sig: String,
    pg_version: Version,
    pg_sig: String,
    dcs_version: Version,
    dcs_sig: String,
    process_version: Version,
    process_sig: String,
    ha_version: Version,
    ha_sig: String,
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

    let config_summary = summarize_config(&snapshot_ctx.config.value);
    let pg_summary = summarize_pg(&snapshot_ctx.pg.value);
    let dcs_summary = summarize_dcs(&snapshot_ctx.dcs.value);
    let process_summary = summarize_process(&snapshot_ctx.process.value);
    let ha_summary = summarize_ha(&snapshot_ctx.ha.value);
    let ha_sig = ha_signature(&snapshot_ctx.ha.value);

    let observed = DebugObservedState {
        app: snapshot_ctx.app.clone(),
        config_revision: snapshot_ctx.config.version,
        config_sig: config_summary.clone(),
        pg_version: snapshot_ctx.pg.version,
        pg_sig: pg_summary.clone(),
        dcs_version: snapshot_ctx.dcs.version,
        dcs_sig: dcs_summary.clone(),
        process_version: snapshot_ctx.process.version,
        process_sig: process_summary.clone(),
        ha_version: snapshot_ctx.ha.version,
        ha_sig,
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
        if previous.config_sig != observed.config_sig {
            ctx.record_change(
                now,
                DebugDomain::Config,
                Some(previous.config_revision),
                Some(observed.config_revision),
                config_summary.clone(),
            )?;
        }
        if previous.pg_sig != observed.pg_sig {
            ctx.record_change(
                now,
                DebugDomain::PgInfo,
                Some(previous.pg_version),
                Some(observed.pg_version),
                pg_summary.clone(),
            )?;
        }
        if previous.dcs_sig != observed.dcs_sig {
            ctx.record_change(
                now,
                DebugDomain::Dcs,
                Some(previous.dcs_version),
                Some(observed.dcs_version),
                dcs_summary.clone(),
            )?;
        }
        if previous.process_sig != observed.process_sig {
            ctx.record_change(
                now,
                DebugDomain::Process,
                Some(previous.process_version),
                Some(observed.process_version),
                process_summary.clone(),
            )?;
        }
        if previous.ha_sig != observed.ha_sig {
            ctx.record_change(
                now,
                DebugDomain::Ha,
                Some(previous.ha_version),
                Some(observed.ha_version),
                ha_summary.clone(),
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
            Some(observed.config_revision),
            config_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::PgInfo,
            None,
            Some(observed.pg_version),
            pg_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Dcs,
            None,
            Some(observed.dcs_version),
            dcs_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Process,
            None,
            Some(observed.process_version),
            process_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Ha,
            None,
            Some(observed.ha_version),
            ha_summary,
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
        config.api.security.tls.mode != crate::config::ApiTlsMode::Disabled
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
        state.cache.member_slots.len(),
        state
            .cache
            .leader_lease
            .as_ref()
            .map(|leader| leader.holder.0.clone())
            .unwrap_or_else(|| "none".to_string()),
        state.cache.switchover_intent.is_some()
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
        "ha worker={:?} role={} tick={} authority={} detail={:?} planned_commands={}",
        state.worker,
        state.role.label(),
        state.tick,
        authority_label(&state.publication.authority),
        state.role,
        state.planned_commands.len()
    )
}

fn ha_signature(state: &HaState) -> String {
    format!(
        "ha worker={:?} role={} authority={} detail={:?}",
        state.worker,
        state.role.label(),
        authority_label(&state.publication.authority),
        state.role
    )
}

fn authority_label(value: &crate::ha::types::ProjectedAuthority) -> String {
    match value {
        crate::ha::types::ProjectedAuthority::Primary { member, epoch } => {
            format!("primary:{}#{}", member.0, epoch.generation)
        }
        crate::ha::types::ProjectedAuthority::NoPrimary(reason) => {
            format!("no_primary:{reason:?}")
        }
        crate::ha::types::ProjectedAuthority::Unknown => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::{ApiTlsMode, RuntimeConfig},
        dcs::state::{DcsCache, DcsState, DcsTrust},
        debug_api::snapshot::{AppLifecycle, DebugDomain, SystemSnapshot},
        ha::state::HaState,
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerError, WorkerStatus},
    };

    use super::{DebugApiContractStubInputs, DebugApiCtx};

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
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
                member_slots: BTreeMap::new(),
                leader_lease: None,
                switchover_intent: None,
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
        HaState::initial(WorkerStatus::Starting)
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
        updated_cfg.api.security.tls.mode = ApiTlsMode::Required;
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
    async fn step_once_does_not_record_ha_tick_only_changes(
    ) -> Result<(), crate::state::WorkerError> {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));

        let initial_ha = sample_ha_state();
        let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha.clone(), UnixMillis(1));

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
            ha_subscriber: ha_subscriber.clone(),
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let before = subscriber.latest();
        let before_timeline_len = before.value.timeline.len();
        let before_sequence = before.value.sequence;

        let mut ha_bumped_tick = initial_ha.clone();
        ha_bumped_tick.tick = ha_bumped_tick.tick.saturating_add(1);
        ha_publisher
            .publish(ha_bumped_tick.clone(), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;

        super::step_once(&mut ctx).await?;
        let after = subscriber.latest();
        assert_eq!(after.value.timeline.len(), before_timeline_len);
        assert_eq!(after.value.sequence, before_sequence);
        assert_eq!(after.value.ha.value.tick, ha_bumped_tick.tick);
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
