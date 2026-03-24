use crate::{
    config_v2::RuntimeConfigV2,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    postgres_roles,
    process::jobs::{PostgresStartIntent, ProcessIntent},
    state::{
        new_state_channel, PgEndpoint, StateSubscriber, UnixMillis, WorkerError, WorkerStatus,
    },
};

use super::{
    decide::decide,
    process_dispatch::dispatch_process_action,
    reconcile::reconcile,
    state::{HaControlPlane, HaObservedState, HaRuntimeCtx, HaState, HaStateChannel},
    types::{
        AuthorityProjection, CandidateState, HaObservation, HaStep, LocalDataState,
        PublicationState, ReadyPrimary,
    },
};

pub(crate) fn bootstrap<'a>(
    cfg: &'a RuntimeConfigV2,
    observed: HaObservedState,
    control: HaControlPlane,
) -> (HaRuntimeCtx<'a>, StateSubscriber<HaState>) {
    bootstrap_with_now(
        cfg,
        observed,
        control,
        Box::new(crate::process::worker::system_now_unix_millis),
    )
}

pub(crate) fn bootstrap_with_now<'a>(
    cfg: &'a RuntimeConfigV2,
    observed: HaObservedState,
    control: HaControlPlane,
    now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
) -> (HaRuntimeCtx<'a>, StateSubscriber<HaState>) {
    let initial_state = HaState::initial(WorkerStatus::Starting);
    let (publisher, state) = new_state_channel(initial_state.clone());

    (
        HaRuntimeCtx {
            cfg,
            now,
            state_channel: HaStateChannel {
                current: initial_state,
                publisher,
            },
            observed,
            control,
        },
        state,
    )
}

pub(crate) async fn run(mut ctx: HaRuntimeCtx<'_>) -> Result<(), WorkerError> {
    let mut interval = tokio::time::interval(ctx.cfg.timing.ha_loop_interval);
    loop {
        tokio::select! {
            changed = ctx.observed.pg.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha pg subscriber closed: {err}")))?;
            }
            changed = ctx.observed.dcs.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha dcs subscriber closed: {err}")))?;
            }
            changed = ctx.observed.process.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha process subscriber closed: {err}")))?;
            }
            _ = interval.tick() => {}
        }
        step_once(&mut ctx).await?;
    }
}

pub(crate) async fn step_once(ctx: &mut HaRuntimeCtx<'_>) -> Result<(), WorkerError> {
    let now = (ctx.now)()?;
    let observation = observe(ctx, now)?;
    let decision = decide(&observation, &ctx.cfg.member_id);
    let steps = reconcile(&observation, &decision);
    let next_publication = match decision.publication.as_ref() {
        Some(projection) => PublicationState::Projected(projection.clone()),
        None => ctx.state_channel.current.publication.clone(),
    };
    let next_managed_roles_reconciled = if steps.iter().any(|step| {
        matches!(
            step,
            HaStep::RunProcess(ProcessIntent::Bootstrap)
                | HaStep::RunProcess(ProcessIntent::ProvisionReplica(_))
                | HaStep::RunProcess(ProcessIntent::Start(PostgresStartIntent::DetachedStandby))
                | HaStep::RunProcess(ProcessIntent::Start(PostgresStartIntent::Replica { .. }))
        )
    }) {
        false
    } else {
        ctx.state_channel.current.managed_roles_reconciled
    };
    let next_state = HaState {
        worker: ctx.state_channel.current.worker.clone(),
        tick: ctx.state_channel.current.tick.saturating_add(1),
        managed_roles_reconciled: next_managed_roles_reconciled,
        publication: next_publication,
        decision: decision.clone(),
        observation: observation.clone(),
        clear_switchover: decision.clear_switchover,
        steps: steps.clone(),
    };

    ctx.state_channel
        .publisher
        .publish(next_state.clone())
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;
    ctx.state_channel.current = next_state;

    for (action_index, step) in steps.iter().enumerate() {
        execute_step(ctx, ctx.state_channel.current.tick, action_index, step).await?;
    }

    Ok(())
}

fn observe(
    ctx: &HaRuntimeCtx<'_>,
    now: crate::state::UnixMillis,
) -> Result<HaObservation, WorkerError> {
    let pg = ctx.observed.pg.latest();
    let dcs = ctx.observed.dcs.latest();
    let process = ctx.observed.process.latest();
    let data_dir_path = ctx.cfg.postgres.data_dir.clone();
    let self_id = &ctx.cfg.member_id;

    let ready_primary = dcs.quorum_state().and_then(|quorum| {
        quorum.members().find_map(|(member_id, member)| {
            ((*member_id != *self_id)
                && matches!(member.postgres(), PgInfoState::Primary { .. })
                && member.postgres().readiness() == Readiness::Ready)
                .then(|| ReadyPrimary {
                    member: member_id.clone(),
                    timeline: member.postgres().timeline().map(|value| u64::from(value.0)),
                    system_identifier: member.postgres().system_identifier().map(|value| value.0),
                })
        })
    });

    let dcs_self_member = if process.basebackup_completed_awaiting_pg_start(&pg) {
        None
    } else {
        dcs.member(self_id)
    };
    let local_timeline = pg.timeline().map(|value| u64::from(value.0)).or_else(|| {
        dcs_self_member
            .and_then(|member| member.postgres().timeline().map(|value| u64::from(value.0)))
    });
    let local_system_identifier = pg.system_identifier().map(|value| value.0).or_else(|| {
        dcs_self_member
            .and_then(|member| member.postgres().system_identifier().map(|value| value.0))
    });

    let local_data = if !data_dir_path.exists() {
        LocalDataState::Missing
    } else if !data_dir_path.join("PG_VERSION").exists() {
        LocalDataState::BootstrapEmpty
    } else {
        match ready_primary.as_ref() {
            Some(primary)
                if primary.system_identifier.is_some()
                    && local_system_identifier.is_some()
                    && local_system_identifier != primary.system_identifier =>
            {
                LocalDataState::DivergedBasebackup
            }
            Some(primary) if primary.timeline == local_timeline => {
                LocalDataState::ConsistentReplica
            }
            Some(primary) if primary.timeline.is_some() && local_timeline.is_some() => {
                LocalDataState::DivergedRewind
            }
            Some(_) if process.rewind_failed_requires_basebackup() => {
                LocalDataState::DivergedBasebackup
            }
            Some(_) | None => LocalDataState::ConsistentReplica,
        }
    };

    let resolved_upstream =
        match &pg {
            PgInfoState::Replica {
                upstream: Some(upstream),
                ..
            } => Some(upstream.member_id.clone()),
            PgInfoState::Replica { common, .. } => common
                .pg_config
                .primary_conninfo
                .as_ref()
                .and_then(|conninfo| match conninfo.route.endpoint() {
                    PgEndpoint::Tcp { host, port } => {
                        dcs.members().find_map(|(member_id, member)| {
                            (member.cluster_postgres_target().host() == host.as_str()
                                && member.cluster_postgres_target().port() == *port)
                                .then_some(member_id.clone())
                        })
                    }
                    PgEndpoint::UnixSocket { .. } => None,
                }),
            PgInfoState::Unknown { .. } | PgInfoState::Primary { .. } => None,
        };

    let self_candidate = match (&local_data, &pg) {
        (LocalDataState::Missing | LocalDataState::BootstrapEmpty, _) => CandidateState::Bootstrap,
        (_, PgInfoState::Primary { common, .. }) if common.sql == SqlStatus::Healthy => pg
            .committed_wal()
            .filter(|position| position.timeline.is_some())
            .map(CandidateState::Promote)
            .unwrap_or(CandidateState::Bootstrap),
        (_, PgInfoState::Replica { common, .. }) if common.sql == SqlStatus::Healthy => pg
            .replay_wal()
            .or_else(|| pg.follow_wal())
            .filter(|position| position.timeline.is_some())
            .map(CandidateState::Promote)
            .unwrap_or(CandidateState::Ineligible),
        (
            _,
            PgInfoState::Unknown { .. } | PgInfoState::Primary { .. } | PgInfoState::Replica { .. },
        ) => CandidateState::Ineligible,
    };

    let storage_stalled = matches!(
        &pg,
        PgInfoState::Primary {
            common: crate::pginfo::state::PgInfoCommon {
                sql: SqlStatus::Healthy,
                ..
            },
            ..
        }
    ) && (dcs.member(self_id).is_none()
        || pg.last_refresh_at().is_none_or(|last_refresh_at| {
            now.0.saturating_sub(last_refresh_at.0) > lease_ttl_ms(ctx.cfg)
        }));

    Ok(HaObservation {
        pg,
        process,
        dcs,
        publication: ctx.state_channel.current.publication.clone(),
        managed_roles_reconciled: ctx.state_channel.current.managed_roles_reconciled,
        local_data,
        resolved_upstream,
        self_candidate,
        storage_stalled,
        ready_primary,
    })
}

async fn execute_step(
    ctx: &mut HaRuntimeCtx<'_>,
    ha_tick: u64,
    action_index: usize,
    step: &HaStep,
) -> Result<(), WorkerError> {
    match step {
        HaStep::Publish(AuthorityProjection::Primary(_))
        | HaStep::Publish(AuthorityProjection::NoPrimary(_)) => Ok(()),
        HaStep::AcquireLease(_) => ctx.control.dcs_handle.acquire_leadership().map_err(|err| {
            WorkerError::Message(format!(
                "ha acquire lease failed at tick {ha_tick} index {action_index}: {err}"
            ))
        }),
        HaStep::ReleaseLease => ctx.control.dcs_handle.release_leadership().map_err(|err| {
            WorkerError::Message(format!(
                "ha release lease failed at tick {ha_tick} index {action_index}: {err}"
            ))
        }),
        HaStep::ClearSwitchover => ctx.control.dcs_handle.clear_switchover().map_err(|err| {
            WorkerError::Message(format!(
                "ha clear switchover failed at tick {ha_tick} index {action_index}: {err}"
            ))
        }),
        HaStep::ReconcileManagedRoles => {
            postgres_roles::reconcile_managed_roles_v2(ctx.cfg)
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha reconcile managed roles failed at tick {ha_tick} index {action_index}: {err}"
                ))
            })?;
            ctx.state_channel.current.managed_roles_reconciled = true;
            Ok(())
        }
        HaStep::RunProcess(intent) => dispatch_process_action(ctx, ha_tick, action_index, intent)
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha process dispatch failed at tick {ha_tick} index {action_index}: {err}"
                ))
            }),
    }
}

fn lease_ttl_ms(cfg: &crate::config_v2::RuntimeConfigV2) -> u64 {
    u64::try_from(cfg.timing.ha_lease_ttl.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config_v2::runtime_test_config_with_data_dir,
        dcs::{DcsHandle, DcsMemberState, DcsSnapshot},
        ha::state::{HaControlPlane, HaObservedState},
        pginfo::conninfo::PgClientTls,
        pginfo::state::{PgConfig, PgConnInfo, PgInfoCommon, Readiness},
        process::state::ProcessState,
    };
    use crate::{
        pginfo::state::SqlStatus,
        state::{
            new_state_channel, MemberId, PgRoute, SwitchoverState, SystemIdentifier, TimelineId,
            UnixMillis, WalLsn, WorkerStatus,
        },
    };

    use super::*;

    fn replica_pg_state(replay_lsn: u64, follow_lsn: Option<u64>) -> PgInfoState {
        PgInfoState::Replica {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: Some(TimelineId(7)),
                system_identifier: Some(SystemIdentifier(41)),
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: std::collections::BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(123)),
            },
            replay_lsn: WalLsn(replay_lsn),
            follow_lsn: follow_lsn.map(WalLsn),
            upstream: None,
        }
    }

    fn replica_pg_state_with_primary_conninfo(
        host: &str,
        port: u16,
    ) -> Result<PgInfoState, String> {
        let mut state = replica_pg_state(67_272_104, Some(67_272_104));
        if let PgInfoState::Replica { common, .. } = &mut state {
            common.pg_config.primary_conninfo = Some(PgConnInfo {
                route: PgRoute::tcp(host.to_string(), port)?,
                user: "replicator".to_string(),
                dbname: "postgres".to_string(),
                application_name: Some("node-a".to_string()),
                connect_timeout_s: Some(5),
                options: None,
                tls: PgClientTls {
                    mode: crate::pginfo::state::PgSslMode::Require,
                    root_cert: None,
                    client_cert: None,
                    client_key: None,
                },
            });
        }
        Ok(state)
    }

    fn dcs_view_for_member(member_id: &str, host: &str, port: u16) -> Result<DcsSnapshot, String> {
        Ok(DcsSnapshot::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(
                MemberId(member_id.to_string()),
                DcsMemberState {
                    cluster_postgres: PgRoute::tcp(host.to_string(), port)?,
                    operator_postgres: None,
                    operator_api: None,
                    postgres: PgInfoState::Unknown {
                        common: PgInfoCommon {
                            worker: WorkerStatus::Running,
                            sql: SqlStatus::Healthy,
                            readiness: Readiness::Ready,
                            timeline: Some(TimelineId(7)),
                            system_identifier: Some(SystemIdentifier(41)),
                            pg_config: PgConfig {
                                port: None,
                                hot_standby: None,
                                primary_conninfo: None,
                                primary_slot_name: None,
                                extra: BTreeMap::new(),
                            },
                            last_refresh_at: Some(UnixMillis(123)),
                        },
                    },
                },
            )]),
        ))
    }

    #[test]
    fn observe_resolves_replica_upstream_from_primary_conninfo() -> Result<(), String> {
        let data_dir =
            std::env::temp_dir().join(format!("pgtm-ha-observe-test-{}", std::process::id()));
        std::fs::create_dir_all(&data_dir).map_err(|err| err.to_string())?;
        std::fs::write(data_dir.join("PG_VERSION"), "16").map_err(|err| err.to_string())?;
        let runtime_config =
            runtime_test_config_with_data_dir(&data_dir).map_err(|err| err.to_string())?;
        let pg = replica_pg_state_with_primary_conninfo("node-b", 5432)?;
        let dcs = dcs_view_for_member("node-b", "node-b", 5432)?;
        let observation = observe(&ha_context(runtime_config, pg, dcs)?, UnixMillis(123))
            .map_err(|err| err.to_string())?;

        assert_eq!(
            observation.resolved_upstream,
            Some(MemberId("node-b".to_string()))
        );
        assert_eq!(
            observation.self_candidate,
            CandidateState::Promote(crate::state::ObservedWalPosition {
                timeline: Some(TimelineId(7)),
                lsn: WalLsn(67_272_104),
            })
        );
        Ok(())
    }

    #[test]
    fn observe_does_not_reuse_dcs_identity_after_basebackup_until_pg_refresh() -> Result<(), String>
    {
        let data_dir =
            std::env::temp_dir().join(format!("pgtm-ha-observe-reset-{}", std::process::id()));
        std::fs::create_dir_all(&data_dir).map_err(|err| err.to_string())?;
        std::fs::write(data_dir.join("PG_VERSION"), "16").map_err(|err| err.to_string())?;
        let runtime_config =
            runtime_test_config_with_data_dir(&data_dir).map_err(|err| err.to_string())?;
        let pg = PgInfoState::unknown(WorkerStatus::Running, SqlStatus::Unknown, None);
        let dcs = dcs_view_for_member("node-a", "node-a", 5432)?;
        let mut ctx = ha_context(runtime_config, pg, dcs)?;
        ctx.state_channel.current.managed_roles_reconciled = true;
        ctx.state_channel.current.publication = PublicationState::Projected(
            AuthorityProjection::NoPrimary(crate::ha::types::NoPrimaryProjection::LeaseOpen),
        );
        ctx.observed.process = new_state_channel(ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: Some(crate::process::state::JobOutcome::Success {
                id: crate::state::JobId("job-1".to_string()),
                job_kind: crate::process::jobs::ProcessJobKind::BaseBackup,
                finished_at: UnixMillis(10),
            }),
        })
        .1;

        let observation = observe(&ctx, UnixMillis(123)).map_err(|err| err.to_string())?;

        assert_eq!(observation.local_data, LocalDataState::ConsistentReplica);
        assert!(observation.managed_roles_reconciled);
        Ok(())
    }

    fn ha_context(
        runtime_config: crate::config_v2::RuntimeConfigV2,
        pg: PgInfoState,
        dcs: DcsSnapshot,
    ) -> Result<HaRuntimeCtx<'static>, String> {
        let cfg = Box::leak(Box::new(runtime_config));
        let (pg_publisher, pg_subscriber) = new_state_channel(pg);
        let (dcs_publisher, dcs_subscriber) = new_state_channel(dcs);
        let (process_publisher, process) = new_state_channel(ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        });
        let (process_intent_inbox, _process_intent_receiver) =
            tokio::sync::mpsc::unbounded_channel();

        drop(pg_publisher);
        drop(dcs_publisher);
        drop(process_publisher);

        Ok(bootstrap_with_now(
            cfg,
            HaObservedState {
                pg: pg_subscriber,
                dcs: dcs_subscriber,
                process,
            },
            HaControlPlane {
                process_intent_inbox,
                dcs_handle: DcsHandle::closed(),
            },
            Box::new(|| Ok(UnixMillis(123))),
        )
        .0)
    }
}
