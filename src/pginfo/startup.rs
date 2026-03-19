use std::time::Duration;

use crate::{
    config::defaults::default_pg_ssl_mode,
    config::RuntimeConfig,
    logging::LogSender,
    process::state::ProcessRuntimePlan,
    state::{new_state_channel, NodeIdentity, PgEndpoint, WorkerError},
};

use super::state::{
    PgConnInfo, PgInfoCadence, PgInfoRuntime, PgInfoState, PgInfoStateChannel, PgInfoWorkerCtx,
};

pub(crate) struct PgInfoRuntimeBundle {
    pub(crate) state: crate::state::StateSubscriber<PgInfoState>,
    pub(crate) worker: PgInfoWorker,
}

pub(crate) struct PgInfoWorker(PgInfoWorkerCtx);

impl PgInfoWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(
    identity: NodeIdentity,
    cfg: &RuntimeConfig,
    process_plan: &ProcessRuntimePlan,
    log: LogSender,
) -> PgInfoRuntimeBundle {
    let (publisher, state) = new_state_channel(PgInfoState::starting());

    PgInfoRuntimeBundle {
        state,
        worker: PgInfoWorker(PgInfoWorkerCtx {
            identity,
            probe_conninfo: PgConnInfo {
                endpoint: PgEndpoint::UnixSocket {
                    socket_dir: process_plan.postgres.paths.socket_dir.clone(),
                    port: process_plan.postgres.port,
                },
                hostaddr: None,
                user: cfg
                    .postgres
                    .roles
                    .mandatory
                    .superuser
                    .username
                    .as_str()
                    .to_owned(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: None,
                options: None,
                tls: super::conninfo::PgClientTls {
                    mode: default_pg_ssl_mode(),
                    root_cert: None,
                    client_cert: None,
                    client_key: None,
                },
            },
            cadence: PgInfoCadence {
                poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
            },
            state_channel: PgInfoStateChannel {
                publisher,
                last_emitted_sql_status: None,
            },
            runtime: PgInfoRuntime { log },
        }),
    }
}
