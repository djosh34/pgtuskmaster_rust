use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::{
    keys::DcsKey,
    state::{
        build_local_member_slot, evaluate_trust, DcsCache, DcsState, DcsTrust, DcsWorkerCtx,
        InitLockRecord, LeaderLeaseRecord, MemberSlot, SwitchoverIntentRecord,
    },
    store::{leader_path, refresh_from_etcd_watch, write_local_member_slot},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsValue {
    Member(MemberSlot),
    Leader(LeaderLeaseRecord),
    Switchover(SwitchoverIntentRecord),
    Config(Box<crate::config::RuntimeConfig>),
    InitLock(InitLockRecord),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsWatchUpdate {
    Put { key: DcsKey, value: Box<DcsValue> },
    Delete { key: DcsKey },
}

fn dcs_append_base_fields(fields: &mut StructuredFields, ctx: &DcsWorkerCtx) {
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.self_id.0.clone());
}

fn dcs_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(severity, message, AppEventHeader::new(name, "dcs", result))
}

fn emit_dcs_event(
    ctx: &DcsWorkerCtx,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

fn dcs_io_error_severity(err: &crate::dcs::store::DcsStoreError) -> SeverityText {
    match err {
        crate::dcs::store::DcsStoreError::Io(_) => SeverityText::Warn,
        _ => SeverityText::Error,
    }
}

fn dcs_refresh_error_severity(err: &crate::dcs::store::DcsStoreError) -> SeverityText {
    match err {
        crate::dcs::store::DcsStoreError::Io(_)
        | crate::dcs::store::DcsStoreError::InvalidKey(_)
        | crate::dcs::store::DcsStoreError::MissingValue(_) => SeverityText::Warn,
        _ => SeverityText::Error,
    }
}

pub(crate) async fn run(mut ctx: DcsWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) fn apply_watch_update(cache: &mut DcsCache, update: DcsWatchUpdate) {
    match update {
        DcsWatchUpdate::Put { key, value } => match (key, *value) {
            (DcsKey::Member(member_id), DcsValue::Member(record)) => {
                cache.member_slots.insert(member_id, record);
            }
            (DcsKey::Leader, DcsValue::Leader(record)) => {
                cache.leader_lease = Some(record);
            }
            (DcsKey::Switchover, DcsValue::Switchover(record)) => {
                cache.switchover_intent = Some(record);
            }
            (DcsKey::Config, DcsValue::Config(_config)) => {}
            (DcsKey::InitLock, DcsValue::InitLock(record)) => {
                cache.init_lock = Some(record);
            }
            _ => {}
        },
        DcsWatchUpdate::Delete { key } => match key {
            DcsKey::Member(member_id) => {
                cache.member_slots.remove(&member_id);
            }
            DcsKey::Leader => {
                cache.leader_lease = None;
            }
            DcsKey::Switchover => {
                cache.switchover_intent = None;
            }
            DcsKey::Config => {}
            DcsKey::InitLock => {
                cache.init_lock = None;
            }
        },
    }
}

pub(crate) async fn step_once(ctx: &mut DcsWorkerCtx) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let pg_snapshot = ctx.pg_subscriber.latest();
    let member_ttl_ms = ctx.member_ttl_ms;
    let local_member_path = format!("/{}/member/{}", ctx.scope.trim_matches('/'), ctx.self_id.0);
    let pg_snapshot_stale = pg_snapshot
        .last_refresh_at()
        .is_none_or(|last_refresh_at| now.0.saturating_sub(last_refresh_at.0) > member_ttl_ms);

    let mut store_healthy = ctx.store.healthy();
    if store_healthy && pg_snapshot_stale {
        match ctx.store.delete_path(local_member_path.as_str()) {
            Ok(()) => {
                ctx.cache.member_slots.remove(&ctx.self_id);
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs local member delete failed",
                    "dcs.local_member.delete_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs local member delete log emit failed",
                )?;
                store_healthy = false;
            }
        }
    } else if store_healthy {
        let local_member = build_local_member_slot(
            &ctx.self_id,
            ctx.local_postgres_host.as_str(),
            ctx.local_postgres_port,
            ctx.local_api_url.as_deref(),
            member_ttl_ms,
            &pg_snapshot,
        );
        match write_local_member_slot(ctx.store.as_mut(), &ctx.scope, &local_member, member_ttl_ms)
        {
            Ok(()) => {
                ctx.cache
                    .member_slots
                    .insert(ctx.self_id.clone(), local_member);
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs local member write failed",
                    "dcs.local_member.write_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs local member write log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let events = match ctx.store.drain_watch_events() {
        Ok(events) => events,
        Err(err) => {
            let mut event = dcs_event(
                dcs_io_error_severity(&err),
                "dcs watch drain failed",
                "dcs.watch.drain_failed",
                "failed",
            );
            let fields = event.fields_mut();
            dcs_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                event,
                "dcs drain log emit failed",
            )?;
            store_healthy = false;
            Vec::new()
        }
    };
    match refresh_from_etcd_watch(&ctx.scope, &mut ctx.cache, events) {
        Ok(result) => {
            if result.had_errors {
                let mut event = dcs_event(
                    SeverityText::Warn,
                    "dcs watch refresh had errors",
                    "dcs.watch.apply_had_errors",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("applied", result.applied);
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs refresh had_errors log emit failed",
                )?;
                store_healthy = false;
            }
        }
        Err(err) => {
            let mut event = dcs_event(
                dcs_refresh_error_severity(&err),
                "dcs watch refresh failed",
                "dcs.watch.refresh_failed",
                "failed",
            );
            let fields = event.fields_mut();
            dcs_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                event,
                "dcs refresh log emit failed",
            )?;
            store_healthy = false;
        }
    }

    if store_healthy {
        let scope_prefix = format!("/{}/", ctx.scope.trim_matches('/'));
        match ctx.store.snapshot_prefix(scope_prefix.as_str()) {
            Ok(snapshot_events) => {
                match refresh_from_etcd_watch(&ctx.scope, &mut ctx.cache, snapshot_events) {
                    Ok(result) => {
                        if result.had_errors {
                            let mut event = dcs_event(
                                SeverityText::Warn,
                                "dcs snapshot refresh had errors",
                                "dcs.snapshot.apply_had_errors",
                                "failed",
                            );
                            let fields = event.fields_mut();
                            dcs_append_base_fields(fields, ctx);
                            fields.insert("applied", result.applied);
                            emit_dcs_event(
                                ctx,
                                "dcs_worker::step_once",
                                event,
                                "dcs snapshot had_errors log emit failed",
                            )?;
                            store_healthy = false;
                        }
                    }
                    Err(err) => {
                        let mut event = dcs_event(
                            dcs_refresh_error_severity(&err),
                            "dcs snapshot refresh failed",
                            "dcs.snapshot.refresh_failed",
                            "failed",
                        );
                        let fields = event.fields_mut();
                        dcs_append_base_fields(fields, ctx);
                        fields.insert("error", err.to_string());
                        emit_dcs_event(
                            ctx,
                            "dcs_worker::step_once",
                            event,
                            "dcs snapshot refresh log emit failed",
                        )?;
                        store_healthy = false;
                    }
                }
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs snapshot read failed",
                    "dcs.snapshot.read_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs snapshot read log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let stale_leader_holder = ctx.cache.leader_lease.as_ref().and_then(|leader| {
        (!ctx.cache.member_slots.contains_key(&leader.holder)).then(|| leader.holder.clone())
    });
    if let Some(member_id) = stale_leader_holder {
        let leader_key = leader_path(ctx.scope.as_str());
        match ctx.store.delete_path(leader_key.as_str()) {
            Ok(()) => {
                ctx.cache.leader_lease = None;
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs stale leader cleanup failed",
                    "dcs.leader.cleanup_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("leader_member_id", member_id.0);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs stale leader cleanup log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let trust = evaluate_trust(store_healthy, &ctx.cache, &ctx.self_id);
    let worker = if store_healthy {
        crate::state::WorkerStatus::Running
    } else {
        crate::state::WorkerStatus::Faulted(WorkerError::Message("dcs store unhealthy".to_string()))
    };

    let next = DcsState {
        worker,
        trust: if store_healthy {
            trust
        } else {
            DcsTrust::NotTrusted
        },
        cache: ctx.cache.clone(),
        last_refresh_at: Some(now),
    };
    if ctx.last_emitted_store_healthy != Some(store_healthy) {
        ctx.last_emitted_store_healthy = Some(store_healthy);
        let mut event = dcs_event(
            if store_healthy {
                SeverityText::Info
            } else {
                SeverityText::Warn
            },
            "dcs store health transition",
            "dcs.store.health_transition",
            if store_healthy { "recovered" } else { "failed" },
        );
        let fields = event.fields_mut();
        dcs_append_base_fields(fields, ctx);
        fields.insert("store_healthy", store_healthy);
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            event,
            "dcs health transition log emit failed",
        )?;
    }
    if ctx.last_emitted_trust.as_ref() != Some(&next.trust) {
        let prev = ctx
            .last_emitted_trust
            .as_ref()
            .map(|value| format!("{value:?}").to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        ctx.last_emitted_trust = Some(next.trust.clone());
        let mut event = dcs_event(
            SeverityText::Info,
            "dcs trust transition",
            "dcs.trust.transition",
            "ok",
        );
        let fields = event.fields_mut();
        dcs_append_base_fields(fields, ctx);
        fields.insert("trust_prev", prev);
        fields.insert("trust_next", format!("{:?}", next.trust).to_lowercase());
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            event,
            "dcs trust transition log emit failed",
        )?;
    }
    ctx.publisher
        .publish(next)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))?;
    Ok(())
}

fn now_unix_millis() -> Result<crate::state::UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(crate::state::UnixMillis(millis))
}
