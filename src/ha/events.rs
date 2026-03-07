use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::{
    actions::HaAction,
    decision::HaDecision,
    lower::HaEffectPlan,
    state::{HaPhase, HaWorkerCtx},
};

fn ha_append_base_fields(fields: &mut StructuredFields, ctx: &HaWorkerCtx, ha_tick: u64) {
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.self_id.0.clone());
    fields.insert("ha_tick", ha_tick);
    fields.insert("ha_dispatch_seq", ha_tick);
}

fn ha_append_action_fields(fields: &mut StructuredFields, action_index: usize, action: &HaAction) {
    fields.insert("action_index", action_index);
    fields.insert("action_id", action.id().label());
    if let HaAction::FollowLeader { leader_member_id } = action {
        fields.insert("leader_member_id", leader_member_id.clone());
    }
}

fn ha_insert_serialized<T: serde::Serialize>(
    fields: &mut StructuredFields,
    key: &str,
    value: &T,
) -> Result<(), WorkerError> {
    fields
        .insert_serialized(key, value)
        .map_err(|err| WorkerError::Message(format!("ha attr serialization failed: {err}")))
}

fn ha_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(severity, message, AppEventHeader::new(name, "ha", result))
}

fn emit_ha_event(
    ctx: &HaWorkerCtx,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

pub(crate) fn emit_ha_action_intent(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha action intent",
        "ha.action.intent",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha intent log emit failed",
    )
}

pub(crate) fn emit_ha_decision_selected(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    decision: &HaDecision,
    plan: &HaEffectPlan,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha decision selected",
        "ha.decision.selected",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_insert_serialized(fields, "decision", decision)?;
    fields.insert("planned_dispatch_step_count", plan.dispatch_step_count());
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha decision log emit failed",
    )
}

pub(crate) fn emit_ha_effect_plan_selected(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    plan: &HaEffectPlan,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha effect plan selected",
        "ha.effect_plan.selected",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_insert_serialized(fields, "effect_plan", plan)?;
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha effect plan log emit failed",
    )
}

pub(crate) fn emit_ha_action_dispatch(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha action dispatch",
        "ha.action.dispatch",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha dispatch log emit failed",
    )
}

pub(crate) fn emit_ha_action_result_ok(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha action result",
        "ha.action.result",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha result log emit failed",
    )
}

pub(crate) fn emit_ha_action_result_skipped(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha action skipped",
        "ha.action.result",
        "skipped",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha result log emit failed",
    )
}

pub(crate) fn emit_ha_action_result_failed(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    error: String,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Warn,
        "ha action failed",
        "ha.action.result",
        "failed",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    fields.insert("error", error);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha result log emit failed",
    )
}

pub(crate) fn emit_ha_lease_transition(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    acquired: bool,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Info,
        if acquired {
            "ha leader lease acquired"
        } else {
            "ha leader lease released"
        },
        if acquired {
            "ha.lease.acquired"
        } else {
            "ha.lease.released"
        },
        "ok",
    );
    ha_append_base_fields(event.fields_mut(), ctx, ha_tick);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha lease log emit failed",
    )
}

pub(crate) fn emit_ha_phase_transition(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    prev_phase: &HaPhase,
    next_phase: &HaPhase,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Info,
        "ha phase transition",
        "ha.phase.transition",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_insert_serialized(fields, "phase_prev", prev_phase)?;
    ha_insert_serialized(fields, "phase_next", next_phase)?;
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha phase log emit failed",
    )
}

pub(crate) fn emit_ha_role_transition(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    prev_role: &str,
    next_role: &str,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Info,
        "ha role transition",
        "ha.role.transition",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    fields.insert("role_prev", prev_role.to_string());
    fields.insert("role_next", next_role.to_string());
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha role log emit failed",
    )
}

pub(crate) fn ha_role_label(phase: &HaPhase) -> &'static str {
    match phase {
        HaPhase::Primary => "primary",
        HaPhase::Replica => "replica",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use crate::{
        config::{LogLevel, LoggingConfig, RuntimeConfig},
        dcs::{
            state::{DcsCache, DcsState, DcsTrust},
            store::{DcsStore, DcsStoreError, WatchEvent},
        },
        ha::{
            actions::HaAction,
            events::emit_ha_action_intent,
            state::{HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx},
        },
        logging::{decode_app_event, LogHandle, LogRecord, LogSink, SeverityText},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, MemberId, UnixMillis, WorkerError, WorkerStatus},
    };

    #[derive(Default)]
    struct NoopStore;

    impl DcsStore for NoopStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn put_path_if_absent(
            &mut self,
            _path: &str,
            _value: String,
        ) -> Result<bool, DcsStoreError> {
            Ok(true)
        }

        fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }
    }

    #[derive(Default)]
    struct CaptureSink {
        records: Mutex<Vec<LogRecord>>,
    }

    impl CaptureSink {
        fn records(&self) -> Result<Vec<LogRecord>, WorkerError> {
            let guard = self
                .records
                .lock()
                .map_err(|_| WorkerError::Message("capture sink lock poisoned".to_string()))?;
            Ok(guard.clone())
        }
    }

    impl LogSink for CaptureSink {
        fn emit(&self, record: &LogRecord) -> Result<(), crate::logging::LogError> {
            let mut guard = self.records.lock().map_err(|_| {
                crate::logging::LogError::SinkIo("capture sink lock poisoned".to_string())
            })?;
            guard.push(record.clone());
            Ok(())
        }
    }

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(PathBuf::from("/tmp/pgtuskmaster-test-events"))
            .with_logging(LoggingConfig {
                level: LogLevel::Debug,
                ..crate::test_harness::runtime_config::sample_logging_config()
            })
            .build()
    }

    fn sample_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
        }
    }

    fn sample_dcs_state(config: RuntimeConfig) -> DcsState {
        DcsState {
            worker: WorkerStatus::Running,
            trust: DcsTrust::FullQuorum,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config,
                init_lock: None,
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        }
    }

    fn build_context(log: LogHandle) -> HaWorkerCtx {
        let runtime_config = sample_runtime_config();
        let (config_publisher, config_subscriber) =
            new_state_channel(runtime_config.clone(), UnixMillis(1));
        let (pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(runtime_config.clone()), UnixMillis(1));
        let (process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (ha_publisher, _ha_subscriber) = new_state_channel(
            HaState {
                worker: WorkerStatus::Starting,
                phase: HaPhase::Init,
                tick: 0,
                decision: crate::ha::decision::HaDecision::NoChange,
            },
            UnixMillis(1),
        );
        let (process_tx, _process_rx) = tokio::sync::mpsc::unbounded_channel();

        let _ = config_publisher;
        let _ = pg_publisher;
        let _ = dcs_publisher;
        let _ = process_publisher;

        let mut ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
            publisher: ha_publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox: process_tx,
            dcs_store: Box::new(NoopStore),
            scope: "scope-a".to_string(),
            self_id: MemberId("node-a".to_string()),
        });
        ctx.log = log;
        ctx
    }

    #[test]
    fn action_intent_event_carries_action_and_leader_metadata() -> Result<(), WorkerError> {
        let sink = Arc::new(CaptureSink::default());
        let log = LogHandle::new("test-host".to_string(), sink.clone(), SeverityText::Trace);
        let ctx = build_context(log);

        emit_ha_action_intent(
            &ctx,
            11,
            2,
            &HaAction::FollowLeader {
                leader_member_id: "node-b".to_string(),
            },
        )?;

        let records = sink.records()?;
        assert_eq!(records.len(), 1);
        let decoded = decode_app_event(&records[0]).map_err(|err| {
            WorkerError::Message(format!("decode ha action intent event failed: {err}"))
        })?;
        assert_eq!(decoded.message, "ha action intent".to_string());
        assert_eq!(
            decoded.header,
            crate::logging::AppEventHeader::new("ha.action.intent", "ha", "ok")
        );
        assert_eq!(
            decoded.fields.get("action_id"),
            Some(&serde_json::Value::String(
                "follow_leader_node-b".to_string()
            ))
        );
        assert_eq!(
            decoded.fields.get("leader_member_id"),
            Some(&serde_json::Value::String("node-b".to_string()))
        );
        assert_eq!(
            decoded.fields.get("scope"),
            Some(&serde_json::Value::String("scope-a".to_string()))
        );
        assert_eq!(
            decoded.fields.get("member_id"),
            Some(&serde_json::Value::String("node-a".to_string()))
        );
        Ok(())
    }
}
