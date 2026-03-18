// use std::{
//     collections::{BTreeMap, BTreeSet},
//     fs::{self, OpenOptions},
//     io::Write as _,
//     path::{Path, PathBuf},
//     sync::{
//         atomic::{AtomicBool, Ordering},
//         Arc, Mutex,
//     },
//     thread::{self, JoinHandle},
//     time::{Duration, SystemTime, UNIX_EPOCH},
// };
//
// use pgtuskmaster_rust::{
//     api::NodeState,
//     ha::types::{AuthorityProjection, PublicationState},
//     pginfo::state::PgInfoState,
// };
// use serde::Serialize;
//
// use crate::support::{
//     error::{HarnessError, Result},
//     observer::{
//         pgtm::{
//             ClusterStateObservation, MemberCommandOutcome, PgtmObserver, PostgresRoutingTarget,
//         },
//         sql::SqlObserver,
//     },
//     topology::ClusterMember,
// };
//
// const PRIMARY_COUNT_VIOLATION_ARTIFACT_NAME: &str = "primary-count-invariant-violation.json";
// const WRITE_CONVERGENCE_EVENTS_ARTIFACT_NAME: &str = "write-convergence-invariant-events.jsonl";
// const WRITE_CONVERGENCE_SUMMARY_ARTIFACT_NAME: &str = "write-convergence-invariant-summary.json";
// const WRITE_CONVERGENCE_VIOLATION_ARTIFACT_NAME: &str =
//     "write-convergence-invariant-violation.json";
// const WRITE_CONVERGENCE_TABLE_NAME: &str = "public.ha_write_convergence_invariant";
//
// #[derive(Debug)]
// pub struct PrimaryCountInvariantRunner {
//     shared: Arc<SharedPrimaryCountInvariantState>,
//     join_handle: Option<JoinHandle<Result<()>>>,
// }
//
// #[derive(Debug)]
// struct SharedPrimaryCountInvariantState {
//     stop_requested: AtomicBool,
//     failure: Mutex<Option<PrimaryCountInvariantFailure>>,
// }
//
// #[derive(Clone, Debug)]
// enum PrimaryCountInvariantFailure {
//     Violation(PrimaryCountInvariantViolation),
//     RunnerError(String),
// }
//
// #[derive(Clone, Debug)]
// struct PrimaryCountInvariantViolation {
//     artifact_path: PathBuf,
//     sample: PrimaryCountSample,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct PrimaryCountSample {
//     observed_at_ms: u128,
//     allowed_primary_counts: [usize; 2],
//     primary_count: usize,
//     members: Vec<MemberPrimaryCountSample>,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct MemberPrimaryCountSample {
//     member: String,
//     self_report: MemberSelfReport,
// }
//
// #[derive(Debug)]
// pub struct WriteConvergenceInvariantRunner {
//     shared: Arc<SharedWriteConvergenceInvariantState>,
//     join_handle: Option<JoinHandle<Result<()>>>,
// }
//
// #[derive(Debug)]
// struct SharedWriteConvergenceInvariantState {
//     stop_requested: AtomicBool,
//     failure: Mutex<Option<WriteConvergenceInvariantFailure>>,
// }
//
// #[derive(Clone, Debug)]
// enum WriteConvergenceInvariantFailure {
//     Violation(WriteConvergenceInvariantViolation),
//     RunnerError(String),
// }
//
// #[derive(Clone, Debug)]
// struct WriteConvergenceInvariantViolation {
//     artifact_path: PathBuf,
//     summary: WriteConvergenceSummary,
// }
//
// #[derive(Clone, Debug)]
// struct WriteConvergenceArtifacts {
//     events_path: PathBuf,
//     summary_path: PathBuf,
//     violation_path: PathBuf,
// }
//
// #[derive(Clone, Debug)]
// struct WriteConvergenceTracker {
//     convergence_window: Duration,
//     next_target_index: usize,
//     next_token_index: u64,
//     accepted_count: usize,
//     rejected_count: usize,
//     converged_count: usize,
//     pending: BTreeMap<String, PendingAcceptedWrite>,
// }
//
// #[derive(Clone, Debug, Serialize)]
// #[serde(tag = "kind", rename_all = "snake_case")]
// enum WriteConvergenceEvent {
//     Accepted(AcceptedWriteRecord),
//     Rejected(RejectedWriteRecord),
//     Converged(ConvergedWriteRecord),
// }
//
// #[derive(Clone, Debug)]
// enum WriteAttemptOutcome {
//     Accepted(AcceptedWriteRecord),
//     Rejected(RejectedWriteRecord),
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct AcceptedWriteRecord {
//     token: String,
//     target_member: String,
//     target_self_report: MemberSelfReport,
//     accepted_at_ms: u128,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct RejectedWriteRecord {
//     token: String,
//     target_member: String,
//     target_self_report: MemberSelfReport,
//     rejected_at_ms: u128,
//     reason: String,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct ConvergedWriteRecord {
//     token: String,
//     target_member: String,
//     accepted_at_ms: u128,
//     converged_at_ms: u128,
//     lag_ms: u128,
//     visibility: Vec<MemberTokenVisibility>,
// }
//
// #[derive(Clone, Debug)]
// struct PendingAcceptedWrite {
//     accepted: AcceptedWriteRecord,
//     visibility: Vec<MemberTokenVisibility>,
// }
//
// #[derive(Clone, Debug)]
// struct MemberTokenSnapshot {
//     member: ClusterMember,
//     observation: MemberTokenObservation,
// }
//
// #[derive(Clone, Debug)]
// enum MemberTokenObservation {
//     VisibleTokens(BTreeSet<String>),
//     QueryFailed(String),
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct WriteConvergenceSummary {
//     observed_at_ms: u128,
//     convergence_window_ms: u128,
//     counts: WriteConvergenceCounts,
//     pending: Vec<PendingAcceptedWriteSummary>,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct WriteConvergenceCounts {
//     accepted: usize,
//     rejected: usize,
//     converged: usize,
//     pending: usize,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct PendingAcceptedWriteSummary {
//     token: String,
//     target_member: String,
//     accepted_at_ms: u128,
//     age_ms: u128,
//     visibility: Vec<MemberTokenVisibility>,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// struct MemberTokenVisibility {
//     member: String,
//     state: TokenVisibilityState,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// #[serde(tag = "kind", rename_all = "snake_case")]
// enum TokenVisibilityState {
//     Visible,
//     Missing,
//     QueryFailed { message: String },
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
// #[serde(tag = "kind", rename_all = "snake_case")]
// enum MemberSelfReport {
//     Primary,
//     NotPrimary { pg_state: NonPrimaryPgState },
//     CommandFailed { message: String },
// }
//
// #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
// #[serde(rename_all = "snake_case")]
// enum NonPrimaryPgState {
//     Replica,
//     Unknown,
// }
//
// impl PrimaryCountInvariantRunner {
//     pub fn start(
//         observer: PgtmObserver,
//         artifacts_dir: PathBuf,
//         poll_interval: Duration,
//     ) -> Result<Self> {
//         let shared = Arc::new(SharedPrimaryCountInvariantState::new());
//         let thread_shared = Arc::clone(&shared);
//         let thread_name = "ha-primary-count-invariant".to_string();
//         let join_handle = thread::Builder::new()
//             .name(thread_name.clone())
//             .spawn(move || {
//                 match run_primary_count_invariant_loop(
//                     observer,
//                     artifacts_dir,
//                     poll_interval,
//                     &thread_shared,
//                 ) {
//                     Ok(()) => Ok(()),
//                     Err(err) => {
//                         thread_shared.store_failure(PrimaryCountInvariantFailure::RunnerError(
//                             err.to_string(),
//                         ))?;
//                         Err(err)
//                     }
//                 }
//             })
//             .map_err(|err| {
//                 HarnessError::message(format!(
//                     "failed to spawn `{thread_name}` background runner: {err}"
//                 ))
//             })?;
//
//         Ok(Self {
//             shared,
//             join_handle: Some(join_handle),
//         })
//     }
//
//     pub fn ensure_healthy(&self) -> Result<()> {
//         self.shared.load_failure()?.map_or(Ok(()), |failure| {
//             Err(HarnessError::message(failure.message()))
//         })
//     }
//
//     pub fn stop(&mut self) -> Result<()> {
//         self.shared.stop_requested.store(true, Ordering::SeqCst);
//         let joined = self.join_handle.take().map(|handle| {
//             handle.join().map_err(|_| {
//                 HarnessError::message("primary-count invariant runner thread panicked")
//             })
//         });
//
//         if let Some(result) = joined.transpose()? {
//             result?;
//         }
//
//         self.ensure_healthy()
//     }
// }
//
// impl WriteConvergenceInvariantRunner {
//     pub fn start(
//         observer: PgtmObserver,
//         sql: SqlObserver,
//         artifacts_dir: PathBuf,
//         poll_interval: Duration,
//         convergence_window: Duration,
//     ) -> Result<Self> {
//         let shared = Arc::new(SharedWriteConvergenceInvariantState::new());
//         let thread_shared = Arc::clone(&shared);
//         let thread_name = "ha-write-convergence-invariant".to_string();
//         let join_handle = thread::Builder::new()
//             .name(thread_name.clone())
//             .spawn(move || {
//                 match run_write_convergence_invariant_loop(
//                     observer,
//                     sql,
//                     artifacts_dir,
//                     poll_interval,
//                     convergence_window,
//                     &thread_shared,
//                 ) {
//                     Ok(()) => Ok(()),
//                     Err(err) => {
//                         thread_shared.store_failure(
//                             WriteConvergenceInvariantFailure::RunnerError(err.to_string()),
//                         )?;
//                         Err(err)
//                     }
//                 }
//             })
//             .map_err(|err| {
//                 HarnessError::message(format!(
//                     "failed to spawn `{thread_name}` background runner: {err}"
//                 ))
//             })?;
//
//         Ok(Self {
//             shared,
//             join_handle: Some(join_handle),
//         })
//     }
//
//     pub fn ensure_healthy(&self) -> Result<()> {
//         self.shared.load_failure()?.map_or(Ok(()), |failure| {
//             Err(HarnessError::message(failure.message()))
//         })
//     }
//
//     pub fn stop(&mut self) -> Result<()> {
//         self.shared.stop_requested.store(true, Ordering::SeqCst);
//         let joined = self.join_handle.take().map(|handle| {
//             handle.join().map_err(|_| {
//                 HarnessError::message("write-convergence invariant runner thread panicked")
//             })
//         });
//
//         if let Some(result) = joined.transpose()? {
//             result?;
//         }
//
//         self.ensure_healthy()
//     }
// }
//
// impl SharedPrimaryCountInvariantState {
//     fn new() -> Self {
//         Self {
//             stop_requested: AtomicBool::new(false),
//             failure: Mutex::new(None),
//         }
//     }
//
//     fn stop_requested(&self) -> bool {
//         self.stop_requested.load(Ordering::SeqCst)
//     }
//
//     fn load_failure(&self) -> Result<Option<PrimaryCountInvariantFailure>> {
//         self.failure
//             .lock()
//             .map(|failure| failure.clone())
//             .map_err(|_| HarnessError::message("primary-count invariant mutex was poisoned"))
//     }
//
//     fn store_failure(&self, failure: PrimaryCountInvariantFailure) -> Result<()> {
//         self.failure
//             .lock()
//             .map(|mut slot| {
//                 if slot.is_none() {
//                     *slot = Some(failure);
//                 }
//             })
//             .map_err(|_| HarnessError::message("primary-count invariant mutex was poisoned"))
//     }
// }
//
// impl SharedWriteConvergenceInvariantState {
//     fn new() -> Self {
//         Self {
//             stop_requested: AtomicBool::new(false),
//             failure: Mutex::new(None),
//         }
//     }
//
//     fn stop_requested(&self) -> bool {
//         self.stop_requested.load(Ordering::SeqCst)
//     }
//
//     fn load_failure(&self) -> Result<Option<WriteConvergenceInvariantFailure>> {
//         self.failure
//             .lock()
//             .map(|failure| failure.clone())
//             .map_err(|_| HarnessError::message("write-convergence invariant mutex was poisoned"))
//     }
//
//     fn store_failure(&self, failure: WriteConvergenceInvariantFailure) -> Result<()> {
//         self.failure
//             .lock()
//             .map(|mut slot| {
//                 if slot.is_none() {
//                     *slot = Some(failure);
//                 }
//             })
//             .map_err(|_| HarnessError::message("write-convergence invariant mutex was poisoned"))
//     }
// }
//
// impl PrimaryCountInvariantFailure {
//     fn message(&self) -> String {
//         match self {
//             Self::Violation(violation) => format!(
//                 "primary-count invariant violated: {}. structured sample: {}",
//                 violation.sample.summary(),
//                 violation.artifact_path.display()
//             ),
//             Self::RunnerError(message) => {
//                 format!("primary-count invariant runner failed: {message}")
//             }
//         }
//     }
// }
//
// impl WriteConvergenceInvariantFailure {
//     fn message(&self) -> String {
//         match self {
//             Self::Violation(violation) => format!(
//                 "write-convergence invariant violated: {}. structured summary: {}",
//                 violation.summary.summary(),
//                 violation.artifact_path.display()
//             ),
//             Self::RunnerError(message) => {
//                 format!("write-convergence invariant runner failed: {message}")
//             }
//         }
//     }
// }
//
// impl PrimaryCountInvariantViolation {
//     fn new(artifact_path: PathBuf, sample: PrimaryCountSample) -> Self {
//         Self {
//             artifact_path,
//             sample,
//         }
//     }
// }
//
// impl WriteConvergenceInvariantViolation {
//     fn new(artifact_path: PathBuf, summary: WriteConvergenceSummary) -> Self {
//         Self {
//             artifact_path,
//             summary,
//         }
//     }
// }
//
// impl PrimaryCountSample {
//     fn from_observation(observation: &ClusterStateObservation) -> Result<Self> {
//         let members = observation
//             .members()
//             .iter()
//             .map(MemberPrimaryCountSample::from_observation)
//             .collect::<Result<Vec<_>>>()?;
//
//         Ok(Self {
//             observed_at_ms: timestamp_millis()?,
//             allowed_primary_counts: [0, 1],
//             primary_count: members
//                 .iter()
//                 .filter(|member| member.self_report.is_primary())
//                 .count(),
//             members,
//         })
//     }
//
//     fn violates_allowed_primary_counts(&self) -> bool {
//         !self.allowed_primary_counts.contains(&self.primary_count)
//     }
//
//     fn summary(&self) -> String {
//         format!(
//             "observed {} self-reported primaries ({})",
//             self.primary_count,
//             self.members
//                 .iter()
//                 .map(MemberPrimaryCountSample::summary)
//                 .collect::<Vec<_>>()
//                 .join(", ")
//         )
//     }
// }
//
// impl MemberPrimaryCountSample {
//     fn from_observation(
//         observation: &crate::support::observer::pgtm::MemberStateObservation,
//     ) -> Result<Self> {
//         Ok(Self {
//             member: observation.member.service_name().to_string(),
//             self_report: member_self_report_from_observation(observation)?,
//         })
//     }
//
//     fn summary(&self) -> String {
//         format!("{}={}", self.member, self.self_report.summary())
//     }
// }
//
// impl WriteConvergenceArtifacts {
//     fn new(artifacts_dir: PathBuf) -> Self {
//         Self {
//             events_path: artifacts_dir.join(WRITE_CONVERGENCE_EVENTS_ARTIFACT_NAME),
//             summary_path: artifacts_dir.join(WRITE_CONVERGENCE_SUMMARY_ARTIFACT_NAME),
//             violation_path: artifacts_dir.join(WRITE_CONVERGENCE_VIOLATION_ARTIFACT_NAME),
//         }
//     }
//
//     fn append_event(&self, event: &WriteConvergenceEvent) -> Result<()> {
//         let rendered = serde_json::to_string(event).map_err(|source| HarnessError::Json {
//             context: "serializing write-convergence event".to_string(),
//             source,
//         })?;
//         append_line(self.events_path.as_path(), rendered.as_str())
//     }
//
//     fn persist_summary(&self, summary: &WriteConvergenceSummary) -> Result<()> {
//         let rendered =
//             serde_json::to_string_pretty(summary).map_err(|source| HarnessError::Json {
//                 context: "serializing write-convergence summary".to_string(),
//                 source,
//             })?;
//         write_text_file(self.summary_path.as_path(), rendered.as_str())
//     }
//
//     fn persist_violation(&self, summary: &WriteConvergenceSummary) -> Result<PathBuf> {
//         let rendered =
//             serde_json::to_string_pretty(summary).map_err(|source| HarnessError::Json {
//                 context: "serializing write-convergence violation".to_string(),
//                 source,
//             })?;
//         write_text_file(self.violation_path.as_path(), rendered.as_str())?;
//         Ok(self.violation_path.clone())
//     }
// }
//
// impl WriteConvergenceTracker {
//     fn new(convergence_window: Duration) -> Self {
//         Self {
//             convergence_window,
//             next_target_index: 0,
//             next_token_index: 0,
//             accepted_count: 0,
//             rejected_count: 0,
//             converged_count: 0,
//             pending: BTreeMap::new(),
//         }
//     }
//
//     fn next_target<'a>(
//         &mut self,
//         routing_targets: &'a [PostgresRoutingTarget],
//     ) -> Result<&'a PostgresRoutingTarget> {
//         let target_count = routing_targets.len();
//         if target_count == 0 {
//             return Err(HarnessError::message(
//                 "write-convergence invariant has no postgres routing targets",
//             ));
//         }
//         let index = self.next_target_index % target_count;
//         self.next_target_index = (index + 1) % target_count;
//         routing_targets.get(index).ok_or_else(|| {
//             HarnessError::message(format!(
//                 "write-convergence invariant target index `{index}` was out of bounds"
//             ))
//         })
//     }
//
//     fn next_non_authoritative_target<'a>(
//         &mut self,
//         routing_targets: &'a [PostgresRoutingTarget],
//         authoritative_primary: Option<ClusterMember>,
//     ) -> Result<Option<&'a PostgresRoutingTarget>> {
//         let target_count = routing_targets.len();
//         if target_count == 0 {
//             return Ok(None);
//         }
//
//         for _ in 0..target_count {
//             let target = self.next_target(routing_targets)?;
//             if Some(target.member) != authoritative_primary {
//                 return Ok(Some(target));
//             }
//         }
//
//         Ok(None)
//     }
//
//     fn next_token(&mut self, target_member: ClusterMember, attempted_at_ms: u128) -> String {
//         let sequence = self.next_token_index;
//         self.next_token_index = self.next_token_index.saturating_add(1);
//         format!(
//             "ha-write-{}-{}-{}",
//             attempted_at_ms,
//             target_member.service_name(),
//             sequence
//         )
//     }
//
//     fn record_attempt(
//         &mut self,
//         attempted_at_ms: u128,
//         outcome: WriteAttemptOutcome,
//         artifacts: &WriteConvergenceArtifacts,
//     ) -> Result<()> {
//         match outcome {
//             WriteAttemptOutcome::Accepted(record) => {
//                 self.accepted_count = self.accepted_count.saturating_add(1);
//                 let token = record.token.clone();
//                 let previous = self
//                     .pending
//                     .insert(token, PendingAcceptedWrite::new(record.clone()));
//                 if previous.is_some() {
//                     return Err(HarnessError::message(
//                         "write-convergence invariant generated a duplicate token",
//                     ));
//                 }
//                 artifacts.append_event(&WriteConvergenceEvent::Accepted(record))?;
//             }
//             WriteAttemptOutcome::Rejected(record) => {
//                 self.rejected_count = self.rejected_count.saturating_add(1);
//                 artifacts.append_event(&WriteConvergenceEvent::Rejected(record))?;
//             }
//         }
//         artifacts.persist_summary(&self.summary(attempted_at_ms))
//     }
//
//     fn reconcile_visibility(
//         &mut self,
//         observed_at_ms: u128,
//         snapshots: &[MemberTokenSnapshot],
//         artifacts: &WriteConvergenceArtifacts,
//     ) -> Result<Option<WriteConvergenceInvariantViolation>> {
//         self.pending
//             .values_mut()
//             .for_each(|pending| pending.refresh_visibility(snapshots));
//
//         let converged_tokens = self
//             .pending
//             .iter()
//             .filter_map(|(token, pending)| pending.is_converged().then_some(token.clone()))
//             .collect::<Vec<_>>();
//
//         let converged_records = converged_tokens
//             .iter()
//             .map(|token| {
//                 self.pending
//                     .remove(token.as_str())
//                     .map(|pending| pending.into_converged_record(observed_at_ms))
//                     .ok_or_else(|| {
//                         HarnessError::message(format!(
//                             "pending write `{token}` disappeared before convergence recording"
//                         ))
//                     })
//             })
//             .collect::<Result<Vec<_>>>()?;
//
//         converged_records.iter().try_for_each(|record| {
//             self.converged_count = self.converged_count.saturating_add(1);
//             artifacts.append_event(&WriteConvergenceEvent::Converged(record.clone()))
//         })?;
//
//         let summary = self.summary(observed_at_ms);
//         artifacts.persist_summary(&summary)?;
//
//         let violation = summary
//             .pending
//             .iter()
//             .any(|pending| pending.age_ms > self.convergence_window.as_millis())
//             .then(|| {
//                 artifacts.persist_violation(&summary).map(|artifact_path| {
//                     WriteConvergenceInvariantViolation::new(artifact_path, summary.clone())
//                 })
//             })
//             .transpose()?;
//
//         Ok(violation)
//     }
//
//     fn has_pending(&self) -> bool {
//         !self.pending.is_empty()
//     }
//
//     fn summary(&self, observed_at_ms: u128) -> WriteConvergenceSummary {
//         let pending = self
//             .pending
//             .values()
//             .map(|pending| pending.summary(observed_at_ms))
//             .collect::<Vec<_>>();
//
//         WriteConvergenceSummary {
//             observed_at_ms,
//             convergence_window_ms: self.convergence_window.as_millis(),
//             counts: WriteConvergenceCounts {
//                 accepted: self.accepted_count,
//                 rejected: self.rejected_count,
//                 converged: self.converged_count,
//                 pending: pending.len(),
//             },
//             pending,
//         }
//     }
// }
//
// impl PendingAcceptedWrite {
//     fn new(accepted: AcceptedWriteRecord) -> Self {
//         Self {
//             accepted,
//             visibility: Vec::new(),
//         }
//     }
//
//     fn refresh_visibility(&mut self, snapshots: &[MemberTokenSnapshot]) {
//         self.visibility = snapshots
//             .iter()
//             .map(|snapshot| MemberTokenVisibility {
//                 member: snapshot.member.service_name().to_string(),
//                 state: match &snapshot.observation {
//                     MemberTokenObservation::VisibleTokens(tokens) => {
//                         if tokens.contains(self.accepted.token.as_str()) {
//                             TokenVisibilityState::Visible
//                         } else {
//                             TokenVisibilityState::Missing
//                         }
//                     }
//                     MemberTokenObservation::QueryFailed(message) => {
//                         TokenVisibilityState::QueryFailed {
//                             message: message.clone(),
//                         }
//                     }
//                 },
//             })
//             .collect::<Vec<_>>();
//     }
//
//     fn is_converged(&self) -> bool {
//         self.visibility
//             .iter()
//             .all(|entry| matches!(entry.state, TokenVisibilityState::Visible))
//     }
//
//     fn into_converged_record(self, converged_at_ms: u128) -> ConvergedWriteRecord {
//         ConvergedWriteRecord {
//             token: self.accepted.token,
//             target_member: self.accepted.target_member,
//             accepted_at_ms: self.accepted.accepted_at_ms,
//             converged_at_ms,
//             lag_ms: converged_at_ms.saturating_sub(self.accepted.accepted_at_ms),
//             visibility: self.visibility,
//         }
//     }
//
//     fn summary(&self, observed_at_ms: u128) -> PendingAcceptedWriteSummary {
//         PendingAcceptedWriteSummary {
//             token: self.accepted.token.clone(),
//             target_member: self.accepted.target_member.clone(),
//             accepted_at_ms: self.accepted.accepted_at_ms,
//             age_ms: observed_at_ms.saturating_sub(self.accepted.accepted_at_ms),
//             visibility: self.visibility.clone(),
//         }
//     }
// }
//
// impl WriteConvergenceSummary {
//     fn summary(&self) -> String {
//         if self.pending.is_empty() {
//             format!(
//                 "accepted={} rejected={} converged={} pending=0",
//                 self.counts.accepted, self.counts.rejected, self.counts.converged
//             )
//         } else {
//             format!(
//                 "accepted={} rejected={} converged={} pending={} ({})",
//                 self.counts.accepted,
//                 self.counts.rejected,
//                 self.counts.converged,
//                 self.counts.pending,
//                 self.pending
//                     .iter()
//                     .map(PendingAcceptedWriteSummary::summary)
//                     .collect::<Vec<_>>()
//                     .join(", ")
//             )
//         }
//     }
// }
//
// impl PendingAcceptedWriteSummary {
//     fn summary(&self) -> String {
//         format!(
//             "{} age_ms={} visibility={}",
//             self.token,
//             self.age_ms,
//             self.visibility
//                 .iter()
//                 .map(MemberTokenVisibility::summary)
//                 .collect::<Vec<_>>()
//                 .join("|")
//         )
//     }
// }
//
// impl MemberTokenVisibility {
//     fn summary(&self) -> String {
//         format!("{}={}", self.member, self.state.summary())
//     }
// }
//
// impl TokenVisibilityState {
//     fn summary(&self) -> String {
//         match self {
//             Self::Visible => "visible".to_string(),
//             Self::Missing => "missing".to_string(),
//             Self::QueryFailed { .. } => "query_failed".to_string(),
//         }
//     }
// }
//
// impl MemberSelfReport {
//     fn is_primary(&self) -> bool {
//         matches!(self, Self::Primary)
//     }
//
//     fn summary(&self) -> String {
//         match self {
//             Self::Primary => "primary".to_string(),
//             Self::NotPrimary { pg_state } => format!("not_primary({})", pg_state.label()),
//             Self::CommandFailed { .. } => "command_failed".to_string(),
//         }
//     }
// }
//
// impl NonPrimaryPgState {
//     fn label(&self) -> &'static str {
//         match self {
//             Self::Replica => "replica",
//             Self::Unknown => "unknown",
//         }
//     }
// }
//
// fn run_primary_count_invariant_loop(
//     observer: PgtmObserver,
//     artifacts_dir: PathBuf,
//     poll_interval: Duration,
//     shared: &SharedPrimaryCountInvariantState,
// ) -> Result<()> {
//     while !shared.stop_requested() {
//         let observation = observer.observe_states()?;
//         let sample = PrimaryCountSample::from_observation(&observation)?;
//         if sample.violates_allowed_primary_counts() {
//             let artifact_path = artifacts_dir.join(PRIMARY_COUNT_VIOLATION_ARTIFACT_NAME);
//             persist_violation_sample(artifact_path.as_path(), &sample)?;
//             shared.store_failure(PrimaryCountInvariantFailure::Violation(
//                 PrimaryCountInvariantViolation::new(artifact_path, sample),
//             ))?;
//             return Ok(());
//         }
//         thread::sleep(poll_interval);
//     }
//
//     Ok(())
// }
//
// fn run_write_convergence_invariant_loop(
//     observer: PgtmObserver,
//     sql: SqlObserver,
//     artifacts_dir: PathBuf,
//     poll_interval: Duration,
//     convergence_window: Duration,
//     shared: &SharedWriteConvergenceInvariantState,
// ) -> Result<()> {
//     let artifacts = WriteConvergenceArtifacts::new(artifacts_dir);
//     let routing_targets = cluster_postgres_routing_targets(&observer)?;
//     let initialized = initialize_write_convergence_table(
//         &sql,
//         routing_targets.as_slice(),
//         poll_interval,
//         shared,
//     )?;
//     if !initialized {
//         return Ok(());
//     }
//
//     let mut tracker = WriteConvergenceTracker::new(convergence_window);
//     artifacts.persist_summary(&tracker.summary(timestamp_millis()?))?;
//
//     while !shared.stop_requested() || tracker.has_pending() {
//         let loop_started_at_ms = timestamp_millis()?;
//         let observation = observer.observe_states()?;
//         let authoritative_primary = cluster_authoritative_primary(&observation);
//
//         if !shared.stop_requested() {
//             if let Some(primary_member) = authoritative_primary {
//                 let target = routing_targets
//                     .iter()
//                     .find(|target| target.member == primary_member)
//                     .ok_or_else(|| {
//                         HarnessError::message(format!(
//                             "write-convergence invariant has no routing target for authoritative primary `{primary_member}`"
//                         ))
//                     })?;
//                 let target_observation = observation.member(target.member)?;
//                 let target_self_report = member_self_report_from_observation(target_observation)?;
//                 let token = tracker.next_token(target.member, loop_started_at_ms);
//                 let outcome = attempt_invariant_write(
//                     &sql,
//                     target,
//                     target_self_report,
//                     token,
//                     loop_started_at_ms,
//                 );
//                 tracker.record_attempt(loop_started_at_ms, outcome, &artifacts)?;
//             }
//
//             let rejection_target = tracker
//                 .next_non_authoritative_target(routing_targets.as_slice(), authoritative_primary)?;
//             let rejection_outcome = match rejection_target {
//                 Some(target) => {
//                     let target_observation = observation.member(target.member)?;
//                     let target_self_report =
//                         member_self_report_from_observation(target_observation)?;
//                     let token = tracker.next_token(target.member, loop_started_at_ms);
//                     attempt_rejected_write(
//                         &sql,
//                         target,
//                         target_self_report,
//                         token,
//                         loop_started_at_ms,
//                         authoritative_primary,
//                     )?
//                 }
//                 None => {
//                     let target = tracker.next_target(routing_targets.as_slice())?;
//                     let target_observation = observation.member(target.member)?;
//                     let target_self_report =
//                         member_self_report_from_observation(target_observation)?;
//                     let token = tracker.next_token(target.member, loop_started_at_ms);
//                     rejected_without_attempt(
//                         target,
//                         target_self_report,
//                         token,
//                         loop_started_at_ms,
//                         "cluster had no non-authoritative target available".to_string(),
//                     )
//                 }
//             };
//             tracker.record_attempt(loop_started_at_ms, rejection_outcome, &artifacts)?;
//         }
//
//         let visibility_snapshots = observe_member_token_snapshots(&sql, routing_targets.as_slice());
//         if let Some(violation) = tracker.reconcile_visibility(
//             timestamp_millis()?,
//             visibility_snapshots.as_slice(),
//             &artifacts,
//         )? {
//             shared.store_failure(WriteConvergenceInvariantFailure::Violation(violation))?;
//             return Ok(());
//         }
//
//         if !shared.stop_requested() || tracker.has_pending() {
//             thread::sleep(poll_interval);
//         }
//     }
//
//     artifacts.persist_summary(&tracker.summary(timestamp_millis()?))
// }
//
// fn member_self_report_from_observation(
//     observation: &crate::support::observer::pgtm::MemberStateObservation,
// ) -> Result<MemberSelfReport> {
//     match &observation.outcome {
//         MemberCommandOutcome::Observed(output) => {
//             classify_self_report(observation.member, &output.state)
//         }
//         MemberCommandOutcome::Failed(message) => Ok(MemberSelfReport::CommandFailed {
//             message: message.clone(),
//         }),
//     }
// }
//
// fn classify_self_report(member: ClusterMember, state: &NodeState) -> Result<MemberSelfReport> {
//     let reported_member = state.identity.member_id.as_str();
//     if reported_member != member.service_name() {
//         return Err(HarnessError::message(format!(
//             "pgtm status via `{member}` returned local identity `{reported_member}`"
//         )));
//     }
//
//     Ok(match state.pg {
//         PgInfoState::Primary { .. } => MemberSelfReport::Primary,
//         PgInfoState::Replica { .. } => MemberSelfReport::NotPrimary {
//             pg_state: NonPrimaryPgState::Replica,
//         },
//         PgInfoState::Unknown { .. } => MemberSelfReport::NotPrimary {
//             pg_state: NonPrimaryPgState::Unknown,
//         },
//     })
// }
//
// fn cluster_postgres_routing_targets(observer: &PgtmObserver) -> Result<Vec<PostgresRoutingTarget>> {
//     ClusterMember::ALL
//         .into_iter()
//         .map(|member| observer.postgres_routing_target(member))
//         .collect::<Result<Vec<_>>>()
// }
//
// fn initialize_write_convergence_table(
//     sql: &SqlObserver,
//     routing_targets: &[PostgresRoutingTarget],
//     poll_interval: Duration,
//     shared: &SharedWriteConvergenceInvariantState,
// ) -> Result<bool> {
//     let mut table_created = false;
//
//     while !shared.stop_requested() {
//         if !table_created {
//             for target in routing_targets {
//                 if sql
//                     .execute(target.dsn.as_str(), write_convergence_table_sql().as_str())
//                     .is_ok()
//                 {
//                     table_created = true;
//                     break;
//                 }
//             }
//         }
//
//         if table_created && invariant_table_visible_on_all_members(sql, routing_targets) {
//             return Ok(true);
//         }
//
//         thread::sleep(poll_interval);
//     }
//
//     Ok(false)
// }
//
// fn write_convergence_table_sql() -> String {
//     format!(
//         "CREATE TABLE IF NOT EXISTS {WRITE_CONVERGENCE_TABLE_NAME} (\
//          token TEXT PRIMARY KEY,\
//          accepted_via TEXT NOT NULL,\
//          accepted_at_ms BIGINT NOT NULL\
//          );"
//     )
// }
//
// fn attempt_invariant_write(
//     sql: &SqlObserver,
//     target: &PostgresRoutingTarget,
//     target_self_report: MemberSelfReport,
//     token: String,
//     attempted_at_ms: u128,
// ) -> WriteAttemptOutcome {
//     let insert_sql = format!(
//         "INSERT INTO {WRITE_CONVERGENCE_TABLE_NAME} (token, accepted_via, accepted_at_ms) \
//          VALUES ('{token}', '{}', {attempted_at_ms}) RETURNING token;",
//         target.member.service_name()
//     );
//     match sql.execute(target.dsn.as_str(), insert_sql.as_str()) {
//         Ok(_) => WriteAttemptOutcome::Accepted(AcceptedWriteRecord {
//             token,
//             target_member: target.member.service_name().to_string(),
//             target_self_report,
//             accepted_at_ms: attempted_at_ms,
//         }),
//         Err(err) => WriteAttemptOutcome::Rejected(RejectedWriteRecord {
//             token,
//             target_member: target.member.service_name().to_string(),
//             target_self_report,
//             rejected_at_ms: attempted_at_ms,
//             reason: err.to_string(),
//         }),
//     }
// }
//
// fn attempt_rejected_write(
//     sql: &SqlObserver,
//     target: &PostgresRoutingTarget,
//     target_self_report: MemberSelfReport,
//     token: String,
//     attempted_at_ms: u128,
//     authoritative_primary: Option<ClusterMember>,
// ) -> Result<WriteAttemptOutcome> {
//     if matches!(target_self_report, MemberSelfReport::Primary) {
//         return Ok(rejected_without_attempt(
//             target,
//             target_self_report,
//             token,
//             attempted_at_ms,
//             format!(
//                 "target was not the authoritative primary (authoritative_primary={})",
//                 authoritative_primary
//                     .map(|member| member.service_name().to_string())
//                     .unwrap_or_else(|| "none".to_string())
//             ),
//         ));
//     }
//
//     let insert_sql = format!(
//         "INSERT INTO {WRITE_CONVERGENCE_TABLE_NAME} (token, accepted_via, accepted_at_ms) \
//          VALUES ('{token}', '{}', {attempted_at_ms}) RETURNING token;",
//         target.member.service_name()
//     );
//
//     match sql.execute(target.dsn.as_str(), insert_sql.as_str()) {
//         Ok(_) => Err(HarnessError::message(format!(
//             "non-authoritative target `{}` unexpectedly accepted an invariant write",
//             target.member
//         ))),
//         Err(err) => Ok(WriteAttemptOutcome::Rejected(RejectedWriteRecord {
//             token,
//             target_member: target.member.service_name().to_string(),
//             target_self_report,
//             rejected_at_ms: attempted_at_ms,
//             reason: err.to_string(),
//         })),
//     }
// }
//
// fn rejected_without_attempt(
//     target: &PostgresRoutingTarget,
//     target_self_report: MemberSelfReport,
//     token: String,
//     rejected_at_ms: u128,
//     reason: String,
// ) -> WriteAttemptOutcome {
//     WriteAttemptOutcome::Rejected(RejectedWriteRecord {
//         token,
//         target_member: target.member.service_name().to_string(),
//         target_self_report,
//         rejected_at_ms,
//         reason,
//     })
// }
//
// fn observe_member_token_snapshots(
//     sql: &SqlObserver,
//     routing_targets: &[PostgresRoutingTarget],
// ) -> Vec<MemberTokenSnapshot> {
//     routing_targets
//         .iter()
//         .map(|target| MemberTokenSnapshot {
//             member: target.member,
//             observation: match sql.execute(target.dsn.as_str(), visible_tokens_sql().as_str()) {
//                 Ok(stdout) => {
//                     MemberTokenObservation::VisibleTokens(parse_visible_tokens(stdout.as_str()))
//                 }
//                 Err(err) if relation_missing_error(&err) => {
//                     MemberTokenObservation::VisibleTokens(BTreeSet::new())
//                 }
//                 Err(err) => MemberTokenObservation::QueryFailed(err.to_string()),
//             },
//         })
//         .collect::<Vec<_>>()
// }
//
// fn visible_tokens_sql() -> String {
//     format!("SELECT token FROM {WRITE_CONVERGENCE_TABLE_NAME} ORDER BY token;")
// }
//
// fn parse_visible_tokens(stdout: &str) -> BTreeSet<String> {
//     stdout
//         .lines()
//         .map(str::trim)
//         .filter(|line| !line.is_empty())
//         .map(ToString::to_string)
//         .collect::<BTreeSet<_>>()
// }
//
// fn invariant_table_visible_on_all_members(
//     sql: &SqlObserver,
//     routing_targets: &[PostgresRoutingTarget],
// ) -> bool {
//     routing_targets.iter().all(|target| {
//         sql.execute(target.dsn.as_str(), invariant_table_presence_sql().as_str())
//             .map(|stdout| stdout.trim() == WRITE_CONVERGENCE_TABLE_NAME)
//             .unwrap_or(false)
//     })
// }
//
// fn invariant_table_presence_sql() -> String {
//     format!("SELECT to_regclass('{WRITE_CONVERGENCE_TABLE_NAME}');")
// }
//
// fn cluster_authoritative_primary(observation: &ClusterStateObservation) -> Option<ClusterMember> {
//     let mut authoritative_holders = observation
//         .members()
//         .iter()
//         .filter_map(|member| member.state().and_then(authoritative_primary))
//         .collect::<BTreeSet<_>>()
//         .into_iter();
//
//     match (authoritative_holders.next(), authoritative_holders.next()) {
//         (Some(primary), None) => Some(primary),
//         _ => None,
//     }
// }
//
// fn authoritative_primary(status: &NodeState) -> Option<ClusterMember> {
//     match &status.ha.publication {
//         PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
//             ClusterMember::parse(epoch.holder.0.as_str()).ok()
//         }
//         PublicationState::Unknown
//         | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
//     }
// }
//
// fn relation_missing_error(err: &HarnessError) -> bool {
//     matches!(
//         err,
//         HarnessError::CommandFailed { stderr, .. }
//         if stderr.contains("relation \"public.ha_write_convergence_invariant\" does not exist")
//     )
// }
//
// fn persist_violation_sample(path: &Path, sample: &PrimaryCountSample) -> Result<()> {
//     let rendered = serde_json::to_string_pretty(sample).map_err(|source| HarnessError::Json {
//         context: "serializing primary-count invariant violation".to_string(),
//         source,
//     })?;
//     write_text_file(path, rendered.as_str())
// }
//
// fn create_dir_all(path: &Path) -> Result<()> {
//     fs::create_dir_all(path).map_err(|source| HarnessError::Io {
//         path: path.to_path_buf(),
//         source,
//     })
// }
//
// fn append_line(path: &Path, line: &str) -> Result<()> {
//     if let Some(parent) = path.parent() {
//         create_dir_all(parent)?;
//     }
//     let mut file = OpenOptions::new()
//         .create(true)
//         .append(true)
//         .open(path)
//         .map_err(|source| HarnessError::Io {
//             path: path.to_path_buf(),
//             source,
//         })?;
//     writeln!(file, "{line}").map_err(|source| HarnessError::Io {
//         path: path.to_path_buf(),
//         source,
//     })
// }
//
// fn write_text_file(path: &Path, content: &str) -> Result<()> {
//     if let Some(parent) = path.parent() {
//         create_dir_all(parent)?;
//     }
//     fs::write(path, content).map_err(|source| HarnessError::Io {
//         path: path.to_path_buf(),
//         source,
//     })
// }
//
// fn timestamp_millis() -> Result<u128> {
//     SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .map(|duration| duration.as_millis())
//         .map_err(|err| HarnessError::message(format!("system clock error: {err}")))
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn primary_count_sample_detects_dual_primary_violation() {
//         let sample = PrimaryCountSample {
//             observed_at_ms: 1,
//             allowed_primary_counts: [0, 1],
//             primary_count: 2,
//             members: vec![
//                 MemberPrimaryCountSample {
//                     member: "node-a".to_string(),
//                     self_report: MemberSelfReport::Primary,
//                 },
//                 MemberPrimaryCountSample {
//                     member: "node-b".to_string(),
//                     self_report: MemberSelfReport::Primary,
//                 },
//                 MemberPrimaryCountSample {
//                     member: "node-c".to_string(),
//                     self_report: MemberSelfReport::NotPrimary {
//                         pg_state: NonPrimaryPgState::Replica,
//                     },
//                 },
//             ],
//         };
//
//         assert!(sample.violates_allowed_primary_counts());
//     }
//
//     #[test]
//     fn pending_write_marks_convergence_once_all_members_see_the_token() {
//         let accepted = AcceptedWriteRecord {
//             token: "token-1".to_string(),
//             target_member: "node-b".to_string(),
//             target_self_report: MemberSelfReport::Primary,
//             accepted_at_ms: 50,
//         };
//         let mut pending = PendingAcceptedWrite::new(accepted);
//         pending.refresh_visibility(
//             [
//                 MemberTokenSnapshot {
//                     member: ClusterMember::NodeA,
//                     observation: MemberTokenObservation::VisibleTokens(
//                         ["token-1".to_string()].into_iter().collect::<BTreeSet<_>>(),
//                     ),
//                 },
//                 MemberTokenSnapshot {
//                     member: ClusterMember::NodeB,
//                     observation: MemberTokenObservation::VisibleTokens(
//                         ["token-1".to_string()].into_iter().collect::<BTreeSet<_>>(),
//                     ),
//                 },
//                 MemberTokenSnapshot {
//                     member: ClusterMember::NodeC,
//                     observation: MemberTokenObservation::VisibleTokens(
//                         ["token-1".to_string()].into_iter().collect::<BTreeSet<_>>(),
//                     ),
//                 },
//             ]
//             .as_slice(),
//         );
//
//         assert!(pending.is_converged());
//     }
//
//     #[test]
//     fn summary_reports_pending_timeout_candidates() {
//         let accepted = AcceptedWriteRecord {
//             token: "token-2".to_string(),
//             target_member: "node-a".to_string(),
//             target_self_report: MemberSelfReport::Primary,
//             accepted_at_ms: 10,
//         };
//         let mut tracker = WriteConvergenceTracker::new(Duration::from_millis(20));
//         tracker.accepted_count = 1;
//         let _ = tracker
//             .pending
//             .insert(accepted.token.clone(), PendingAcceptedWrite::new(accepted));
//         let summary = tracker.summary(40);
//
//         assert_eq!(summary.counts.pending, 1);
//         assert_eq!(summary.pending[0].age_ms, 30);
//     }
// }
