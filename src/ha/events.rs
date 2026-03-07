use std::collections::BTreeMap;

use crate::{
    logging::{EventMeta, SeverityText},
    state::WorkerError,
};

use super::{
    actions::HaAction,
    decision::HaDecision,
    lower::HaEffectPlan,
    state::{HaPhase, HaWorkerCtx},
};

pub(crate) fn ha_base_attrs(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
) -> BTreeMap<String, serde_json::Value> {
    let mut attrs = BTreeMap::new();
    attrs.insert(
        "scope".to_string(),
        serde_json::Value::String(ctx.scope.clone()),
    );
    attrs.insert(
        "member_id".to_string(),
        serde_json::Value::String(ctx.self_id.0.clone()),
    );
    attrs.insert(
        "ha_tick".to_string(),
        serde_json::Value::Number(serde_json::Number::from(ha_tick)),
    );
    attrs.insert(
        "ha_dispatch_seq".to_string(),
        serde_json::Value::Number(serde_json::Number::from(ha_tick)),
    );
    attrs
}

pub(crate) fn emit_ha_action_intent(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut attrs = ha_base_attrs(ctx, ha_tick);
    attrs.insert(
        "action_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(action_index as u64)),
    );
    attrs.insert(
        "action_id".to_string(),
        serde_json::Value::String(action.id().label()),
    );
    if let HaAction::FollowLeader { leader_member_id } = action {
        attrs.insert(
            "leader_member_id".to_string(),
            serde_json::Value::String(leader_member_id.clone()),
        );
    }
    emit_event(
        ctx,
        SeverityText::Debug,
        "ha action intent",
        "ha_worker::step_once",
        EventMeta::new("ha.action.intent", "ha", "ok"),
        attrs,
        "ha intent log emit failed",
    )
}

pub(crate) fn emit_ha_decision_selected(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    decision: &HaDecision,
    plan: &HaEffectPlan,
) -> Result<(), WorkerError> {
    let mut attrs = ha_base_attrs(ctx, ha_tick);
    attrs.insert("decision".to_string(), serialize_attr_value(decision)?);
    attrs.insert(
        "planned_dispatch_step_count".to_string(),
        serde_json::Value::Number(serde_json::Number::from(plan.dispatch_step_count() as u64)),
    );
    emit_event(
        ctx,
        SeverityText::Debug,
        "ha decision selected",
        "ha_worker::step_once",
        EventMeta::new("ha.decision.selected", "ha", "ok"),
        attrs,
        "ha decision log emit failed",
    )
}

pub(crate) fn emit_ha_effect_plan_selected(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    plan: &HaEffectPlan,
) -> Result<(), WorkerError> {
    let mut attrs = ha_base_attrs(ctx, ha_tick);
    attrs.insert("effect_plan".to_string(), serialize_attr_value(plan)?);
    emit_event(
        ctx,
        SeverityText::Debug,
        "ha effect plan selected",
        "ha_worker::step_once",
        EventMeta::new("ha.effect_plan.selected", "ha", "ok"),
        attrs,
        "ha effect plan log emit failed",
    )
}

pub(crate) fn emit_ha_action_dispatch(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut attrs = ha_base_attrs(ctx, ha_tick);
    attrs.insert(
        "action_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(action_index as u64)),
    );
    attrs.insert(
        "action_id".to_string(),
        serde_json::Value::String(action.id().label()),
    );
    emit_event(
        ctx,
        SeverityText::Debug,
        "ha action dispatch",
        "ha_worker::dispatch_actions",
        EventMeta::new("ha.action.dispatch", "ha", "ok"),
        attrs,
        "ha dispatch log emit failed",
    )
}

pub(crate) fn emit_ha_action_result_ok(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut attrs = ha_base_attrs(ctx, ha_tick);
    attrs.insert(
        "action_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(action_index as u64)),
    );
    attrs.insert(
        "action_id".to_string(),
        serde_json::Value::String(action.id().label()),
    );
    emit_event(
        ctx,
        SeverityText::Debug,
        "ha action result",
        "ha_worker::dispatch_actions",
        EventMeta::new("ha.action.result", "ha", "ok"),
        attrs,
        "ha result log emit failed",
    )
}

pub(crate) fn emit_ha_action_result_skipped(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut attrs = ha_base_attrs(ctx, ha_tick);
    attrs.insert(
        "action_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(action_index as u64)),
    );
    attrs.insert(
        "action_id".to_string(),
        serde_json::Value::String(action.id().label()),
    );
    emit_event(
        ctx,
        SeverityText::Debug,
        "ha action skipped",
        "ha_worker::dispatch_actions",
        EventMeta::new("ha.action.result", "ha", "skipped"),
        attrs,
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
    let mut attrs = ha_base_attrs(ctx, ha_tick);
    attrs.insert(
        "action_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(action_index as u64)),
    );
    attrs.insert(
        "action_id".to_string(),
        serde_json::Value::String(action.id().label()),
    );
    attrs.insert("error".to_string(), serde_json::Value::String(error));
    emit_event(
        ctx,
        SeverityText::Warn,
        "ha action failed",
        "ha_worker::dispatch_actions",
        EventMeta::new("ha.action.result", "ha", "failed"),
        attrs,
        "ha result log emit failed",
    )
}

pub(crate) fn emit_ha_lease_transition(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    acquired: bool,
) -> Result<(), WorkerError> {
    let attrs = ha_base_attrs(ctx, ha_tick);
    let (name, message) = if acquired {
        ("ha.lease.acquired", "ha leader lease acquired")
    } else {
        ("ha.lease.released", "ha leader lease released")
    };
    emit_event(
        ctx,
        SeverityText::Info,
        message,
        "ha_worker::dispatch_actions",
        EventMeta::new(name, "ha", "ok"),
        attrs,
        "ha lease log emit failed",
    )
}

pub(crate) fn ha_role_label(phase: &HaPhase) -> &'static str {
    match phase {
        HaPhase::Primary => "primary",
        HaPhase::Replica => "replica",
        _ => "unknown",
    }
}

pub(crate) fn serialize_attr_value<T: serde::Serialize>(
    value: &T,
) -> Result<serde_json::Value, WorkerError> {
    serde_json::to_value(value)
        .map_err(|err| WorkerError::Message(format!("ha attr serialization failed: {err}")))
}

fn emit_event(
    ctx: &HaWorkerCtx,
    severity: SeverityText,
    message: &str,
    origin: &str,
    meta: EventMeta,
    attrs: BTreeMap<String, serde_json::Value>,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_event(severity, message, origin, meta, attrs)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
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
        logging::{LogHandle, LogRecord, LogSink, SeverityText},
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
        let record = &records[0];
        assert_eq!(record.message, "ha action intent".to_string());
        assert_eq!(
            record.attributes.get("event.name"),
            Some(&serde_json::Value::String("ha.action.intent".to_string()))
        );
        assert_eq!(
            record.attributes.get("action_id"),
            Some(&serde_json::Value::String(
                "follow_leader_node-b".to_string()
            ))
        );
        assert_eq!(
            record.attributes.get("leader_member_id"),
            Some(&serde_json::Value::String("node-b".to_string()))
        );
        assert_eq!(
            record.attributes.get("scope"),
            Some(&serde_json::Value::String("scope-a".to_string()))
        );
        assert_eq!(
            record.attributes.get("member_id"),
            Some(&serde_json::Value::String("node-a".to_string()))
        );
        Ok(())
    }
}
