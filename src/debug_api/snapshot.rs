use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    ha::state::HaState,
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{UnixMillis, Versioned},
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

pub(crate) fn build_snapshot(ctx: &DebugSnapshotCtx, _now: UnixMillis) -> SystemSnapshot {
    SystemSnapshot {
        app: ctx.app.clone(),
        config: ctx.config.clone(),
        pg: ctx.pg.clone(),
        dcs: ctx.dcs.clone(),
        process: ctx.process.clone(),
        ha: ctx.ha.clone(),
    }
}
