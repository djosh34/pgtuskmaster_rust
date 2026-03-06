## Task: Post-Rewrite Skeptical Claim Verification with 15+ Parallel Spark Subagents <status>done</status> <passes>true</passes>

<description>
**Goal:** After the operator-doc transformation is complete, run a deep, adversarial verification of every claim in the docs using many independent `spark` subagents, and resolve all mismatches before finalizing docs.

**Scope:**
- This task MUST start only after Task 01 is complete and docs structure/content are stabilized.
- Build a comprehensive claim inventory from the rewritten docs across the new structure:
  - Start Here
  - Quick Start
  - Operator Guide
  - System Lifecycle
  - Architecture Assurance
  - Interfaces
  - (Contributor sections only for implementation/process claims that are still normative)
  - extract all non-trivial claims (behavior, safety guarantees, endpoint semantics, config effects, DCS write ownership, failure behavior, startup/HA transitions, safety-case assumptions)
  - assign each claim a unique claim ID and exact location (`path:line`)
  - classify each claim type (descriptive, behavioral, invariant, absence/negative, operational expectation).
- Create a post-rewrite verification matrix (generated after rewrite, not before):
  - one row per claim with expected evidence type and verification method
  - include strict pass/fail criteria and required evidence anchors.
- Execute verification using 15+ parallel `spark` subagents with independent ownership slices.
- Keep all verification process details out of operator docs:
  - verification artifacts belong in internal task evidence and/or contributor-only verification records
  - operator-facing docs should contain final accurate content only.

**Context from research:**
- High-trust documentation requires evidence-backed claim validation, especially for architecture and operational behavior.
- Independent parallel verification reduces shared blind spots and confirmation bias.
- Negative claims and safety claims require stronger evidence standards than descriptive claims.

**Expected outcome:**
- Every operator-facing claim is either proven with code/test/runtime evidence, rewritten to bounded language, or removed.
- DCS key ownership and write-path claims are explicitly verified against implementation entry points.
- Verification is adversarial, reproducible, and explicitly skeptical.
- Final docs are accurate without exposing internal verification mechanics in the operator reading flow.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x]  Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Task execution starts only after Task 01 completion and rewritten docs freeze point is recorded
- [x] A full post-rewrite claim inventory exists with claim IDs and exact `path:line` anchors
- [x] A verification matrix is generated after rewrite (not before), with per-claim evidence requirements and pass/fail rules
- [x] At least 15 `spark` subagents run in parallel, each with a disjoint claim slice and explicit ownership
- [x] Every subagent receives precise instructions to be maximally skeptical: assume docs/comments can be wrong, trust only code/tests/runtime evidence
- [x] Every subagent instruction includes scoped verification bullets covering:
  - exact claims to verify
  - specific section coverage in the new IA (Start Here, Quick Start, Operator, Lifecycle, Assurance, Interfaces)
  - required code paths/symbols/tests/runtime checks
  - forbidden weak evidence (for example, unverified comments or second-hand doc statements)
  - handling for uncertain/ambiguous findings
  - required evidence output format
- [x] Each claim outcome is one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
- [x] Absence/negative claims are accepted only with explicit guards/tests; otherwise rewritten to bounded wording or removed
- [x] Conflicting subagent conclusions are adjudicated and resolved with final evidence-backed disposition
- [x] Verification artifacts remain outside operator docs; operator docs show only corrected final content
- [x] `make docs-lint` passes cleanly after all rewrites
- [x] `make docs-build` passes cleanly after all rewrites
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

This task is *post-rewrite* and MUST be executed skeptically: assume docs can be wrong, comments can be wrong, and that “sounds-right” behavior is wrong until proven by code/tests/runtime evidence.

### Preconditions / Freeze Point

- [x] Confirm Task 01 is complete (it is currently marked `<status>done</status> <passes>true</passes>` in `.ralph/tasks/story-operator-architecture-docs/01-task-restructure-operator-docs-for-flow-depth-and-rationale.md`).
- [x] Record the docs freeze point (this is the anchor for all `path:line` claim IDs):
  - [x] Record `git rev-parse HEAD` and `git status --porcelain` into evidence.
  - [x] Record the exact `docs/src/` file list into evidence (so claim coverage is measurable).
  - [x] Record the `SUMMARY`-reachable docs set separately (`summary-reachable-files.txt`) and treat it as the default operator-facing claim surface.
  - [x] Build a fail-closed scope map (`scope-map.csv`) for all markdown under `docs/src/` with status `{reachable, internal-only, orphan}`; no file may be silently ignored.
  - [x] Orphan docs triage (greenfield; no backwards-compat required):
    - [x] For any `orphan` file, decide `{remove, migrate-into-IA, mark-internal-only}` and record rationale in evidence (`orphan-docs-triage.md`).
    - [x] Prefer removal over “keep around” unless there is an explicit, current owner and purpose.
  - [x] **Rule:** the initial claim inventory uses `path:line` as of this freeze commit; if later rewrites move text, update the claim inventory anchor and keep the old anchor in an `original_anchor` column (so the audit trail stays consistent).

### Exhaustive Checklist (Files / Modules / Requirements)

#### Operator-facing docs: claim scan + correction surface (MAY be modified)

Every file below is in-scope for claim extraction. Operator-facing docs MUST NOT include internal verification mechanics (claim IDs, evidence tables, or “this was verified by…” process notes). They MAY be rewritten to become accurate, bounded, and evidence-consistent.

- [x] `docs/src/introduction.md`
- [x] `docs/src/start-here/problem.md`
- [x] `docs/src/start-here/solution.md`
- [x] `docs/src/start-here/docs-map.md`
- [x] `docs/src/quick-start/index.md`
- [x] `docs/src/quick-start/prerequisites.md`
- [x] `docs/src/quick-start/first-run.md`
- [x] `docs/src/quick-start/initial-validation.md`
- [x] `docs/src/operator/index.md`
- [x] `docs/src/operator/configuration.md`
- [x] `docs/src/operator/deployment.md`
- [x] `docs/src/operator/observability.md`
- [x] `docs/src/operator/troubleshooting.md`
- [x] `docs/src/lifecycle/index.md`
- [x] `docs/src/lifecycle/bootstrap.md`
- [x] `docs/src/lifecycle/steady-state.md`
- [x] `docs/src/lifecycle/switchover.md`
- [x] `docs/src/lifecycle/failover.md`
- [x] `docs/src/lifecycle/failsafe-fencing.md`
- [x] `docs/src/lifecycle/recovery.md`
- [x] `docs/src/assurance/index.md`
- [x] `docs/src/assurance/safety-invariants.md`
- [x] `docs/src/assurance/decision-model.md`
- [x] `docs/src/assurance/dcs-data-model.md`
- [x] `docs/src/assurance/runtime-topology.md`
- [x] `docs/src/assurance/safety-case.md`
- [x] `docs/src/assurance/tradeoffs-limits.md`
- [x] `docs/src/interfaces/index.md`
- [x] `docs/src/interfaces/node-api.md`
- [x] `docs/src/interfaces/cli.md`
- [x] `docs/src/concepts/glossary.md` (only claims that are *actually claims*; most content is definitional)

#### Contributor-only docs: may be referenced for process, but keep operator flow clean (MAY be modified)

These may be updated only if we need to fix implementation/process claims (or to add a pointer to where the evidence lives), but they must not leak into operator flow.

- [x] `docs/src/contributors/index.md`
- [x] `docs/src/contributors/codebase-map.md`
- [x] `docs/src/contributors/worker-wiring.md`
- [x] `docs/src/contributors/ha-pipeline.md`
- [x] `docs/src/contributors/testing-system.md`
- [x] `docs/src/contributors/harness-internals.md`
- [x] `docs/src/contributors/docs-style.md`
- [x] `docs/src/contributors/verification.md`
- [x] `docs/src/contributors/task-33-docs-verification-report.md` (currently a pointer/appendix)
- [x] `docs/src/verification/index.md` (NOT linked from SUMMARY today; treat as internal ledger area)
- [x] `docs/src/verification/task-33-docs-verification-report.md` (historical; use as template only)

#### Evidence artifacts (MUST be created; NOT in operator docs)

Create a task evidence directory, committed in-repo:

- [x] `.ralph/evidence/story-operator-architecture-docs/02-task-post-rewrite-skeptical-claim-verification-with-spark/`
  - [x] `git-head.txt` (`git rev-parse HEAD`)
  - [x] `git-status.txt` (`git status --porcelain`)
  - [x] `docs-src-files.txt` (`git ls-files -- docs/src`)
  - [x] `summary-reachable-files.txt` (derived from `docs/src/SUMMARY.md`)
  - [x] `scope-map.csv` (`path,scope_status,reason`)
  - [x] `claim-candidates.txt` (raw high-risk-token hits)
  - [x] `claim-inventory.csv` (authoritative claim list; one row per claim)
  - [x] `verification-matrix.csv` (claim → evidence method + pass/fail + outcome)
  - [x] `claim-coverage-check.txt` (machine check proving every in-scope file and every claim ID is covered exactly once)
  - [x] `subagents/` (one file per agent with assigned claim IDs + output + evidence anchors)
  - [x] `adjudication.md` (conflict resolution log + final dispositions)
  - [x] `build-warmup.log`, `make-docs-lint.log`, `make-docs-build.log`, `make-check.log`, `make-test.log`, `make-lint.log`, `make-test-long.log` (final gate evidence)

#### Code / tests: authoritative evidence sources (MAY be modified if docs reveal missing guards/tests)

The verification work MUST cite (or derive) evidence from these modules/tests. If a doc makes a strong claim that isn’t supported, resolve by **(a)** adding a guard/test, **or** **(b)** rewriting/removing the claim. Avoid adding “trust me” claims to docs.

**Node API / Interfaces**
- [x] `src/runtime/node.rs` (API wiring and runtime bootstrap; server binding + worker startup)
- [x] `src/api/worker.rs` (HTTP accept/auth/route dispatch)
- [x] `src/api/controller.rs` (HA endpoints: switchover intent + state)
- [x] `src/api/fallback.rs` (fallback endpoints)
- [x] `src/debug_api/view.rs` (debug payload projection)
- [x] `src/cli/client.rs` (CLI→API contract and auth roles)
- [x] `tests/bdd_api_http.rs` (black-box HTTP contract assertions)
- [x] `tests/policy_e2e_api_only.rs` (post-start hands-off policy guard)
- [x] `tests/cli_binary.rs` (CLI behavior contract in real-ish harness)

**DCS / Keyspace**
- [x] `src/dcs/keys.rs` (key definitions + path parsing)
- [x] `src/dcs/state.rs` (DCS cache + trust evaluation)
- [x] `src/dcs/store.rs` (watch decode/apply; writer helpers)
- [x] `src/dcs/worker.rs` (writes local member; applies watch updates)
- [x] `src/dcs/etcd_store.rs` (real etcd backend; reconnect/reset semantics; txns)
- [x] `src/test_harness/etcd3.rs` (etcd harness)
- [x] `src/test_harness/ha_e2e/startup.rs` (startup harness asserts `/init` and config key behavior)

**HA / Lifecycle**
- [x] `src/ha/state.rs` (phase model; invariant fields)
- [x] `src/ha/decide.rs` (pure decision logic; trust gates; switchover/failover/fencing rules)
- [x] `src/ha/worker.rs` (dispatch mapping to DCS/process jobs; idempotency and sequencing)
- [x] `src/process/worker.rs` (actual `initdb`/`pg_basebackup`/`pg_rewind`/fencing commands + timeouts)
- [x] `src/ha/e2e_multi_node.rs` (real-binary HA scenario matrix + stress + no-quorum fencing)
- [x] `src/ha/e2e_partition_chaos.rs` (network partition scenarios)
- [x] `src/test_harness/ha_e2e/*` (API polling helpers; timeouts; startup wiring)

**Config / Operator surface**
- [x] `src/config/parser.rs` (config validation and defaults; “fail closed” behavior)
- [x] `src/config/schema.rs` (schema fields referenced by docs)
- [x] `examples/*` (if docs claim example correctness; keep them compiling)

### Claim Inventory (Build After Rewrite)

- [x] Generate a raw candidate list (high-risk tokens):
  - tokens include: `always`, `never`, `guarantee`, `ensures`, `must`, `will`, `only`, `cannot`, `impossible`, `prevents`, `safe`, `split brain`, `fence`, `fail-safe`, `quorum`, `DCS`, `etcd`, `lease`, `leader`, `primary`, `replica`, `rewind`, `basebackup`, `initdb`.
  - output to evidence `claim-candidates.txt` as `path:line:content`.
- [x] Convert candidates into an authoritative `claim-inventory.csv`:
  - columns (minimum): `claim_id`, `anchor` (`path:line`), `section`, `claim_type`, `severity`, `claim_text`, `expected_evidence_type`, `verification_method`, `pass_criteria`, `status`, `evidence_anchor`, `notes`, `original_anchor`.
  - claim types: `descriptive`, `behavioral`, `invariant`, `absence/negative`, `operational expectation`.
  - statuses: `unverified`, `verified`, `rewritten`, `removed`, `uncertain-with-followup`.
  - **Rule:** treat “absence/negative” claims as *unsafe by default* unless backed by an explicit guard/test or a mechanically-enforced restriction.
- [x] Add a strict completeness check:
  - for each in-scope doc file, there is an explicit “reviewed” marker (in the inventory or a separate coverage table) so “missed file” is impossible.
  - fail closed on claim IDs: no duplicate `claim_id`, no duplicate `anchor`, and no `unverified` rows allowed before gate runs.

### Verification Matrix (Per-claim Evidence Requirements)

- [x] Generate `verification-matrix.csv` *after* the claim inventory exists:
  - evidence classes: `code symbol`, `unit test`, `BDD black-box`, `real-binary e2e`, `runtime log evidence`, `mechanical tooling guard`.
  - pass/fail rule examples:
    - Node API claim is “verified” only if endpoint exists in routing and is tested in `tests/bdd_api_http.rs` (or equivalent) with correct status codes and payload semantics.
    - DCS key ownership claim is “verified” only if key path, writer, and consumer match code paths (for example `/<scope>/switchover` is written by API and cleared by HA, not by operator docs).
    - HA safety claim is “verified” only if decision logic + dispatch + at least one integration/e2e scenario demonstrates the behavior under fault.
    - “Never/always” language is accepted only with explicit mechanical enforcement or exhaustive tests; otherwise rewrite to bounded language.

### Parallel Skeptical Verification (15+ `spark` subagents)

- [x] Split claim IDs into **at least 15 disjoint slices** (prefer 18–24 for smaller ownership and redundancy):
  - slice by doc sections (Start Here / Quick Start / Operator / Lifecycle / Assurance / Interfaces / Contributors) and then by file.
  - emit `subagents/slice-XX.md` files containing:
    - the assigned claim IDs + anchors + exact claim text
    - the required evidence class for each claim
    - any “high-risk” notes (negative claim, safety claim, split-brain claim)
- [x] Spawn 15+ `spark` agents, each owning exactly one slice, with **hard evidence rules**:
  - forbidden evidence: “docs say so”, “comments imply”, “it seems”, “I think”.
  - required evidence output per claim: `claim_id`, `status`, `evidence_anchor` (file path + symbol + test name), plus a one-sentence rationale.
  - uncertainty handling: if evidence is missing or ambiguous, set `uncertain-with-followup` and propose the minimal correction (rewrite claim vs add test vs fix code).
  - after fan-out, run a machine coverage check proving: every claim appears in exactly one slice, every slice has an owning subagent output file, and agent count is `>= 15`; otherwise fail the task immediately.

### Adjudication and Fixes

- [x] Merge subagent outputs into the verification matrix; resolve conflicts:
  - if two agents disagree, assign a single “arbiter” (can be the main agent) to re-check code/tests and pick the final disposition with cited evidence.
- [x] Apply corrections:
  - rewrite/remove claims in operator docs when not provable.
  - prefer adding guards/tests only when the claim describes *intended* safety behavior and the implementation is missing an enforceable invariant.
  - ensure `docs/src/operator/*` stays free of programming-language code blocks (docs lint enforces this).
- [x] Pay special attention to known tricky areas where docs often drift:
  - API surface vs implementation:
    - HTTP route matching + auth gating live in `src/api/worker.rs`; the controller exposes endpoint handlers (no routing) in `src/api/controller.rs` and `src/api/fallback.rs`.
    - treat `/debug/*` as separate endpoints; verify each individually (`/debug/ui`, `/debug/verbose`, `/debug/snapshot`).
    - treat `POST /fallback/heartbeat` as a known-risk claim slice: pure-function validation exists, but route-level HTTP contract coverage may need to be added if docs claim it is stable/guaranteed.
  - DCS reconnect semantics MUST match code and tests (treat as safety-critical, fail-closed):
    - verify disconnect-time stale-event drop: `invalidate_watch_session` clears pending queue (`clear_watch_events`).
    - verify reconnect-time authoritative reset+snapshot: `bootstrap_snapshot(is_reconnect=true)` injects `WatchOp::Reset` and `replace_watch_events` replaces queued events.
    - verify apply semantics in `refresh_from_etcd_watch`: reset clears cached `members/leader/switchover/init_lock` but **preserves `config`**; docs must not claim “full cache wipe”.
    - require concrete evidence anchors (minimum): `src/dcs/store.rs` reset tests and `src/dcs/etcd_store.rs` reconnect/disconnect tests.
  - Fail-safe / fencing wording must not over-claim immediacy or absolutes; confirm the real e2e tolerance behaviors (for example no-quorum fencing allows a bounded number of post-cutoff commits in stress proofs).

### Gates (No Skips)

- [x] Warm compile to avoid `make test` timeout false negatives: `cargo test --all-targets --no-run` (save to `build-warmup.log`)
- [x] `make docs-lint`
- [x] `make docs-build`
- [x] `make check`
- [x] `make test`
- [x] `make lint`
- [x] `make test-long`
- [x] Save *full* logs into evidence directory and ensure they represent the final (post-fix) state.

DONE (all gates pass; evidence committed under `.ralph/evidence/story-operator-architecture-docs/02-task-post-rewrite-skeptical-claim-verification-with-spark/`)
