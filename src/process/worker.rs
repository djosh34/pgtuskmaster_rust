use std::process::Stdio;

use tokio::{
    process::{Child, Command},
    sync::mpsc::error::TryRecvError,
};

use crate::{
    config::ProcessConfig,
    state::{UnixMillis, WorkerError, WorkerStatus},
};

use super::{
    jobs::{
        ActiveJob, ActiveJobKind, CancelReason, ProcessCommandSpec, ProcessError, ProcessExit,
        ProcessHandle,
    },
    state::{
        ActiveRuntime, JobOutcome, ProcessJobKind, ProcessJobRejection, ProcessJobRequest,
        ProcessState, ProcessWorkerCtx,
    },
};

#[derive(Default)]
pub(crate) struct TokioCommandRunner;

struct TokioProcessHandle {
    child: Child,
}

impl ProcessHandle for TokioProcessHandle {
    fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError> {
        let status = self
            .child
            .try_wait()
            .map_err(|err| ProcessError::SpawnFailure {
                binary: "process-child".to_string(),
                message: err.to_string(),
            })?;

        Ok(status.map(|exit| {
            if exit.success() {
                ProcessExit::Success
            } else {
                ProcessExit::Failure { code: exit.code() }
            }
        }))
    }

    fn cancel<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ProcessError>> + Send + 'a>>
    {
        Box::pin(async move {
            if self
                .child
                .try_wait()
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?
                .is_some()
            {
                return Ok(());
            }

            self.child
                .start_kill()
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?;
            let _ = self
                .child
                .wait()
                .await
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?;
            Ok(())
        })
    }
}

impl super::jobs::ProcessCommandRunner for TokioCommandRunner {
    fn spawn(&mut self, spec: ProcessCommandSpec) -> Result<Box<dyn ProcessHandle>, ProcessError> {
        let mut command = Command::new(&spec.program);
        command
            .args(spec.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = command.spawn().map_err(|err| ProcessError::SpawnFailure {
            binary: spec.program.display().to_string(),
            message: err.to_string(),
        })?;

        Ok(Box::new(TokioProcessHandle { child }))
    }
}

pub(crate) fn can_accept_job(state: &ProcessState) -> bool {
    matches!(state, ProcessState::Idle { .. })
}

pub(crate) async fn run(mut ctx: ProcessWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    match ctx.inbox.try_recv() {
        Ok(request) => {
            start_job(ctx, request).await?;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {}
    }

    tick_active_job(ctx).await
}

pub(crate) async fn start_job(
    ctx: &mut ProcessWorkerCtx,
    request: ProcessJobRequest,
) -> Result<(), WorkerError> {
    if !can_accept_job(&ctx.state) {
        let now = current_time(ctx)?;
        ctx.last_rejection = Some(ProcessJobRejection {
            id: request.id,
            error: ProcessError::Busy,
            rejected_at: now,
        });
        return Ok(());
    }

    let now = current_time(ctx)?;
    let timeout_ms = timeout_for_kind(&request.kind, &ctx.config);
    let deadline_at = UnixMillis(now.0.saturating_add(timeout_ms));

    let command = match build_command(&ctx.config, &request.kind) {
        Ok(command) => command,
        Err(error) => {
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let handle = match ctx.command_runner.spawn(command) {
        Ok(handle) => handle,
        Err(error) => {
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let active = ActiveJob {
        id: request.id.clone(),
        kind: active_kind(&request.kind),
        started_at: now,
        deadline_at,
    };

    ctx.active_runtime = Some(ActiveRuntime {
        request,
        timeout_ms,
        started_at: now,
        deadline_at,
        handle,
    });
    ctx.state = ProcessState::Running {
        worker: WorkerStatus::Running,
        active,
    };
    publish_state(ctx, now)
}

pub(crate) async fn tick_active_job(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    let mut runtime = match ctx.active_runtime.take() {
        Some(runtime) => runtime,
        None => return Ok(()),
    };

    let now = current_time(ctx)?;
    if now.0 >= runtime.deadline_at.0 {
        let cancel_result = runtime.handle.cancel().await;
        let outcome = match cancel_result {
            Ok(()) => JobOutcome::Timeout {
                id: runtime.request.id,
                finished_at: now,
            },
            Err(error) => JobOutcome::Failure {
                id: runtime.request.id,
                error,
                finished_at: now,
            },
        };
        transition_to_idle(ctx, outcome, now)?;
        return Ok(());
    }

    let poll = runtime.handle.poll_exit();
    match poll {
        Ok(None) => {
            ctx.active_runtime = Some(runtime);
            Ok(())
        }
        Ok(Some(ProcessExit::Success)) => {
            let outcome = JobOutcome::Success {
                id: runtime.request.id,
                finished_at: now,
            };
            transition_to_idle(ctx, outcome, now)
        }
        Ok(Some(exit)) => {
            let outcome = JobOutcome::Failure {
                id: runtime.request.id,
                error: ProcessError::from_exit(exit),
                finished_at: now,
            };
            transition_to_idle(ctx, outcome, now)
        }
        Err(error) => {
            let outcome = JobOutcome::Failure {
                id: runtime.request.id,
                error,
                finished_at: now,
            };
            transition_to_idle(ctx, outcome, now)
        }
    }
}

pub(crate) async fn cancel_active_job(
    ctx: &mut ProcessWorkerCtx,
    _reason: CancelReason,
) -> Result<(), WorkerError> {
    let mut runtime = match ctx.active_runtime.take() {
        Some(runtime) => runtime,
        None => return Ok(()),
    };

    let now = current_time(ctx)?;
    let cancel_result = runtime.handle.cancel().await;
    let outcome = match cancel_result {
        Ok(()) => JobOutcome::Cancelled {
            id: runtime.request.id,
            finished_at: now,
        },
        Err(error) => JobOutcome::Failure {
            id: runtime.request.id,
            error,
            finished_at: now,
        },
    };

    transition_to_idle(ctx, outcome, now)
}

fn transition_to_idle(
    ctx: &mut ProcessWorkerCtx,
    outcome: JobOutcome,
    now: UnixMillis,
) -> Result<(), WorkerError> {
    ctx.state = ProcessState::Idle {
        worker: WorkerStatus::Running,
        last_outcome: Some(outcome),
    };
    publish_state(ctx, now)
}

fn publish_state(ctx: &mut ProcessWorkerCtx, now: UnixMillis) -> Result<(), WorkerError> {
    ctx.publisher
        .publish(ctx.state.clone(), now)
        .map_err(|err| WorkerError::Message(format!("process publish failed: {err}")))?;
    Ok(())
}

fn current_time(ctx: &mut ProcessWorkerCtx) -> Result<UnixMillis, WorkerError> {
    (ctx.now)()
}

pub(crate) fn system_now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

fn timeout_for_kind(kind: &ProcessJobKind, config: &ProcessConfig) -> u64 {
    match kind {
        ProcessJobKind::Bootstrap(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::PgRewind(spec) => spec.timeout_ms.unwrap_or(config.pg_rewind_timeout_ms),
        ProcessJobKind::Fencing(spec) => spec.timeout_ms.unwrap_or(config.fencing_timeout_ms),
        ProcessJobKind::Promote(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::Demote(spec) => spec.timeout_ms.unwrap_or(config.fencing_timeout_ms),
        ProcessJobKind::StartPostgres(spec) => {
            spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms)
        }
        ProcessJobKind::StopPostgres(spec) => spec.timeout_ms.unwrap_or(config.fencing_timeout_ms),
        ProcessJobKind::RestartPostgres(spec) => {
            spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms)
        }
    }
}

fn active_kind(kind: &ProcessJobKind) -> ActiveJobKind {
    match kind {
        ProcessJobKind::Bootstrap(_) => ActiveJobKind::Bootstrap,
        ProcessJobKind::PgRewind(_) => ActiveJobKind::PgRewind,
        ProcessJobKind::Promote(_) => ActiveJobKind::Promote,
        ProcessJobKind::Demote(_) => ActiveJobKind::Demote,
        ProcessJobKind::StartPostgres(_) => ActiveJobKind::StartPostgres,
        ProcessJobKind::StopPostgres(_) => ActiveJobKind::StopPostgres,
        ProcessJobKind::RestartPostgres(_) => ActiveJobKind::RestartPostgres,
        ProcessJobKind::Fencing(_) => ActiveJobKind::Fencing,
    }
}

fn build_command(
    config: &ProcessConfig,
    kind: &ProcessJobKind,
) -> Result<ProcessCommandSpec, ProcessError> {
    match kind {
        ProcessJobKind::Bootstrap(spec) => {
            validate_non_empty_path("bootstrap.data_dir", &spec.data_dir)?;
            Ok(ProcessCommandSpec {
                program: config.binaries.initdb.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-A".to_string(),
                    "trust".to_string(),
                    "-U".to_string(),
                    "postgres".to_string(),
                ],
            })
        }
        ProcessJobKind::PgRewind(spec) => {
            validate_non_empty_path("pg_rewind.target_data_dir", &spec.target_data_dir)?;
            if spec.source_conninfo.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo must not be empty".to_string(),
                ));
            }
            Ok(ProcessCommandSpec {
                program: config.binaries.pg_rewind.clone(),
                args: vec![
                    "--target-pgdata".to_string(),
                    spec.target_data_dir.display().to_string(),
                    "--source-server".to_string(),
                    spec.source_conninfo.clone(),
                ],
            })
        }
        ProcessJobKind::Promote(spec) => {
            validate_non_empty_path("promote.data_dir", &spec.data_dir)?;
            let mut args = vec![
                "-D".to_string(),
                spec.data_dir.display().to_string(),
                "promote".to_string(),
                "-w".to_string(),
            ];
            if let Some(wait_seconds) = spec.wait_seconds {
                args.push("-t".to_string());
                args.push(wait_seconds.to_string());
            }
            Ok(ProcessCommandSpec {
                program: config.binaries.pg_ctl.clone(),
                args,
            })
        }
        ProcessJobKind::Demote(spec) => {
            validate_non_empty_path("demote.data_dir", &spec.data_dir)?;
            Ok(ProcessCommandSpec {
                program: config.binaries.pg_ctl.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "stop".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                ],
            })
        }
        ProcessJobKind::StartPostgres(spec) => {
            validate_non_empty_path("start_postgres.data_dir", &spec.data_dir)?;
            validate_non_empty_path("start_postgres.socket_dir", &spec.socket_dir)?;
            validate_non_empty_path("start_postgres.log_file", &spec.log_file)?;
            if spec.host.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "start_postgres.host must not be empty".to_string(),
                ));
            }
            let wait_seconds = spec.wait_seconds.unwrap_or(30);
            Ok(ProcessCommandSpec {
                program: config.binaries.pg_ctl.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-l".to_string(),
                    spec.log_file.display().to_string(),
                    "-o".to_string(),
                    format!(
                        "-h {} -p {} -k {}",
                        spec.host,
                        spec.port,
                        spec.socket_dir.display()
                    ),
                    "start".to_string(),
                    "-w".to_string(),
                    "-t".to_string(),
                    wait_seconds.to_string(),
                ],
            })
        }
        ProcessJobKind::StopPostgres(spec) => {
            validate_non_empty_path("stop_postgres.data_dir", &spec.data_dir)?;
            Ok(ProcessCommandSpec {
                program: config.binaries.pg_ctl.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "stop".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                ],
            })
        }
        ProcessJobKind::RestartPostgres(spec) => {
            validate_non_empty_path("restart_postgres.data_dir", &spec.data_dir)?;
            validate_non_empty_path("restart_postgres.socket_dir", &spec.socket_dir)?;
            validate_non_empty_path("restart_postgres.log_file", &spec.log_file)?;
            if spec.host.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "restart_postgres.host must not be empty".to_string(),
                ));
            }
            let wait_seconds = spec.wait_seconds.unwrap_or(30);
            Ok(ProcessCommandSpec {
                program: config.binaries.pg_ctl.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-l".to_string(),
                    spec.log_file.display().to_string(),
                    "-o".to_string(),
                    format!(
                        "-h {} -p {} -k {}",
                        spec.host,
                        spec.port,
                        spec.socket_dir.display()
                    ),
                    "restart".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                    "-t".to_string(),
                    wait_seconds.to_string(),
                ],
            })
        }
        ProcessJobKind::Fencing(spec) => {
            validate_non_empty_path("fencing.data_dir", &spec.data_dir)?;
            Ok(ProcessCommandSpec {
                program: config.binaries.pg_ctl.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "stop".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                ],
            })
        }
    }
}

fn validate_non_empty_path(field: &str, value: &std::path::Path) -> Result<(), ProcessError> {
    if value.as_os_str().is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, fs, path::PathBuf, time::Duration};

    use tokio::{
        sync::mpsc,
        time::{sleep, Instant},
    };

    use crate::{
        config::{BinaryPaths, ProcessConfig},
        process::{
            jobs::{
                ActiveJob, BootstrapSpec, CancelReason, DemoteSpec, FencingSpec, NoopCommandRunner,
                PgRewindSpec, ProcessCommandRunner, ProcessError, ProcessExit, ProcessHandle,
                PromoteSpec, RestartPostgresSpec, ShutdownMode, StartPostgresSpec,
                StopPostgresSpec,
            },
            state::{
                JobOutcome, ProcessJobKind, ProcessJobRequest, ProcessState, ProcessWorkerCtx,
            },
            worker::{
                can_accept_job, cancel_active_job, start_job, step_once, tick_active_job,
                TokioCommandRunner,
            },
        },
        state::{new_state_channel, JobId, UnixMillis, WorkerError, WorkerStatus},
        test_harness::{
            binaries::require_pg16_process_binaries_for_real_tests, namespace::NamespaceGuard,
            ports::allocate_ports,
        },
    };

    struct FakeHandle {
        polls: VecDeque<Result<Option<ProcessExit>, ProcessError>>,
        cancel_result: Result<(), ProcessError>,
    }

    impl ProcessHandle for FakeHandle {
        fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError> {
            match self.polls.pop_front() {
                Some(result) => result,
                None => Ok(None),
            }
        }

        fn cancel<'a>(
            &'a mut self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), ProcessError>> + Send + 'a>,
        > {
            let result = self.cancel_result.clone();
            Box::pin(async move { result })
        }
    }

    struct FakeRunner {
        spawn_results: VecDeque<Result<FakeHandle, ProcessError>>,
    }

    impl ProcessCommandRunner for FakeRunner {
        fn spawn(
            &mut self,
            _spec: crate::process::jobs::ProcessCommandSpec,
        ) -> Result<Box<dyn ProcessHandle>, ProcessError> {
            match self.spawn_results.pop_front() {
                Some(Ok(handle)) => Ok(Box::new(handle)),
                Some(Err(err)) => Err(err),
                None => Err(ProcessError::UnsupportedInput(
                    "fake runner exhausted spawn queue".to_string(),
                )),
            }
        }
    }

    fn sample_config() -> ProcessConfig {
        ProcessConfig {
            pg_rewind_timeout_ms: 1_000,
            bootstrap_timeout_ms: 1_000,
            fencing_timeout_ms: 1_000,
            binaries: BinaryPaths {
                postgres: PathBuf::from("/usr/bin/postgres"),
                pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                initdb: PathBuf::from("/usr/bin/initdb"),
                psql: PathBuf::from("/usr/bin/psql"),
            },
        }
    }

    fn sample_start_spec() -> StartPostgresSpec {
        StartPostgresSpec {
            data_dir: PathBuf::from("/tmp/node/data"),
            host: "127.0.0.1".to_string(),
            port: 5544,
            socket_dir: PathBuf::from("/tmp/node/socket"),
            log_file: PathBuf::from("/tmp/node/postgres.log"),
            wait_seconds: Some(1),
            timeout_ms: Some(1_000),
        }
    }

    fn queued_clock(times: Vec<u64>) -> Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send> {
        let mut queue: VecDeque<u64> = times.into_iter().collect();
        let mut last = 0_u64;
        Box::new(move || {
            if let Some(next) = queue.pop_front() {
                last = next;
                return Ok(UnixMillis(next));
            }
            Ok(UnixMillis(last))
        })
    }

    fn test_ctx(
        runner: Box<dyn ProcessCommandRunner>,
        now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    ) -> (
        ProcessWorkerCtx,
        mpsc::UnboundedSender<ProcessJobRequest>,
        crate::state::StateSubscriber<ProcessState>,
    ) {
        let initial = ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        };
        let (publisher, subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
        let (tx, rx) = mpsc::unbounded_channel();
        (
            ProcessWorkerCtx {
                poll_interval: Duration::from_millis(10),
                config: sample_config(),
                state: initial,
                publisher,
                inbox: rx,
                command_runner: runner,
                active_runtime: None,
                last_rejection: None,
                now,
            },
            tx,
            subscriber,
        )
    }

    fn running_state_for_acceptance() -> ProcessState {
        ProcessState::Running {
            worker: WorkerStatus::Running,
            active: ActiveJob {
                id: JobId("job-running".to_string()),
                kind: crate::process::jobs::ActiveJobKind::StartPostgres,
                started_at: UnixMillis(1),
                deadline_at: UnixMillis(2),
            },
        }
    }

    fn outcome_id(outcome: &JobOutcome) -> &JobId {
        match outcome {
            JobOutcome::Success { id, .. }
            | JobOutcome::Failure { id, .. }
            | JobOutcome::Timeout { id, .. }
            | JobOutcome::Cancelled { id, .. } => id,
        }
    }

    #[test]
    fn can_accept_job_is_true_only_when_idle() {
        let idle = ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        };
        assert!(can_accept_job(&idle));
        assert!(!can_accept_job(&running_state_for_acceptance()));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn start_job_when_idle_transitions_to_running_and_publishes() {
        let runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut ctx, tx, subscriber) = test_ctx(Box::new(runner), queued_clock(vec![10, 11]));

        let send_result = tx.send(ProcessJobRequest {
            id: JobId("job-1".to_string()),
            kind: ProcessJobKind::StartPostgres(sample_start_spec()),
        });
        assert!(send_result.is_ok());

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));

        assert!(matches!(&ctx.state, ProcessState::Running { .. }));
        if let ProcessState::Running { active, .. } = &ctx.state {
            assert_eq!(active.id, JobId("job-1".to_string()));
        }

        let published = subscriber.latest();
        assert!(matches!(published.value, ProcessState::Running { .. }));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rejecting_new_job_while_active_records_deterministic_rejection() {
        let runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None), Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut ctx, tx, _subscriber) = test_ctx(Box::new(runner), queued_clock(vec![1, 2, 3, 4]));

        assert!(tx
            .send(ProcessJobRequest {
                id: JobId("job-a".to_string()),
                kind: ProcessJobKind::StartPostgres(sample_start_spec()),
            })
            .is_ok());
        assert_eq!(step_once(&mut ctx).await, Ok(()));
        assert!(matches!(&ctx.state, ProcessState::Running { .. }));

        assert!(tx
            .send(ProcessJobRequest {
                id: JobId("job-b".to_string()),
                kind: ProcessJobKind::StopPostgres(StopPostgresSpec {
                    data_dir: PathBuf::from("/tmp/node/data"),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10),
                }),
            })
            .is_ok());
        assert_eq!(step_once(&mut ctx).await, Ok(()));

        assert!(matches!(&ctx.state, ProcessState::Running { .. }));
        assert!(ctx.last_rejection.is_some());
        if let Some(rejection) = &ctx.last_rejection {
            assert_eq!(rejection.id, JobId("job-b".to_string()));
            assert_eq!(rejection.error, ProcessError::Busy);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tick_active_job_maps_success_failure_and_timeout_outcomes() {
        let success_runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(Some(ProcessExit::Success))]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut success_ctx, success_tx, _success_subscriber) =
            test_ctx(Box::new(success_runner), queued_clock(vec![1, 2, 3]));
        assert!(success_tx
            .send(ProcessJobRequest {
                id: JobId("job-success".to_string()),
                kind: ProcessJobKind::StartPostgres(sample_start_spec()),
            })
            .is_ok());
        assert_eq!(step_once(&mut success_ctx).await, Ok(()));
        assert!(matches!(
            &success_ctx.state,
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { .. }),
                ..
            }
        ));
        if let ProcessState::Idle {
            last_outcome: Some(JobOutcome::Success { id, .. }),
            ..
        } = &success_ctx.state
        {
            assert_eq!(id, &JobId("job-success".to_string()));
        }

        let failure_runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(Some(ProcessExit::Failure { code: Some(4) }))]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut failure_ctx, failure_tx, _failure_subscriber) =
            test_ctx(Box::new(failure_runner), queued_clock(vec![10, 11, 12]));
        assert!(failure_tx
            .send(ProcessJobRequest {
                id: JobId("job-failure".to_string()),
                kind: ProcessJobKind::StartPostgres(sample_start_spec()),
            })
            .is_ok());
        assert_eq!(step_once(&mut failure_ctx).await, Ok(()));
        assert!(matches!(
            &failure_ctx.state,
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Failure {
                    error: ProcessError::EarlyExit { .. },
                    ..
                }),
                ..
            }
        ));
        if let ProcessState::Idle {
            last_outcome:
                Some(JobOutcome::Failure {
                    id,
                    error: ProcessError::EarlyExit { code },
                    ..
                }),
            ..
        } = &failure_ctx.state
        {
            assert_eq!(id, &JobId("job-failure".to_string()));
            assert_eq!(*code, Some(4));
        }

        let timeout_runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None), Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut timeout_ctx, timeout_tx, _timeout_subscriber) =
            test_ctx(Box::new(timeout_runner), queued_clock(vec![20, 40, 41]));
        assert!(timeout_tx
            .send(ProcessJobRequest {
                id: JobId("job-timeout".to_string()),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    timeout_ms: Some(1),
                    ..sample_start_spec()
                }),
            })
            .is_ok());
        assert_eq!(step_once(&mut timeout_ctx).await, Ok(()));
        assert!(matches!(
            &timeout_ctx.state,
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Timeout { .. }),
                ..
            }
        ));
        if let ProcessState::Idle {
            last_outcome: Some(JobOutcome::Timeout { id, .. }),
            ..
        } = &timeout_ctx.state
        {
            assert_eq!(id, &JobId("job-timeout".to_string()));
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancel_active_job_emits_cancelled_and_is_noop_when_idle() {
        let runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut ctx, _tx, _subscriber) =
            test_ctx(Box::new(runner), queued_clock(vec![1, 2, 3, 4]));

        assert_eq!(
            start_job(
                &mut ctx,
                ProcessJobRequest {
                    id: JobId("job-direct".to_string()),
                    kind: ProcessJobKind::StartPostgres(sample_start_spec()),
                },
            )
            .await,
            Ok(())
        );

        assert_eq!(
            cancel_active_job(&mut ctx, CancelReason::Shutdown).await,
            Ok(())
        );
        assert!(matches!(
            &ctx.state,
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Cancelled { .. }),
                ..
            }
        ));
        if let ProcessState::Idle {
            last_outcome: Some(JobOutcome::Cancelled { id, .. }),
            ..
        } = &ctx.state
        {
            assert_eq!(id, &JobId("job-direct".to_string()));
        }

        assert_eq!(
            cancel_active_job(&mut ctx, CancelReason::Superseded).await,
            Ok(())
        );
    }

    fn pg16_binaries() -> Result<Option<BinaryPaths>, WorkerError> {
        require_pg16_process_binaries_for_real_tests()
            .map_err(|err| WorkerError::Message(format!("pg16 binary lookup failed: {err}")))
    }

    fn real_config(binaries: BinaryPaths) -> ProcessConfig {
        ProcessConfig {
            pg_rewind_timeout_ms: 20_000,
            bootstrap_timeout_ms: 40_000,
            fencing_timeout_ms: 20_000,
            binaries,
        }
    }

    fn real_ctx(
        config: ProcessConfig,
    ) -> (
        ProcessWorkerCtx,
        mpsc::UnboundedSender<ProcessJobRequest>,
        crate::state::StateSubscriber<ProcessState>,
    ) {
        let initial = ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        };
        let (publisher, subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
        let (tx, rx) = mpsc::unbounded_channel();
        (
            ProcessWorkerCtx {
                poll_interval: Duration::from_millis(50),
                config,
                state: initial,
                publisher,
                inbox: rx,
                command_runner: Box::new(TokioCommandRunner),
                active_runtime: None,
                last_rejection: None,
                now: Box::new(super::system_now_unix_millis),
            },
            tx,
            subscriber,
        )
    }

    async fn wait_for_outcome(
        ctx: &mut ProcessWorkerCtx,
        expected_job: &JobId,
        timeout: Duration,
    ) -> Result<JobOutcome, WorkerError> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            step_once(ctx).await?;
            if let ProcessState::Idle {
                last_outcome: Some(outcome),
                ..
            } = &ctx.state
            {
                if outcome_id(outcome) == expected_job {
                    return Ok(outcome.clone());
                }
            }
            sleep(Duration::from_millis(50)).await;
        }

        Err(WorkerError::Message(format!(
            "timed out waiting for process outcome for {}",
            expected_job.0
        )))
    }

    async fn submit_job_and_wait(
        tx: &mpsc::UnboundedSender<ProcessJobRequest>,
        ctx: &mut ProcessWorkerCtx,
        id: &str,
        kind: ProcessJobKind,
        timeout: Duration,
    ) -> Result<JobOutcome, WorkerError> {
        tx.send(ProcessJobRequest {
            id: JobId(id.to_string()),
            kind,
        })
        .map_err(|err| WorkerError::Message(format!("send process job failed: {err}")))?;

        wait_for_outcome(ctx, &JobId(id.to_string()), timeout).await
    }

    struct RealProcessFixture {
        ctx: ProcessWorkerCtx,
        tx: mpsc::UnboundedSender<ProcessJobRequest>,
        subscriber: crate::state::StateSubscriber<ProcessState>,
        _guard: NamespaceGuard,
        data_dir: PathBuf,
        socket_dir: PathBuf,
        log_file: PathBuf,
        port: u16,
    }

    impl RealProcessFixture {
        async fn bootstrap_and_start(
            binaries: BinaryPaths,
            ns_name: &str,
        ) -> Result<Self, WorkerError> {
            let guard = NamespaceGuard::new(ns_name)
                .map_err(|err| WorkerError::Message(format!("namespace create failed: {err}")))?;
            let namespace = guard
                .namespace()
                .map_err(|err| WorkerError::Message(format!("namespace lookup failed: {err}")))?
                .clone();

            let reservation = allocate_ports(1)
                .map_err(|err| WorkerError::Message(format!("port allocation failed: {err}")))?;
            let port = reservation.as_slice()[0];

            let data_dir = namespace.child_dir("process/node-a/data");
            let socket_dir = namespace.child_dir("process/node-a/socket");
            let log_dir = namespace.child_dir("logs/process-node-a");
            let log_file = log_dir.join("postgres.log");
            fs::create_dir_all(&socket_dir)
                .map_err(|err| WorkerError::Message(format!("socket dir create failed: {err}")))?;
            fs::create_dir_all(&log_dir)
                .map_err(|err| WorkerError::Message(format!("log dir create failed: {err}")))?;

            let (ctx, tx, subscriber) = real_ctx(real_config(binaries));
            let mut fixture = Self {
                ctx,
                tx,
                subscriber,
                _guard: guard,
                data_dir,
                socket_dir,
                log_file,
                port,
            };

            let bootstrap = fixture
                .submit_job_and_wait(
                    "bootstrap",
                    ProcessJobKind::Bootstrap(BootstrapSpec {
                        data_dir: fixture.data_dir.clone(),
                        timeout_ms: Some(30_000),
                    }),
                    Duration::from_secs(40),
                )
                .await?;
            if !matches!(bootstrap, JobOutcome::Success { .. }) {
                return Err(WorkerError::Message(format!(
                    "bootstrap setup failed: {bootstrap:?}"
                )));
            }

            // Release the reserved port immediately before starting postgres so
            // `pg_ctl` can bind the requested port.
            drop(reservation);
            let start = fixture
                .submit_job_and_wait(
                    "start",
                    ProcessJobKind::StartPostgres(StartPostgresSpec {
                        data_dir: fixture.data_dir.clone(),
                        host: "127.0.0.1".to_string(),
                        port: fixture.port,
                        socket_dir: fixture.socket_dir.clone(),
                        log_file: fixture.log_file.clone(),
                        wait_seconds: Some(20),
                        timeout_ms: Some(30_000),
                    }),
                    Duration::from_secs(40),
                )
                .await?;
            if !matches!(start, JobOutcome::Success { .. }) {
                return Err(WorkerError::Message(format!(
                    "start setup failed: {start:?}"
                )));
            }

            Ok(fixture)
        }

        async fn submit_job_and_wait(
            &mut self,
            id: &str,
            kind: ProcessJobKind,
            timeout: Duration,
        ) -> Result<JobOutcome, WorkerError> {
            let expected_job = JobId(id.to_string());
            self.tx
                .send(ProcessJobRequest {
                    id: expected_job.clone(),
                    kind,
                })
                .map_err(|err| WorkerError::Message(format!("send process job failed: {err}")))?;

            self.wait_for_outcome(&expected_job, timeout).await
        }

        async fn wait_for_outcome(
            &mut self,
            expected_job: &JobId,
            timeout: Duration,
        ) -> Result<JobOutcome, WorkerError> {
            let start = Instant::now();
            let mut last_snapshot = self.subscriber.latest();
            while start.elapsed() < timeout {
                let _before = self.subscriber.latest();
                step_once(&mut self.ctx).await?;
                last_snapshot = self.subscriber.latest();
                if let ProcessState::Idle {
                    last_outcome: Some(outcome),
                    ..
                } = &self.ctx.state
                {
                    if outcome_id(outcome) == expected_job {
                        return Ok(outcome.clone());
                    }
                }
                sleep(Duration::from_millis(50)).await;
            }

            Err(WorkerError::Message(format!(
                "timed out waiting for process outcome for {} (last snapshot: {:?})",
                expected_job.0, last_snapshot
            )))
        }
    }

    fn assert_success_outcome(label: &str, outcome: &JobOutcome) -> Result<(), WorkerError> {
        match outcome {
            JobOutcome::Success { .. } => Ok(()),
            other => Err(WorkerError::Message(format!(
                "expected {label} to succeed, got: {other:?}"
            ))),
        }
    }

    fn assert_promote_outcome(outcome: &JobOutcome) -> Result<(), WorkerError> {
        match outcome {
            JobOutcome::Success { .. } => Ok(()),
            JobOutcome::Failure {
                error: ProcessError::EarlyExit { code: Some(1) },
                ..
            } => Ok(()),
            JobOutcome::Failure { error, .. } => Err(WorkerError::Message(format!(
                "expected promote success or standby-state early-exit, got failure: {error:?}"
            ))),
            other => Err(WorkerError::Message(format!(
                "expected promote success or standby-state early-exit, got: {other:?}"
            ))),
        }
    }

    fn assert_shutdown_cleanup_outcome(
        label: &str,
        outcome: &JobOutcome,
    ) -> Result<(), WorkerError> {
        match outcome {
            JobOutcome::Success { .. } => Ok(()),
            JobOutcome::Failure {
                error:
                    ProcessError::EarlyExit {
                        code: Some(1) | Some(3),
                    },
                ..
            } => Ok(()),
            JobOutcome::Failure { error, .. } => Err(WorkerError::Message(format!(
                "{label} failure is not an expected already-stopped shutdown result: {error:?}"
            ))),
            other => Err(WorkerError::Message(format!(
                "expected {label} cleanup success or already-stopped early-exit, got: {other:?}"
            ))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_bootstrap_job_executes_initdb() -> Result<(), WorkerError> {
        let binaries = match pg16_binaries()? {
            Some(paths) => paths,
            None => return Ok(()),
        };
        let guard = NamespaceGuard::new("process-bootstrap")
            .map_err(|err| WorkerError::Message(format!("namespace setup failed: {err}")))?;
        let namespace = guard
            .namespace()
            .map_err(|err| WorkerError::Message(format!("namespace lookup failed: {err}")))?;

        let data_dir = namespace.child_dir("process/node-a/data");
        let (mut ctx, tx, _sub) = real_ctx(real_config(binaries));
        let outcome = submit_job_and_wait(
            &tx,
            &mut ctx,
            "bootstrap-1",
            ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: data_dir.clone(),
                timeout_ms: Some(30_000),
            }),
            Duration::from_secs(40),
        )
        .await;

        match outcome {
            Ok(JobOutcome::Success { .. }) => {
                assert!(data_dir.join("PG_VERSION").exists());
                Ok(())
            }
            Ok(other) => Err(WorkerError::Message(format!(
                "expected bootstrap success, got: {other:?}"
            ))),
            Err(err) => Err(WorkerError::Message(format!("bootstrap job failed: {err}"))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_pg_rewind_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = match pg16_binaries()? {
            Some(paths) => paths,
            None => return Ok(()),
        };

        let guard = NamespaceGuard::new("process-rewind")
            .map_err(|err| WorkerError::Message(format!("namespace setup failed: {err}")))?;
        let namespace = guard
            .namespace()
            .map_err(|err| WorkerError::Message(format!("namespace lookup failed: {err}")))?;

        let data_dir = namespace.child_dir("process/rewind/target");
        fs::create_dir_all(&data_dir)
            .map_err(|err| WorkerError::Message(format!("create rewind data dir failed: {err}")))?;

        let (mut ctx, tx, _sub) = real_ctx(real_config(binaries));
        let outcome = submit_job_and_wait(
            &tx,
            &mut ctx,
            "rewind-1",
            ProcessJobKind::PgRewind(PgRewindSpec {
                target_data_dir: data_dir,
                source_conninfo: "host=127.0.0.1 port=9 user=postgres dbname=postgres".to_string(),
                timeout_ms: Some(5_000),
            }),
            Duration::from_secs(10),
        )
        .await;

        match outcome {
            Ok(JobOutcome::Failure {
                error: ProcessError::EarlyExit { code: Some(_) },
                ..
            }) => Ok(()),
            Ok(JobOutcome::Failure { error, .. }) => Err(WorkerError::Message(format!(
                "expected rewind early-exit failure for invalid source, got failure: {error:?}"
            ))),
            Ok(other) => Err(WorkerError::Message(format!(
                "expected rewind early-exit failure for invalid source, got: {other:?}"
            ))),
            Err(err) => Err(WorkerError::Message(format!(
                "rewind job wait failed: {err}"
            ))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_promote_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = match pg16_binaries()? {
            Some(paths) => paths,
            None => return Ok(()),
        };
        let mut fixture =
            RealProcessFixture::bootstrap_and_start(binaries, "process-promote").await?;

        let promote = fixture
            .submit_job_and_wait(
                "promote",
                ProcessJobKind::Promote(PromoteSpec {
                    data_dir: fixture.data_dir.clone(),
                    wait_seconds: Some(10),
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_promote_outcome(&promote)?;

        let stop = fixture
            .submit_job_and_wait(
                "stop-after-promote",
                ProcessJobKind::StopPostgres(StopPostgresSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_success_outcome("stop-after-promote", &stop)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_demote_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = match pg16_binaries()? {
            Some(paths) => paths,
            None => return Ok(()),
        };
        let mut fixture =
            RealProcessFixture::bootstrap_and_start(binaries, "process-demote").await?;

        let outcome = fixture
            .submit_job_and_wait(
                "demote",
                ProcessJobKind::Demote(DemoteSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_success_outcome("demote", &outcome)?;

        let cleanup = fixture
            .submit_job_and_wait(
                "stop-after-demote",
                ProcessJobKind::StopPostgres(StopPostgresSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_shutdown_cleanup_outcome("stop-after-demote", &cleanup)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_start_and_stop_jobs_execute_binary_paths() -> Result<(), WorkerError> {
        let binaries = match pg16_binaries()? {
            Some(paths) => paths,
            None => return Ok(()),
        };
        let mut fixture =
            RealProcessFixture::bootstrap_and_start(binaries, "process-start-stop").await?;

        let stop = fixture
            .submit_job_and_wait(
                "stop",
                ProcessJobKind::StopPostgres(StopPostgresSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        if !matches!(stop, JobOutcome::Success { .. }) {
            return Err(WorkerError::Message(format!(
                "expected stop success, got: {stop:?}"
            )));
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_restart_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = match pg16_binaries()? {
            Some(paths) => paths,
            None => return Ok(()),
        };
        let mut fixture =
            RealProcessFixture::bootstrap_and_start(binaries, "process-restart").await?;

        let restart = fixture
            .submit_job_and_wait(
                "restart",
                ProcessJobKind::RestartPostgres(RestartPostgresSpec {
                    data_dir: fixture.data_dir.clone(),
                    host: "127.0.0.1".to_string(),
                    port: fixture.port,
                    socket_dir: fixture.socket_dir.clone(),
                    log_file: fixture.log_file.clone(),
                    mode: ShutdownMode::Fast,
                    wait_seconds: Some(20),
                    timeout_ms: Some(20_000),
                }),
                Duration::from_secs(30),
            )
            .await?;
        assert_success_outcome("restart", &restart)?;

        let stop = fixture
            .submit_job_and_wait(
                "stop-after-restart",
                ProcessJobKind::StopPostgres(StopPostgresSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_success_outcome("stop-after-restart", &stop)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_fencing_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = match pg16_binaries()? {
            Some(paths) => paths,
            None => return Ok(()),
        };
        let mut fixture =
            RealProcessFixture::bootstrap_and_start(binaries, "process-fencing").await?;

        let outcome = fixture
            .submit_job_and_wait(
                "fence",
                ProcessJobKind::Fencing(FencingSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Immediate,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_success_outcome("fence", &outcome)?;

        let cleanup = fixture
            .submit_job_and_wait(
                "stop-after-fencing",
                ProcessJobKind::StopPostgres(StopPostgresSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_shutdown_cleanup_outcome("stop-after-fencing", &cleanup)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn start_job_returns_channel_closed_when_all_subscribers_are_dropped() {
        let runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let initial = ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        };
        let (publisher, subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
        drop(subscriber);
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut ctx = ProcessWorkerCtx {
            poll_interval: Duration::from_millis(10),
            config: sample_config(),
            state: initial,
            publisher,
            inbox: rx,
            command_runner: Box::new(runner),
            active_runtime: None,
            last_rejection: None,
            now: queued_clock(vec![1, 2]),
        };

        let result = start_job(
            &mut ctx,
            ProcessJobRequest {
                id: JobId("job-no-subscriber".to_string()),
                kind: ProcessJobKind::StartPostgres(sample_start_spec()),
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(WorkerError::Message(message)) if message.contains("state channel is closed")
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tick_active_job_is_safe_when_no_runtime() {
        let (mut ctx, _tx, _sub) = test_ctx(Box::new(NoopCommandRunner), queued_clock(vec![1]));
        assert_eq!(tick_active_job(&mut ctx).await, Ok(()));
    }
}
