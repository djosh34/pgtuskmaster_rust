## Task: Rebuild contributor docs as a codebase navigation and design-contract guide <status>completed</status> <passes>true</passes>

<description>
Rewrite the contributor documentation so it becomes a genuinely useful guide for understanding the codebase, subsystem boundaries, implementation approach, and design contracts.

The agent must explore the current codebase and docs first, then rebuild contributor docs around the exact things a new contributor needs to learn:
- how to navigate the codebase
- which modules own which responsibilities
- how the major systems are implemented
- what the important design contracts and invariants are
- how runtime data and control flow move between components
- how to locate the code for specific behaviors quickly
- how the implementation is split between runtime, process control, HA, DCS, APIs, config, tests, and harness code

This task must implement the following fixed product decisions:
- the contributor section must be a strong guide for learning the whole codebase
- it must explain code navigation, implementation paths, subsystem boundaries, and design contracts in terms grounded in the current code
- it must not become a vague essay, a chapter directory, or a dump of file names without explanation
- it must make the current contributor docs materially useful for understanding how the system is built and how to safely change it
- it must also be better to read and better to use as documentation, not just more complete

The agent should use parallel subagents after exploration to cover different subsystem areas and then unify the writing.
</description>

<acceptance_criteria>
- [x] Contributor docs clearly explain codebase navigation and module ownership
- [x] Contributor docs explain major subsystem implementation paths and design contracts in terms grounded in the current code
- [x] Contributor docs explain how to find the code for important behaviors and how information/control flow moves across subsystems
- [x] Contributor docs are useful for learning the system rather than merely listing chapters or files
- [x] Writing quality is substantially improved over the current contributor section
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make test-long` — passes cleanly
- [x] `make lint` — passes cleanly
</acceptance_criteria>

## Execution plan

### 1. Re-audit the current contributor section against the live codebase

- Treat the existing contributor pages as draft material, not accepted truth.
- Re-read these docs in book order and note where they are too shallow, too repetitive, missing navigation help, or no longer reflect the current implementation:
  - `docs/src/contributors/index.md`
  - `docs/src/contributors/codebase-map.md`
  - `docs/src/contributors/worker-wiring.md`
  - `docs/src/contributors/ha-pipeline.md`
  - `docs/src/contributors/api-debug-contracts.md`
  - `docs/src/contributors/testing-system.md`
  - `docs/src/contributors/harness-internals.md`
  - `docs/src/contributors/verification.md`
- Re-verify every major contributor-doc claim against the code paths that currently own those behaviors. Minimum source set to inspect during execution:
  - `src/bin/pgtuskmaster.rs`
  - `src/runtime/node.rs`
  - `src/state/mod.rs`
  - `src/state/watch_state.rs`
  - `src/pginfo/worker.rs`
  - `src/pginfo/state.rs`
  - `src/dcs/worker.rs`
  - `src/dcs/state.rs`
  - `src/dcs/store.rs`
  - `src/ha/worker.rs`
  - `src/ha/decide.rs`
  - `src/ha/decision.rs`
  - `src/ha/actions.rs`
  - `src/ha/state.rs`
  - `src/ha/lower.rs`
  - `src/ha/apply.rs`
  - `src/ha/process_dispatch.rs`
  - `src/process/worker.rs`
  - `src/process/jobs.rs`
  - `src/process/state.rs`
  - `src/api/worker.rs`
  - `src/api/controller.rs`
  - `src/api/fallback.rs`
  - `src/debug_api/worker.rs`
  - `src/debug_api/snapshot.rs`
  - `src/debug_api/view.rs`
  - `src/test_harness/mod.rs`
  - `src/test_harness/binaries.rs`
  - `src/test_harness/provenance.rs`
  - `src/test_harness/ha_e2e/mod.rs`
  - `src/test_harness/ha_e2e/startup.rs`
  - `src/test_harness/ha_e2e/ops.rs`
  - `tests/bdd_api_http.rs`
  - `tests/ha_multi_node_failover.rs`
  - `tests/ha_partition_recovery.rs`
- During this audit, capture a concrete mismatch list inside the task file before rewriting prose so the eventual doc changes are driven by evidence instead of generic cleanup.

#### Execution audit mismatch list

- `docs/src/contributors/codebase-map.md` still points readers at `src/dcs/etcd_store.rs` as a primary evidence file, but the contributor-facing ownership split is actually anchored by `src/dcs/store.rs` plus `src/dcs/worker.rs`, with `src/dcs/etcd_store.rs` only one concrete backend detail.
- The contributor section currently buries test-harness mechanics one level under `testing-system.md`, while the current repo shape gives harness code first-class ownership under `src/test_harness/{namespace,ports,etcd3,pg16,net_proxy,ha_e2e}` and scenario support under `tests/ha/support/*`; that deserves explicit contributor navigation instead of a subordinate afterthought.
- `verification.md` and `docs-style.md` are both maintenance chapters and currently pull the contributor reading path away from code-navigation value. Their content is both useful, but split across two pages it reads like process overhead instead of one contributor-doc contract.
- Several pages explain responsibilities well but are weaker on “open these identifiers first” guidance. The rewrite needs to add concrete entrypoints such as `run_node_from_config`, `run_workers`, `StateSubscriber::latest`, `ha::decide::decide`, `lower_decision`, `apply_effect_plan`, `api::worker::route_request`, and `debug_api::snapshot::build_snapshot`.
- The current contributor index reads mostly as a chapter directory. It needs a stronger “question map” and explicit contributor jobs so a new engineer can jump from symptom or change goal to the right code path quickly.

### 2. Rebuild the contributor section around actual contributor jobs

- Keep the contributor section as a learning and navigation guide for the running system, not as an essay or a raw file listing.
- Decide the chapter split before rewriting prose. Do not treat the current filenames as the default; compare the existing split with a more task-oriented alternative that groups pages by contributor jobs (`orient`, `trace control flow`, `change HA safely`, `debug APIs/read models`, `verify with harness/tests`, `maintain docs rigor`).
- Rewrite the section so a new engineer can answer these concrete questions quickly:
  - where the runtime starts and where startup ends
  - which module owns observation, coordination, decision, side effects, projection, and external interfaces
  - which files to open when debugging switchover, failover, fencing, startup, API state, or test failures
  - which invariants must not be broken when editing each subsystem
- Preserve the existing section only where the current page already helps; otherwise rewrite aggressively.
- Preferred contributor reading path after rewrite should be finalized only after that split decision. The default skeptical preference is:
  1. `index.md` as the orientation page and question map
  2. one ownership/navigation chapter (`codebase-map.md` or a replacement) that answers “where do I start in code?”
  3. one control-flow chapter (`worker-wiring.md` or a replacement) that explains startup, steady state, and read-model publication together
  4. one HA change-safety chapter (`ha-pipeline.md` or a replacement) that separates decisions from effects
  5. one external-contract chapter (`api-debug-contracts.md` or a replacement) that explains writes, reads, and debug projections
  6. one verification chapter that may merge `testing-system.md` and `harness-internals.md` if that produces a tighter contributor workflow
  7. one docs-maintenance chapter that may merge `verification.md` and `docs-style.md` if keeping both would dilute codebase-navigation value
- Remove stale pages aggressively when consolidating. Do not keep low-signal maintenance chapters in the contributor flow just because they already exist.

### 3. Make the rewritten docs more concrete than the current version

- Add explicit code-navigation help, not just responsibility summaries. Each major page should tell the reader which functions/types/files are the fastest starting points for that topic.
- Add implementation-path explanations that connect startup, steady state, API intent writes, DCS watch updates, HA decision ticks, process job outcomes, and debug snapshot projection into one mental model.
- Make design contracts explicit where they matter most. At minimum, cover:
  - single-owner state publishing and latest-snapshot semantics
  - separation between startup planning/execution and steady-state HA control
  - separation between pure HA decision logic and side-effect application
  - separation between raw worker state and the composed debug/API read model
  - test-harness responsibility for proving behavior with real binaries rather than mocked-only confidence
- Include “how to safely change this area” guidance on the pages where the invariant matters, especially for DCS writes, process side effects, and HA transitions.
- Remove wording that reads like a chapter catalog, generic architecture boilerplate, or unsupported certainty.

### 4. Use parallel subagents during execution, then integrate editorially

- After the plan is promoted to `NOW EXECUTE`, use parallel subagents on disjoint contributor-doc slices so the rewrite is informed by multiple subsystem audits without fragmenting ownership.
- Planned split:
  - subagent A: runtime bootstrap, state/watch semantics, worker wiring, and codebase-map accuracy
  - subagent B: HA pipeline plus API/debug contract accuracy
  - subagent C: testing-system, harness internals, and verification workflow accuracy
- Keep final integration, voice, cross-link cleanup, and task-file bookkeeping in the main agent.
- If the audit uncovers a real product/code bug instead of a docs issue, either fix it if it is tightly coupled to the contributor contract being documented or create a follow-up bug immediately with the `add-bug` skill.

### 5. Files expected to change during `NOW EXECUTE`

- Primary docs:
  - `docs/src/contributors/index.md`
  - `docs/src/contributors/codebase-map.md`
  - `docs/src/contributors/worker-wiring.md`
  - `docs/src/contributors/ha-pipeline.md`
  - `docs/src/contributors/api-debug-contracts.md`
  - `docs/src/contributors/testing-system.md`
  - `docs/src/contributors/harness-internals.md`
  - `docs/src/contributors/verification.md`
- Navigation/docs structure if needed:
  - `docs/src/SUMMARY.md`
  - `docs/src/start-here/docs-map.md`
- Task bookkeeping:
  - this task file
  - relevant `.ralph` state files updated by normal Ralph workflow

### 6. Exact execution order for the later `NOW EXECUTE` pass

- First, re-open this task file, confirm it says `NOW EXECUTE`, and follow this sequence without broad fresh exploration.
- Re-audit the current contributor docs and record the concrete mismatch list inside this task file.
- Make and record the chapter-split decision before substantive prose rewrites; if consolidation wins, update the expected file list in this task file at the same time.
- Spawn the planned parallel subagents on the three contributor-doc slices after the main agent has enough context to assign bounded ownership.
- Rewrite or replace the contributor pages in place, keeping code references and cross-links aligned with the current implementation.
- Re-read the contributor section in navigation order and remove stale links, duplicate explanations, and weak or generic prose.
- Update `docs/src/SUMMARY.md` and any adjacent navigation pages if chapter names/order changed.
- Tick off acceptance-criteria boxes in this task file only after the rewritten docs genuinely satisfy them.

### 6a. Execution audit evidence recorded during `NOW EXECUTE`

- Concrete mismatches found in the current contributor docs before rewrite:
  - The contributor flow currently ends with two separate maintenance-only chapters (`verification.md` and `docs-style.md`). That split dilutes the code-navigation guide and makes the section feel like process overhead instead of one coherent “how to keep contributor docs true to code” page.
  - `docs/src/contributors/ha-pipeline.md` describes deterministic effect dispatch, but the current code dispatches buckets in this exact order inside `src/ha/apply.rs`: Postgres, lease, switchover, replication, then safety. The rewritten chapter must name the real order because it explains why demote/start effects can happen before lease or replication writes on the same tick.
  - `docs/src/contributors/ha-pipeline.md` should explicitly document the current redundant-dispatch suppression rule in `src/ha/worker.rs::should_skip_redundant_process_dispatch(...)`: only repeated `WaitForPostgres(start_requested=true)`, `RecoverReplica`, and `FenceNode` decisions suppress duplicate process dispatches across identical ticks. The existing prose does not make that boundary obvious.
  - `docs/src/contributors/api-debug-contracts.md` underplays that the API worker is a hand-rolled timed accept/request loop in `src/api/worker.rs`, not a framework router. That matters for contributors changing timeouts, TLS behavior, auth, or per-request fault handling.
  - `docs/src/contributors/api-debug-contracts.md` should explicitly document that `/ha/state` is derived from the composed debug snapshot via `api/controller.rs::get_ha_state(...)`, while `/debug/verbose` comes from `debug_api/view.rs::build_verbose_payload(...)`. The current page says this broadly but does not point readers to the precise projection code they must edit.
  - `docs/src/contributors/api-debug-contracts.md` should note the current endpoint-role split precisely: `POST /switchover`, `DELETE /ha/switchover`, and `POST /fallback/heartbeat` are admin routes; `GET /ha/state`, `GET /fallback/cluster`, and `GET /debug/*` are read routes, with `401` vs `403` decided in `authorize_request(...)`.
  - `docs/src/contributors/worker-wiring.md` and adjacent pages should mention that `run_workers(...)` creates three separate `EtcdDcsStore` handles for DCS watch/publication, HA writes, and API intent writes. The code intentionally avoids one shared mutable store handle across all workers.
  - `docs/src/contributors/testing-system.md` references `src/worker_contract_tests.rs`, which is real, but the contributor flow should point readers more directly at the real-binary HA scenario entrypoints (`tests/ha_multi_node_failover.rs`, `tests/ha_partition_recovery.rs`) and the harness startup path in `src/test_harness/ha_e2e/startup.rs`.

- Chapter-split decision recorded before rewrite:
  - Keep `testing-system.md` plus nested `harness-internals.md`; that split still matches two distinct contributor jobs (choosing a test layer vs debugging real-binary fixture behavior).
  - Consolidate `verification.md` and `docs-style.md` into a single contributor docs-maintenance chapter because both pages are about “how to keep docs truthful and useful” rather than about navigating the runtime itself.
  - Carry the merged chapter in `docs/src/contributors/verification.md` and update contributor navigation to end with one short maintenance chapter instead of two weakly separated ones.

#### Chapter-split decision recorded during execution

- Keep the core technical spine as separate chapters because contributors need distinct entry points for ownership, runtime wiring, HA decisions, API/debug contracts, and test strategy.
- Keep `testing-system.md` and `harness-internals.md` as separate chapters, but make both first-class contributor navigation targets instead of treating harness internals as a subordinate appendix.
- Consolidate `verification.md` and `docs-style.md` into a single contributor-maintenance chapter carried by `docs/src/contributors/verification.md`, then remove `docs-style.md` and update `SUMMARY.md` accordingly.

### 7. Planned verification and closeout order

- Run the required gates in full:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- If a gate fails, fix the issue and re-run the affected gate until all four are green.
- Only after every required gate passes:
  - set `<passes>true</passes>` in this task file
  - run `/bin/bash .ralph/task_switch.sh`
  - commit all modified files, including `.ralph` bookkeeping files, with `task finished [task name]: ...`
  - `git push`
- Stop immediately after the required push.

### 8. Required skeptical review targets for the `TO BE VERIFIED` pass

- Challenge whether keeping the current contributor page split is actually the best structure or whether a different split would better match contributor jobs.
- Challenge whether the current plan over-preserves existing prose that should instead be replaced.
- Challenge whether the source-file audit set is sufficient to ground every major contract claimed in the rewritten docs.
- The `TO BE VERIFIED` pass must alter at least one concrete part of this plan before replacing the marker with `NOW EXECUTE`.

NOW EXECUTE
