use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    ha::state::HaState,
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{UnixMillis, Version, Versioned},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AppLifecycle {
    Starting,
    Running,
    Stopping,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SystemSnapshot {
    pub(crate) app: AppLifecycle,
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
    pub(crate) ha: Versioned<HaState>,
    pub(crate) generated_at: UnixMillis,
    pub(crate) sequence: u64,
    pub(crate) changes: Vec<DebugChangeEvent>,
    pub(crate) timeline: Vec<DebugTimelineEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DebugDomain {
    App,
    Config,
    PgInfo,
    Dcs,
    Process,
    Ha,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugChangeEvent {
    pub(crate) sequence: u64,
    pub(crate) at: UnixMillis,
    pub(crate) domain: DebugDomain,
    pub(crate) previous_version: Option<Version>,
    pub(crate) current_version: Option<Version>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugTimelineEntry {
    pub(crate) sequence: u64,
    pub(crate) at: UnixMillis,
    pub(crate) domain: DebugDomain,
    pub(crate) message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugSnapshotCtx {
    pub(crate) app: AppLifecycle,
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
    pub(crate) ha: Versioned<HaState>,
}

pub(crate) fn build_snapshot(
    ctx: &DebugSnapshotCtx,
    now: UnixMillis,
    sequence: u64,
    changes: &[DebugChangeEvent],
    timeline: &[DebugTimelineEntry],
) -> SystemSnapshot {
    SystemSnapshot {
        app: ctx.app.clone(),
        config: ctx.config.clone(),
        pg: ctx.pg.clone(),
        dcs: ctx.dcs.clone(),
        process: ctx.process.clone(),
        ha: ctx.ha.clone(),
        generated_at: now,
        sequence,
        changes: changes.to_vec(),
        timeline: timeline.to_vec(),
    }
}
