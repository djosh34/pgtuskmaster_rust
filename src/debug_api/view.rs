use serde::Serialize;

use crate::{
    config::RuntimeConfig,
    dcs::state::{DcsState, DcsTrust},
    debug_api::snapshot::{DebugChangeEvent, DebugDomain, DebugTimelineEntry, SystemSnapshot},
    ha::state::HaState,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::state::{JobOutcome, ProcessState},
    state::{Versioned, WorkerStatus},
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugVerbosePayload {
    pub(crate) meta: DebugMeta,
    pub(crate) config: ConfigSection,
    pub(crate) pginfo: PgInfoSection,
    pub(crate) dcs: DcsSection,
    pub(crate) process: ProcessSection,
    pub(crate) ha: HaSection,
    pub(crate) api: ApiSection,
    pub(crate) debug: DebugSection,
    pub(crate) changes: Vec<DebugChangeView>,
    pub(crate) timeline: Vec<DebugTimelineView>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugMeta {
    pub(crate) schema_version: &'static str,
    pub(crate) generated_at_ms: u64,
    pub(crate) channel_updated_at_ms: u64,
    pub(crate) channel_version: u64,
    pub(crate) app_lifecycle: String,
    pub(crate) sequence: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConfigSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) cluster_name: String,
    pub(crate) member_id: String,
    pub(crate) scope: String,
    pub(crate) debug_enabled: bool,
    pub(crate) tls_enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PgInfoSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) variant: &'static str,
    pub(crate) worker: String,
    pub(crate) sql: String,
    pub(crate) readiness: String,
    pub(crate) timeline: Option<u64>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DcsSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) trust: String,
    pub(crate) member_count: usize,
    pub(crate) leader: Option<String>,
    pub(crate) has_switchover_request: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProcessSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) state: &'static str,
    pub(crate) running_job_id: Option<String>,
    pub(crate) last_outcome: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct HaSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) phase: String,
    pub(crate) tick: u64,
    pub(crate) pending_actions: usize,
    pub(crate) recent_action_ids: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ApiSection {
    pub(crate) endpoints: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugSection {
    pub(crate) history_changes: usize,
    pub(crate) history_timeline: usize,
    pub(crate) last_sequence: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugChangeView {
    pub(crate) sequence: u64,
    pub(crate) at_ms: u64,
    pub(crate) domain: String,
    pub(crate) previous_version: Option<u64>,
    pub(crate) current_version: Option<u64>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugTimelineView {
    pub(crate) sequence: u64,
    pub(crate) at_ms: u64,
    pub(crate) category: String,
    pub(crate) message: String,
}

pub(crate) fn build_verbose_payload(
    snapshot: &Versioned<SystemSnapshot>,
    since_sequence: Option<u64>,
) -> DebugVerbosePayload {
    let cutoff = since_sequence.unwrap_or(0);
    let filtered_changes = snapshot
        .value
        .changes
        .iter()
        .filter(|event| event.sequence > cutoff)
        .map(to_change_view)
        .collect::<Vec<_>>();
    let filtered_timeline = snapshot
        .value
        .timeline
        .iter()
        .filter(|entry| entry.sequence > cutoff)
        .map(to_timeline_view)
        .collect::<Vec<_>>();

    let cfg = &snapshot.value.config;
    let pg = &snapshot.value.pg;
    let dcs = &snapshot.value.dcs;
    let process = &snapshot.value.process;
    let ha = &snapshot.value.ha;

    DebugVerbosePayload {
        meta: DebugMeta {
            schema_version: "v1",
            generated_at_ms: snapshot.value.generated_at.0,
            channel_updated_at_ms: snapshot.updated_at.0,
            channel_version: snapshot.version.0,
            app_lifecycle: format!("{:?}", snapshot.value.app),
            sequence: snapshot.value.sequence,
        },
        config: to_config_section(cfg),
        pginfo: to_pg_section(pg),
        dcs: to_dcs_section(dcs),
        process: to_process_section(process),
        ha: to_ha_section(ha),
        api: ApiSection {
            endpoints: vec![
                "/debug/snapshot",
                "/debug/verbose",
                "/debug/ui",
                "/fallback/cluster",
                "/switchover",
                "/ha/state",
                "/ha/switchover",
            ],
        },
        debug: DebugSection {
            history_changes: snapshot.value.changes.len(),
            history_timeline: snapshot.value.timeline.len(),
            last_sequence: snapshot.value.sequence,
        },
        changes: filtered_changes,
        timeline: filtered_timeline,
    }
}

fn to_config_section(cfg: &Versioned<RuntimeConfig>) -> ConfigSection {
    ConfigSection {
        version: cfg.version.0,
        updated_at_ms: cfg.updated_at.0,
        cluster_name: cfg.value.cluster.name.clone(),
        member_id: cfg.value.cluster.member_id.clone(),
        scope: cfg.value.dcs.scope.clone(),
        debug_enabled: cfg.value.debug.enabled,
        tls_enabled: cfg.value.api.security.tls.mode != crate::config::ApiTlsMode::Disabled,
    }
}

fn to_pg_section(pg: &Versioned<PgInfoState>) -> PgInfoSection {
    match &pg.value {
        PgInfoState::Unknown { common } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Unknown",
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "unknown worker={} sql={} readiness={}",
                worker_status_label(&common.worker),
                sql_label(&common.sql),
                readiness_label(&common.readiness)
            ),
        },
        PgInfoState::Primary {
            common,
            wal_lsn,
            slots,
        } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Primary",
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "primary wal_lsn={} slots={} readiness={}",
                wal_lsn.0,
                slots.len(),
                readiness_label(&common.readiness)
            ),
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Replica",
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "replica replay_lsn={} follow_lsn={} upstream={}",
                replay_lsn.0,
                follow_lsn
                    .map(|value| value.0)
                    .map_or_else(|| "none".to_string(), |value| value.to_string()),
                upstream
                    .as_ref()
                    .map(|value| value.member_id.0.clone())
                    .unwrap_or_else(|| "none".to_string())
            ),
        },
    }
}

fn to_dcs_section(dcs: &Versioned<DcsState>) -> DcsSection {
    DcsSection {
        version: dcs.version.0,
        updated_at_ms: dcs.updated_at.0,
        worker: worker_status_label(&dcs.value.worker),
        trust: dcs_trust_label(&dcs.value.trust),
        member_count: dcs.value.cache.members.len(),
        leader: dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone()),
        has_switchover_request: dcs.value.cache.switchover.is_some(),
    }
}

fn to_process_section(process: &Versioned<ProcessState>) -> ProcessSection {
    match &process.value {
        ProcessState::Idle {
            worker,
            last_outcome,
        } => ProcessSection {
            version: process.version.0,
            updated_at_ms: process.updated_at.0,
            worker: worker_status_label(worker),
            state: "Idle",
            running_job_id: None,
            last_outcome: last_outcome.as_ref().map(job_outcome_label),
        },
        ProcessState::Running { worker, active } => ProcessSection {
            version: process.version.0,
            updated_at_ms: process.updated_at.0,
            worker: worker_status_label(worker),
            state: "Running",
            running_job_id: Some(active.id.0.clone()),
            last_outcome: None,
        },
    }
}

fn to_ha_section(ha: &Versioned<HaState>) -> HaSection {
    HaSection {
        version: ha.version.0,
        updated_at_ms: ha.updated_at.0,
        worker: worker_status_label(&ha.value.worker),
        phase: format!("{:?}", ha.value.phase),
        tick: ha.value.tick,
        pending_actions: ha.value.pending.len(),
        recent_action_ids: ha.value.recent_action_ids.len(),
    }
}

fn to_change_view(event: &DebugChangeEvent) -> DebugChangeView {
    DebugChangeView {
        sequence: event.sequence,
        at_ms: event.at.0,
        domain: debug_domain_label(&event.domain).to_string(),
        previous_version: event.previous_version.map(|value| value.0),
        current_version: event.current_version.map(|value| value.0),
        summary: event.summary.clone(),
    }
}

fn to_timeline_view(entry: &DebugTimelineEntry) -> DebugTimelineView {
    DebugTimelineView {
        sequence: entry.sequence,
        at_ms: entry.at.0,
        category: debug_domain_label(&entry.domain).to_string(),
        message: entry.message.clone(),
    }
}

fn worker_status_label(status: &WorkerStatus) -> String {
    match status {
        WorkerStatus::Starting => "Starting".to_string(),
        WorkerStatus::Running => "Running".to_string(),
        WorkerStatus::Stopping => "Stopping".to_string(),
        WorkerStatus::Stopped => "Stopped".to_string(),
        WorkerStatus::Faulted(error) => format!("Faulted({error})"),
    }
}

fn sql_label(status: &SqlStatus) -> String {
    match status {
        SqlStatus::Unknown => "Unknown".to_string(),
        SqlStatus::Healthy => "Healthy".to_string(),
        SqlStatus::Unreachable => "Unreachable".to_string(),
    }
}

fn readiness_label(readiness: &Readiness) -> String {
    match readiness {
        Readiness::Unknown => "Unknown".to_string(),
        Readiness::Ready => "Ready".to_string(),
        Readiness::NotReady => "NotReady".to_string(),
    }
}

fn dcs_trust_label(trust: &DcsTrust) -> String {
    match trust {
        DcsTrust::FullQuorum => "FullQuorum".to_string(),
        DcsTrust::FailSafe => "FailSafe".to_string(),
        DcsTrust::NotTrusted => "NotTrusted".to_string(),
    }
}

fn debug_domain_label(domain: &DebugDomain) -> &'static str {
    match domain {
        DebugDomain::App => "app",
        DebugDomain::Config => "config",
        DebugDomain::PgInfo => "pginfo",
        DebugDomain::Dcs => "dcs",
        DebugDomain::Process => "process",
        DebugDomain::Ha => "ha",
    }
}

fn job_outcome_label(outcome: &JobOutcome) -> String {
    match outcome {
        JobOutcome::Success { id, .. } => format!("Success({})", id.0),
        JobOutcome::Failure { id, error, .. } => format!("Failure({}: {:?})", id.0, error),
        JobOutcome::Timeout { id, .. } => format!("Timeout({})", id.0),
        JobOutcome::Cancelled { id, .. } => format!("Cancelled({})", id.0),
    }
}
