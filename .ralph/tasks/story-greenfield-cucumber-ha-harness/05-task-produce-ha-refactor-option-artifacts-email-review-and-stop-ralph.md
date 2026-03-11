## Task: Produce HA Refactor Option Artifacts, Email Review, And Stop Ralph <status>completed</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Produce a design-only refactor study for the HA loop and all surrounding startup/reconciliation logic. This task must not change production code, tests, docs, configuration, or behavior. The only repo outputs are this task file plus comprehensive design artifacts under this greenfield story directory.

**Non-negotiable clarity on test outcomes:** Green test outcomes are NOT the goal of this task. Passing `make test`, passing `make test-long`, or fixing any failing HA behavior is explicitly NOT the objective here. This task exists only to study, explain, compare, and communicate redesign options. The executor must NOT “helpfully” fix code, patch tests, or chase green gates during this task.

**Original user shift / motivation:** The user wants a full redesign plan for the HA loop because the current architecture drifted away from the intended shape. The desired model is: newest observations first, then a pure decide step, then a typed outcome that lower layers turn into actions. The user explicitly likes the current worker send/receive state style and wants to keep that functional direction, but believes the implementation is now too spread out, startup logic is disconnected from the decide loop, sender-side dedup slipped into the HA worker, and the current quorum/failsafe boundary is wrong.

**Higher-order goal:** Prepare ten materially different, extremely detailed, fully self-contained refactor designs that can later guide an implementation which unifies startup and steady-state reconciliation into a clearer typed state machine, keeps side effects outside the HA decider, makes quorum/lease semantics correct, and gives a clean path to passing the HA feature suite. Every design must be verbose enough that a later implementer can understand the intended architecture, boundaries, transitions, required file/type changes, and operator-visible behavior without opening any chat history, prior task file, or repo documentation.

**Scope:**
- Artifact-only work inside `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/`.
- Create `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/` and write ten `.md` design documents there, one file per design.
- Run `make test` and `make test-long` as diagnostic inputs only. Failures are expected and must be treated as evidence, not as bugs to fix in this task.
- Sanity-check the mail reply interface with the absolute-path script `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh`.
- Email every design artifact in full, one email per design, to `user@toffemail.nl`, using `reply.sh` and absolute paths.
- Send one final email ranking the preferred top three designs and explaining why.
- Touch `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/STOP` only after all ten design emails and the final ranking email are sent.

**Execution chunking rule:**
- This task must be worked in very small increments to protect context.
- After creating just one design artifact, the executor must immediately send that one design by email via `reply.sh` and then `QUIT IMMEDIATELY`.
- Do not continue to a second design in the same run after the first design email is sent.
- On later resumptions of this same task, repeat the same pattern: create the next design, email it in full, then `QUIT IMMEDIATELY`.
- Only after the tenth design has been created and emailed may the executor send the final top-three ranking email and then touch `.ralph/STOP`.

**Important boundaries:**
- Do not change any files under `src/`, `tests/`, `docs/`, `docker/`, `.config/`, or `Cargo.toml`.
- Do not read anything under `docs/`. Reading `docs/` for this task is strictly forbidden. The designs must stand on code/test/repo evidence and the task context only.
- Do not fix any bugs, do not implement any design, do not edit existing tests, and do not try to make repo gates pass in this task.
- DO NOT try to make tests green in this task.
- DO NOT treat a red `make test` or red `make test-long` outcome as a problem to solve here.
- DO NOT patch production code or tests in response to any failure you observe.
- Do not create `add-bug` tasks from findings in this turn. Capture discovered issues inside the design artifacts instead.
- Do not run `cargo test` anywhere in this repo. If the executor wants diagnostics, use `make test` and `make test-long` exactly as requested by the user.
- Do not silently collapse multiple design ideas into one file. The user asked for ten different redesigns, not ten restatements of the same proposal.

**Context from research:**
- `src/runtime/node.rs` still contains a separate startup planner/executor (`plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, `build_startup_actions(...)`) that makes important HA/startup decisions before the long-running workers take over. This separation is directly relevant to the user's complaint that startup logic is absent from the decide loop.
- `src/ha/worker.rs` currently follows `world_snapshot -> decide -> lower -> publish -> apply`, but it also contains sender-side dedup logic via `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. The user explicitly wants this deduplication concern moved away from senders and into receivers / effect consumers.
- `src/ha/decide.rs` immediately routes any non-`DcsTrust::FullQuorum` state into `HaPhase::FailSafe`, and a primary under non-full-quorum returns `HaDecision::EnterFailSafe { release_leader_lease: false }`. The user explicitly called this boundary wrong and wants continued operation in degraded-but-valid quorum situations, such as a three-node cluster still operating with two healthy members and re-electing a leader.
- `src/ha/decision.rs` centralizes `DecisionFacts`, `HaDecision`, `RecoveryStrategy`, and process-activity interpretation. This is a likely anchor for a future typed redesign and should be discussed in every artifact.
- `src/ha/lower.rs` already preserves the user-preferred split between pure decision selection and lower-level effect planning (`LeaseEffect`, `ReplicationEffect`, `PostgresEffect`, `SafetyEffect`). The user wants this functional style kept, not replaced with effectful branching.
- `src/ha/process_dispatch.rs` still derives `StartPostgres` intent from the previous HA decision and contains the authoritative start-intent bridge (`start_intent_from_dcs(...)`, `start_postgres_leader_member_id(...)`, rewind/basebackup source validation). This is part of the startup/rejoin boundary the user wants redesigned.
- `src/dcs/worker.rs` constructs and publishes the local `MemberRecord` from the latest pginfo snapshot on every DCS step. `src/dcs/state.rs` defines `MemberRecord`, trust evaluation, freshness, and quorum heuristics. The user explicitly wants member keys to always contain the latest obtainable information, including “pginfo failed but pgtuskmaster is up” style partial truth rather than silence.
- `src/pginfo/state.rs` models partial information already: `PgInfoState::Unknown`, `SqlStatus::{Unknown,Healthy,Unreachable}`, and `Readiness::{Unknown,Ready,NotReady}`. The designs should consider how to preserve and publish these partial facts instead of collapsing them into absence.
- `tests/ha.rs` enumerates the current HA feature suite. The designs must logically account for, at minimum, `ha_dcs_quorum_lost_enters_failsafe`, `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`, `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`, `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`, `ha_primary_killed_then_rejoins_as_replica`, `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`, `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, `ha_rewind_fails_then_basebackup_rejoins_old_primary`, `ha_replica_stopped_primary_stays_primary`, and `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`.
- The mail reply tool is not in this repo root. The correct script is `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh`. A sanity check with `--help` currently prints usage text and exits non-zero because the script requires `<to_address>` and `<original_subject>`. For this task, the reply recipient is fixed to `user@toffemail.nl`.
- The exact original subject from the requesting mail thread is not recorded anywhere inside this repo snapshot. To keep the task executable, use the canonical thread subject `Produce HA Refactor Option Artifacts, Email Review, And Stop Ralph` for every reply in this task unless a later resumption recovers a more exact subject from local receive-mail state.

**Approved design requirements that every artifact must address:**
- Every artifact must be VERY complete, verbosely detailed, and literally self-contained. Do not rely on “implied” steps, omitted transitions, or “future implementation can figure this out” shortcuts. Spell out the changed method, changed flow, changed boundaries, changed types, and all required implementation areas in enough detail that the design stands alone.
- Every artifact must include at least one ASCII diagram that shows the proposed new method / changed control flow / changed responsibility boundary. The diagram must be meaningful, not decorative.
- Startup must be folded into the same overall HA reconciliation model so that “same newest info + same state => same actions” applies on startup as well as steady state.
- The HA loop should keep the functional chain `newest info -> decide -> typed outcome -> actions`, with no direct Postgres or etcd side effects inside the pure decider.
- Only the DCS layer may read/write etcd3 keys. Only the pginfo worker may read Postgres. The HA loop itself may only consume observations and produce typed outcomes/effect plans.
- Deduplication must move out of sender-side HA logic and into receivers/effect consumers.
- The no-consensus / no-full-quorum behavior must be redesigned so the system does not blindly drop into the current primary failsafe path when a valid majority can still operate.
- Lease thinking must be stronger. Leader authority and leader loss should be reasoned about explicitly, including killed-primary / lost-lease situations.
- On startup the node must first reason about existing cluster state, leader state, and member state before deciding whether it should initialize, follow, promote, rewind, basebackup, or simply continue.
- Member publication must preserve partial truth. “pginfo failed but pgtuskmaster is up” is still valid information and should remain publishable.
- Bootstrapping must be reconsidered, including whether bootstrap should have substates and whether existing `pgdata` can still be used when the node wins the init lock.
- Replica convergence should be simplified into one coherent sequence: healthy follow if already good, tolerate minor lag when acceptable, rewind on wrong timeline when possible, fall back to basebackup when rewind is impossible, and treat previously-primary / previously-replica / freshly-restored nodes as variations of the same convergence path when a valid healthy leader exists.

**Expected outcome:**
- The greenfield story contains ten exhaustive, standalone HA-loop redesign documents that somebody can read without opening the original conversation.
- Each design explicitly explains the current design problems, the proposed state model, how startup is unified, how quorum/lease boundaries change, where deduplication moves, how DCS/member publication works, how bootstrap works, how replica convergence works, what files and types would later change, and how the proposal would logically satisfy the HA feature tests.
- The design set makes it unmistakable that diagnosing failing tests was only an input to planning and that no code fixes were part of this task.
- The task is intentionally incremental: one design per run, one email per run, then `QUIT IMMEDIATELY`.
- After the tenth design, the executor sends one final ranking email with the preferred top three options and reasons, then touches `.ralph/STOP`.
</description>

<acceptance_criteria>
- [x] No files under `src/`, `tests/`, `docs/`, `docker/`, `.config/`, or `Cargo.toml` are modified by this task; the work is artifact-only.
- [x] `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/` exists and contains ten design `.md` files, with one comprehensive redesign per file.
- [x] Every design file is fully self-contained, exhaustive, and readable without any outside context from chat history or prior task files.
- [x] Every design explicitly addresses the user's required themes: unified startup + HA loop, sender-side dedup removal, reduced HA spread / more unified state machine shape, corrected degraded-quorum boundary, stronger lease semantics, authoritative startup observation, partial-truth member publication, bootstrap rethink, and a simplified replica convergence path.
- [x] Every design names the concrete future code areas it would affect, including the relevant boundaries in `src/runtime/node.rs`, `src/ha/worker.rs`, `src/ha/decide.rs`, `src/ha/decision.rs`, `src/ha/lower.rs`, `src/ha/process_dispatch.rs`, `src/dcs/worker.rs`, `src/dcs/state.rs`, and the HA feature suite under `tests/ha.rs` / `tests/ha/features/`.
- [x] Every design explicitly lists all meaningful code-path, type, module, state-transition, and behavior changes needed for that option; hand-wavy “then refactor accordingly” wording does not satisfy this task.
- [x] Every design includes at least one meaningful ASCII diagram showing the proposed new method / changed flow / changed boundaries.
- [x] No executor work for this task reads from `docs/`; `docs/` is a forbidden input source for this task.
- [x] Every design contains a logical feature-test verification section that explains, test by test, how the proposal would make the HA behavior correct without actually implementing code in this task.
- [x] Every design contains at least five explicit question sections at the end, using the exact markdown pattern `## Q1 [short question 1]`, `## Q2 ...`, through at least `## Q5 ...`, where each question section includes context, the problem/decision point, and a restatement of the question; ASCII diagrams may be included inside those sections when helpful.
- [x] The ten designs are materially different from each other; duplicated wording with minor renaming does not satisfy this task.
- [x] `make test` and `make test-long` are each run as diagnostic inputs only, their outcomes are recorded inside the design set, and no attempt is made to fix the failures in this task.
- [x] The artifacts explicitly state that green test outcomes are NOT the goal of this task and that no production or test-code fixes were allowed here.
- [x] The reply script interface is sanity-checked at `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh`, and the executor uses the absolute script path plus absolute artifact paths when sending mail.
- [x] After each individual design is written, that design is emailed in full to `user@toffemail.nl` using `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh`, and the executor then `QUIT IMMEDIATELY`.
- [x] After the tenth design has been emailed, one final email is sent ranking the preferred top three designs and explaining why they are preferred.
- [x] `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/STOP` is touched only after all ten design emails and the final ranking email are sent.
- [x] `<passes>true</passes>` is not set for this task unless a separate follow-up implementation task later makes `make check`, `make test`, `make test-long`, and `make lint` pass; this task itself is explicitly diagnostic and design-only.
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Reconfirm the design-only contract and collect failure evidence
- [x] Re-read this task and preserve the non-goals: no code changes, no bug fixes, no test edits, no docs edits, no `cargo test`, no implementation.
- [x] Do not read from `docs/` at all while working this task. `docs/` is a strictly forbidden input source here.
- [x] Create `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/`.
- [x] Run `make test` from `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust` and capture the failing themes that are relevant to the HA redesign. Treat the failures as evidence only.
- [x] Run `make test-long` from `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust` and capture the failing or behavior-mismatch themes that are relevant to the HA redesign. Treat the failures as evidence only.
- [x] Do not modify code or tests in response to those failures. The only allowed outputs are the design artifacts under this greenfield story.
- [x] Write the first paragraph of the current run's artifact so it explicitly says this is a design task and NOT a code-fixing task, and that green tests are NOT the target outcome.

### Phase 2: Define the ten option set before writing
- [x] Decide on ten clearly different redesign directions before drafting any artifact. Each option must have a one-paragraph differentiator that makes it obviously distinct from the others.
- [x] Ensure the ten options are not just stylistic variants. They should differ in state-machine structure, startup integration shape, lease/quorum semantics, recovery funneling, or responsibility boundaries while still honoring the user's fixed constraints.
- [x] For every option, pre-commit to keeping the functional chain `newest info -> decide -> lower -> actions` and to keeping DCS/Postgres IO out of the pure HA decider.
- [x] For every option, pre-commit to relocating deduplication away from sender-side HA worker code and into receivers/effect consumers.
- [x] Before writing the first design in any given run, decide which exact single option will be completed in that run. Do not start multiple designs in parallel in one run.

### Phase 3: Write exactly one comprehensive artifact in the current run
- [x] In the current run, write exactly one new design file under `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/`, using a descriptive filename that reflects the option's central idea.
- [x] In that design file, include a clear title and a short “why this option exists” differentiator at the top.
- [x] In that design file, make the writing VERY complete, verbose, and literally self-contained. The file must be able to stand alone as implementation guidance without requiring the reader to inspect chat history, other task files, or anything under `docs/`.
- [x] In that design file, include a “Current design problems” section that explicitly covers:
  current startup logic split across `src/runtime/node.rs`
  sender-side dedup in `src/ha/worker.rs`
  spread of HA logic across runtime / decide / dispatch boundaries
  current non-full-quorum -> `FailSafe` shortcut in `src/ha/decide.rs`
  startup/rejoin ambiguity in `src/ha/process_dispatch.rs`
  member publication / partial-truth requirements in `src/dcs/worker.rs` and `src/pginfo/state.rs`
- [x] In that design file, describe the full proposed control flow from startup through steady-state ticks, including how the same observation/decision model applies before and after process startup.
- [x] In that design file, include at least one meaningful ASCII diagram that shows the proposed new method / changed control flow / changed responsibility boundaries.
- [x] In that design file, define the proposed typed state machine in detail, including phase names, substate names where relevant, transition triggers, invariants, and any new structs/enums that would later encode the model.
- [x] In that design file, explain the redesigned quorum model. Explicitly discuss why degraded-but-valid majority operation should continue, when the node must fence or demote, and how leadership re-election should work in 2-of-3 style cases.
- [x] In that design file, explain the lease model. Cover lease acquisition, lease expiry / loss, how a killed primary loses authority, and how lease state interacts with startup and failover.
- [x] In that design file, explain startup reasoning. Cover cluster already up, cluster leader already present, existing members already published, empty vs existing `pgdata`, init lock behavior, and when existing local data may still be valid for initialization.
- [x] In that design file, explain replica convergence as one coherent path, including healthy follow, tolerable lag, wrong-timeline rewind, and basebackup fallback when rewind cannot succeed.
- [x] In that design file, explain how partial information is published to member keys when pginfo is degraded or unavailable but the process is still running.
- [x] In that design file, explain where deduplication moves and why the chosen boundary is safer than the current `should_skip_redundant_process_dispatch(...)` sender-side approach.
- [x] In that design file, list the concrete repo files, modules, functions, and types that a future implementation would touch.
- [x] In that design file, list all meaningful changes needed for the option: new types, deleted paths, moved responsibilities, changed transitions, changed effect-lowering boundaries, changed DCS publication behavior, changed startup handling, changed convergence handling, and any required test updates that a later implementation would need.
- [x] In that design file, include a migration sketch for how a later implementation could move from the current architecture to that option without silently leaving stale legacy paths behind.
- [x] In that design file, include explicit non-goals and tradeoffs so the option can be judged on its own merits.
- [x] In that design file, include a final open-questions section containing at least five questions in exactly this form:
  `## Q1 [short question 1]`
  `[context for question + potential ascii diagram]`
  `[problem or decision problem]`
  `[restating question in different way]`
  Then continue with `## Q2 ...` through at least `## Q5 ...`.

### Phase 4: Logically verify the current run's design against the HA feature suite
- [x] In the current run's design file, add a “Logical feature-test verification” section.
- [x] Explicitly map the current option against the key HA scenarios from `tests/ha.rs`, including:
  `ha_dcs_quorum_lost_enters_failsafe`
  `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`
  `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`
  `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`
  `ha_primary_killed_then_rejoins_as_replica`
  `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`
  `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`
  `ha_rewind_fails_then_basebackup_rejoins_old_primary`
  `ha_replica_stopped_primary_stays_primary`
  `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`
- [x] When the design alters the interpretation of an existing scenario, explain the new boundary precisely instead of hand-waving. Be explicit about what should remain “degraded but healthy enough” versus what must still fence or fail safe.

### Phase 5: Email the current run's artifact and stop immediately
- [x] Sanity-check the mail script interface with `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh --help` and note that it prints usage text and exits non-zero because it expects `<to_address>` and `<original_subject>`.
- [x] Use `user@toffemail.nl` as the reply recipient for every design email and for the final ranking email.
- [x] Reuse the exact original subject from the mail thread that requested this work if it is recoverable from local receive-mail state; otherwise use the canonical subject `Produce HA Refactor Option Artifacts, Email Review, And Stop Ralph` recorded in this task file so the reply step remains executable in later resumptions.
- [x] Use the absolute script path and the absolute artifact path when sending mail. The required form is:
  `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh "user@toffemail.nl" "<original_subject>" < "/absolute/path/to/design.md"`
- [x] Send the current run's one completed design artifact as a full-text email body. Do not send a summary instead of the full text.
- [x] After sending that one design email, `QUIT IMMEDIATELY`.

### Phase 6: Final completion only after all ten designs exist
- [x] Repeat Phases 2 through 5 across later resumptions until ten materially different design files have been created and each one has been emailed in full.
- [x] After the tenth design has been emailed, send one more reply email on the same thread that ranks the preferred top three options and explains the preference ordering in concrete architectural terms.
- [x] Reconfirm that no code, tests, or docs were changed outside this story directory.
- [x] Reconfirm that the ten design files exist and are materially distinct.
- [x] Reconfirm that `make test` and `make test-long` were used only for diagnostics and that their outcomes are reflected in the artifact set, and that no code-fixing work was performed.
- [x] Reconfirm that every design email and the final top-three email were sent to `user@toffemail.nl` via `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh`.
- [x] Touch `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/STOP`.
- [x] Leave `<passes>false</passes>` unchanged for this task, because it is a no-code research/design task and repo gates are not being made green here.

NOW EXECUTE
