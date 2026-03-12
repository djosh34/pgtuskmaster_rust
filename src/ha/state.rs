use std::{path::PathBuf, time::Duration};

use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::{RoleAuthConfig, RuntimeConfig},
    dcs::{state::DcsState, store::DcsLeaderStore},
    logging::LogHandle,
    pginfo::state::{PgInfoState, PgSslMode},
    process::state::{ProcessJobRequest, ProcessState},
    state::{MemberId, StatePublisher, StateSubscriber, UnixMillis, WorkerError, WorkerStatus},
};

use super::types::{PublicationState, ReconcileAction, TargetRole};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HaState {
    pub(crate) worker: WorkerStatus,
    pub(crate) tick: u64,
    pub(crate) publication: PublicationState,
    pub(crate) role: TargetRole,
    pub(crate) clear_switchover: bool,
    pub(crate) planned_actions: Vec<ReconcileAction>,
}

pub(crate) struct HaWorkerCtx {
    pub(crate) poll_interval: Duration,
    pub(crate) state: HaState,
    pub(crate) publisher: StatePublisher<HaState>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_inbox: UnboundedSender<ProcessJobRequest>,
    pub(crate) dcs_store: Box<dyn DcsLeaderStore>,
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
    pub(crate) process_defaults: ProcessDispatchDefaults,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) log: LogHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessDispatchDefaults {
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) replicator_username: String,
    pub(crate) replicator_auth: RoleAuthConfig,
    pub(crate) rewinder_username: String,
    pub(crate) rewinder_auth: RoleAuthConfig,
    pub(crate) remote_dbname: String,
    pub(crate) remote_ssl_mode: PgSslMode,
    pub(crate) remote_ssl_root_cert: Option<PathBuf>,
    pub(crate) connect_timeout_s: u32,
}

impl ProcessDispatchDefaults {
    pub(crate) fn contract_stub() -> Self {
        Self {
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
            log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
            replicator_username: "replicator".to_string(),
            replicator_auth: contract_stub_password_auth(),
            rewinder_username: "rewinder".to_string(),
            rewinder_auth: contract_stub_password_auth(),
            remote_dbname: "postgres".to_string(),
            remote_ssl_mode: PgSslMode::Prefer,
            remote_ssl_root_cert: None,
            connect_timeout_s: 5,
        }
    }
}

fn contract_stub_password_auth() -> RoleAuthConfig {
    RoleAuthConfig::Password {
        password: crate::config::SecretSource::Inline {
            content: "secret-password".to_string(),
        },
    }
}

pub(crate) struct HaWorkerContractStubInputs {
    pub(crate) publisher: StatePublisher<HaState>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_inbox: UnboundedSender<ProcessJobRequest>,
    pub(crate) dcs_store: Box<dyn DcsLeaderStore>,
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
}

impl HaState {
    pub(crate) fn initial(worker: WorkerStatus) -> Self {
        Self {
            worker,
            tick: 0,
            publication: PublicationState::unknown(),
            role: TargetRole::Idle(super::types::IdleReason::AwaitingLeader),
            clear_switchover: false,
            planned_actions: Vec::new(),
        }
    }
}

impl HaWorkerCtx {
    pub(crate) fn contract_stub(inputs: HaWorkerContractStubInputs) -> Self {
        let HaWorkerContractStubInputs {
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox,
            dcs_store,
            scope,
            self_id,
        } = inputs;

        Self {
            poll_interval: Duration::from_millis(10),
            state: HaState::initial(WorkerStatus::Starting),
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox,
            dcs_store,
            scope,
            self_id,
            process_defaults: ProcessDispatchDefaults::contract_stub(),
            now: Box::new(|| Ok(UnixMillis(0))),
            log: LogHandle::disabled(),
        }
    }
}
