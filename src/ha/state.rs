use std::{path::PathBuf, time::Duration};

use crate::{
    config::{RoleAuthConfig, RuntimeConfig},
    dcs::{state::DcsState, store::DcsLeaderStore},
    logging::LogHandle,
    pginfo::state::{PgInfoState, PgSslMode},
    process::{
        jobs::ShutdownMode,
        state::{ProcessJobRequest, ProcessState},
    },
    state::{
        MemberId, StatePublisher, StateSubscriber, UnixMillis, Versioned, WorkerError, WorkerStatus,
    },
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ClusterMode {
    DcsUnavailable,
    UninitializedNoBootstrapOwner,
    UninitializedBootstrapInProgress { holder: MemberId },
    InitializedLeaderPresent { leader: MemberId },
    InitializedNoLeaderFreshQuorum,
    InitializedNoLeaderNoFreshQuorum,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum DesiredNodeState {
    Bootstrap { plan: BootstrapPlan },
    Primary { plan: PrimaryPlan },
    Replica { plan: ReplicaPlan },
    Quiescent { reason: QuiescentReason },
    Fence { plan: FencePlan },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BootstrapPlan {
    InitDb,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PrimaryPlan {
    KeepLeader,
    AcquireLeaderThenResumePrimary,
    AcquireLeaderThenPromote,
    AcquireLeaderThenStartPrimary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ReplicaPlan {
    DirectFollow { leader_member_id: MemberId },
    RewindThenFollow { leader_member_id: MemberId },
    BasebackupThenFollow { leader_member_id: MemberId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum QuiescentReason {
    WaitingForBootstrapWinner,
    WaitingForAuthoritativeLeader,
    WaitingForFreshQuorum,
    WaitingForAuthoritativeClusterState,
    WaitingForRecoveryPreconditions,
    UnsafeUninitializedPgData,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FencePlan {
    StopAndStayNonWritable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LeadershipTransferState {
    None,
    WaitingForOtherLeader { target: Option<MemberId> },
}

impl Default for LeadershipTransferState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HaState {
    pub(crate) worker: WorkerStatus,
    pub(crate) cluster_mode: ClusterMode,
    pub(crate) desired_state: DesiredNodeState,
    pub(crate) leadership_transfer: LeadershipTransferState,
    pub(crate) tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorldSnapshot {
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecideInput {
    pub(crate) current: HaState,
    pub(crate) world: WorldSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecideOutput {
    pub(crate) next: HaState,
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
    pub(crate) postgres_binary: PathBuf,
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
    pub(crate) connect_timeout_s: u32,
    pub(crate) shutdown_mode: ShutdownMode,
}

impl ProcessDispatchDefaults {
    pub(crate) fn contract_stub() -> Self {
        Self {
            postgres_binary: PathBuf::from("/usr/bin/postgres"),
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
            connect_timeout_s: 5,
            shutdown_mode: ShutdownMode::Fast,
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
            state: HaState {
                worker: WorkerStatus::Starting,
                cluster_mode: ClusterMode::DcsUnavailable,
                desired_state: DesiredNodeState::Quiescent {
                    reason: QuiescentReason::WaitingForAuthoritativeClusterState,
                },
                leadership_transfer: LeadershipTransferState::None,
                tick: 0,
            },
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
