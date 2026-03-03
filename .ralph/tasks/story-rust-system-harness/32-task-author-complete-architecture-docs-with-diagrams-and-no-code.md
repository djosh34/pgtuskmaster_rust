---
## Task: Author full architecture documentation with rich diagrams and zero code-level narration <status>completed</status> <passes>true</passes> <passing>true</passing>

<blocked_by>31-task-docs-framework-selection-install-and-artifact-hygiene</blocked_by>

<description>
**Goal:** Create complete, human-flowing architecture documentation for the full system using the chosen framework, with diagram-first explanations and no implementation-level code discussion.

**Scope:**
- Produce documentation for first-time readers who have never seen the repository.
- Cover top-level architecture, subsystem responsibilities, runtime/control/data flow, failure/recovery model, deployment/testing mental model, and operational interaction surfaces.
- Explain behavior in component-interaction terms (for example: “component X reacts when component Y emits/changes Z”), not function signatures or code argument details.
- Include substantial diagrams throughout (for example Mermaid/PlantUML/embedded SVG) to clarify:
  - system context
  - container/deployment topology
  - control loops/state transitions
  - request/response and event flow
  - failover and recovery paths
- Keep writing natural, editorial, and coherent (HashiCorp-docs-level readability and structure), while staying technically grounded in this codebase.
- Explicitly forbid code dumps and low-level API/argument walkthrough prose in the architecture docs.
- Please don't hesitate to COMPLETELY alter the current ToC. If you don't change any port of the current structure, that means you didn't think this through enough.
- Think proper GREAT rustbook, with subpages and subsubpages, overviews, suboverviews, diagrams, nicely flowing text, natural text, etc etc

**Context from research:**
- User requested “VitePress-level beauty” and highly readable docs with heavy diagram support.
- User requested architecture-oriented docs only: no nitty-gritty code details, no signature-driven narrative, and strong newcomer orientation.

**Expected outcome:**
- A polished, navigable docs set that lets a new engineer understand how the system works end-to-end at architecture level, with consistent diagrams and high-quality prose.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete file/module requirements: docs IA/navigation pages added, architecture overview page added, subsystem pages added, runtime and failover behavior pages added, operational/testing mental model pages added, glossary/concepts page added
- [x] Each major section includes at least one meaningful diagram and diagrams are consistent with actual component boundaries and behavior
- [x] Writing quality bar met: natural flow, minimal jargon overload, reader-first explanations, and explicitly architecture-focused content with no function-signature or argument-level narrative
- [x] “No code in architecture docs” rule enforced: no code blocks except optional tiny config/CLI examples if absolutely required for orientation, and those must not dominate content
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2, verified)

Goal reminder:
- Architecture docs must read like a newcomer-friendly “system book”: diagram-first, component-interaction language, and strictly avoid implementation narration (no function-by-function walkthroughs, no signature/argument discussion).
- Diagrams are required throughout (Mermaid/PlantUML/embedded SVG), but they must reflect *actual* component boundaries/behavior in this repo.

### Phase 0 — Preflight + evidence (make it reproducible and reviewable)
- [x] Create evidence folder: `.ralph/evidence/task-32-architecture-docs/`
- [x] Record baseline repo state:
  - [x] `git status --porcelain > .ralph/evidence/task-32-architecture-docs/baseline-status.txt`
  - [x] `git diff --name-only > .ralph/evidence/task-32-architecture-docs/baseline-changed-files.txt`
  - [x] `git rev-parse HEAD > .ralph/evidence/task-32-architecture-docs/baseline-head.txt`
- [x] Confirm docs framework from task 31 is present:
  - [x] `test -x .tools/mdbook/bin/mdbook || ./tools/install-mdbook.sh`
- [x] Confirm generated docs outputs are not tracked:
  - [x] `make docs-hygiene |& tee .ralph/evidence/task-32-architecture-docs/docs-hygiene-pre.txt`

### Phase 1 — Architecture source-of-truth research (no writing yet)
Objective: build an accurate “component vocabulary” and interaction map before authoring prose/diagrams.
- [x] Identify the real deployable artifacts and roles:
  - [x] Node binary: `pgtuskmaster` (`src/bin/pgtuskmaster.rs`)
  - [x] CLI binary: `pgtuskmasterctl` (`src/bin/pgtuskmasterctl.rs`)
- [x] Build a component list (names used consistently across docs):
  - [x] Node Runtime / Supervisor (runs a startup planner, then a steady-state worker set)
  - [x] Startup Planner/Executor: `InitializePrimary` / `CloneReplica` / `ResumeExisting`
  - [x] Workers (steady-state): `pginfo`, `dcs`, `process`, `ha`, `api`, `debug_api`
  - [x] External systems: PostgreSQL, etcd (DCS), clients/operators/automation
- [x] Verify DCS keyspace and control concepts directly from code/tests (for diagram correctness):
  - [x] Member records path pattern: `/<scope>/member/<id>`
  - [x] Leader path: `/<scope>/leader`
  - [x] Switchover intent path: `/<scope>/switchover`
  - [x] Runtime config path: `/<scope>/config`
  - [x] Init lock path: `/<scope>/init`
  - [x] Trust model terms (use the code names, not generic prose): `FullQuorum`, `FailSafe`, `NotTrusted`
- [x] Verify HA “big phases” and their triggers from HA module and e2e tests:
  - [x] Canonical HA lifecycle phases (document these explicitly): `Init`, `WaitingPostgresReachable`, `WaitingDcsTrusted`, `Replica`, `CandidateLeader`, `Primary`, `Rewinding`, `Bootstrapping`, `Fencing`, `FailSafe`
  - [x] Split-brain protection behavior when conflicting leader records exist (fencing) vs “leader is unavailable” (follow/promotion decisions)
  - [x] Switchover control flow (API/CLI -> DCS intent -> HA action -> DCS clears)
- [x] Capture a single “architecture notes” scratchpad (not published in docs) in evidence folder:
  - [x] `.ralph/evidence/task-32-architecture-docs/architecture-notes.md`

### Phase 2 — Pick & wire the diagram standard (Mermaid-first, mandatory infrastructure)
Need: “rich diagrams throughout” that are easy to maintain.

Decision:
- Prefer Mermaid for all flow/sequence/state diagrams.
- Use embedded SVG only for cases Mermaid cannot express cleanly (rare).

Implementation plan:
- [x] Add a pinned installer for Mermaid support (preprocessor):
  - [x] Create `tools/install-mdbook-mermaid.sh` (pinned `mdbook-mermaid` version, installed via `cargo install --locked`, output placed in `.tools/mdbook/bin/` as `mdbook-mermaid`)
  - [x] Capture `mdbook-mermaid --version` output to `.ralph/evidence/task-32-architecture-docs/install-mdbook-mermaid.log`
  - [x] Update `docs/book.toml` to enable Mermaid preprocessor (`[preprocessor.mermaid]`), so ` ```mermaid` blocks render
  - [x] Update `Makefile` `docs-build`/`docs-serve` to ensure `.tools/mdbook/bin` is on `PATH` so mdBook can find `mdbook-mermaid`
- [ ] Add a short “Docs style + constraints” page:
- [x] Add a short “Docs style + constraints” page:
  - [x] Defines the architecture-writing rules (no code narration, no signature walkthroughs)
  - [x] Defines what fenced blocks are allowed:
    - [x] Allowed: `mermaid`, and tiny `bash`/`console`/`toml` for orientation only
    - [x] Disallowed: `rust` (and other programming-language fences) in architecture pages
  - [x] Include a “diagram conventions” section: naming, actors, arrows, error paths, and legend

### Phase 3 — Completely redesign the IA (book structure + navigation)
Requirement: do not keep the current flat ToC. Build a real “book” structure with overviews and subpages.

Proposed IA (directories + pages under `docs/src/`):
- [x] `introduction.md` (rewrite as newcomer start-here, with “what problem this solves”)
- [x] `reading-guide.md` (15-minute + 60-minute reading paths; “where to jump based on role”)
- [x] `concepts/`
  - [x] `concepts/index.md` (mental model, vocabulary)
  - [x] `concepts/mental-model.md` (diagram-first: concepts map and what each worker “owns”)
  - [x] `concepts/glossary.md` (expanded, cross-linked)
  - [x] `concepts/roles-and-trust.md` (roles + `DcsTrust` states + what “FailSafe” means here)
- [x] `architecture/`
  - [x] `architecture/index.md` (one-page tour + links)
  - [x] `architecture/system-context.md` (system context diagram)
  - [x] `architecture/deployment-topology.md` (node + etcd + PostgreSQL topology)
  - [x] `architecture/node-runtime.md` (inside one node: workers + channels)
  - [x] `architecture/control-loop.md` (the steady-state reconciliation loop)
  - [x] `architecture/startup-planner.md` (startup decision flow: `InitializePrimary` / `CloneReplica` / `ResumeExisting`)
  - [x] `architecture/ha-lifecycle.md` (HA phases/state diagrams; steady-state vs recovery)
  - [x] `architecture/dcs-keyspace.md` (DCS record model, ownership, watch/cache; include `/config` + `/init`)
  - [x] `architecture/failover-and-recovery.md` (failover paths, recovery/rewind/bootstrap)
  - [x] `architecture/switchover.md` (operator-driven switchover sequence)
  - [x] `architecture/safety-and-fencing.md` (split-brain prevention, fencing model; diagram “leader unavailable” vs “conflicting leader => fence”)
- [x] `interfaces/`
  - [x] `interfaces/index.md` (interaction surfaces overview)
  - [x] `interfaces/node-api.md` (operator/API mental model; focus on `GET /ha/state`, `POST /switchover`, `DELETE /ha/switchover`)
  - [x] `interfaces/cli.md` (CLI mental model + common workflows; tiny examples only)
- [x] `operations/`
  - [x] `operations/index.md` (operational mental model)
  - [x] `operations/docs.md` (existing docs build/serve instructions, updated if tooling changes)
  - [x] `operations/deployment.md` (systemd/container assumptions, ports, directories, prerequisites)
  - [x] `operations/observability.md` (debug API, logs, “what to look at first”)
- [x] `testing/`
  - [x] `testing/index.md` (testing mental model)
  - [x] `testing/harness.md` (test harness architecture: real binaries, etcd/postgres installers)
  - [x] `testing/bdd.md` (BDD intent: what it validates at architecture level)

Navigation implementation:
- [x] Rewrite `docs/src/SUMMARY.md` to match this IA (ordered for first-time readers).
- [x] Remove/retire old placeholder pages (`architecture.md`, `components.md`, `operations.md`, `glossary.md`) by either:
  - [x] Converting them into index pages that redirect readers to the new structure, or
  - [x] Keeping them as short “moved to …” pages temporarily to avoid broken links during transition.

### Phase 4 — Author the actual architecture content (diagram-first)
Rule: each major page has at least one meaningful diagram.

Diagram checklist (minimum set):
- [x] System context diagram (actors: operator/automation, clients, node, postgres, etcd)
- [x] Deployment/topology diagram (single node + multi-node cluster)
- [x] Node internals diagram (workers + dependencies + data flow arrows)
- [x] Control-loop sequence diagram (one “tick” end-to-end)
- [x] Startup decision flowchart (startup planner: `InitializePrimary` / `CloneReplica` / `ResumeExisting`)
- [x] HA phase/state diagrams:
  - [x] Steady-state roles (Replica/CandidateLeader/Primary)
  - [x] Recovery/fencing path (Rewinding/Bootstrapping/Fencing/FailSafe)
- [x] Switchover sequence diagram (request -> DCS intent -> demotion/promotion)
- [x] Failure/recovery diagrams:
  - [x] Leader loss -> election/failover path
  - [x] Split-brain detection -> fencing/demotion safety path
- [x] DCS keyspace map diagram (paths + producer/consumer ownership; include `/config` + `/init`)
- [x] Operator interface diagram (API/CLI -> node -> DCS; emphasize that `/ha/leader` is not an admin surface)

Writing checklist (enforce architecture-level tone):
- [x] Use “component X reacts when component Y changes Z” phrasing.
- [x] Never narrate function calls, argument lists, or internal structs in prose.
- [x] Keep each page readable standalone (repeat minimal context + link to deeper pages).
- [x] Add “If you only remember 3 things…” summaries at the end of major chapters.

### Phase 5 — Enforce “no code in architecture docs” (lightweight, automated)
The policy must be enforceable, not just aspirational.
- [x] Add a small script `tools/docs-architecture-no-code-guard.sh`:
  - [x] Scopes to architecture-oriented docs only:
    - [x] `docs/src/architecture/**/*.md`
    - [x] `docs/src/concepts/**/*.md`
    - [x] `docs/src/interfaces/**/*.md`
    - [x] `docs/src/testing/**/*.md`
  - [x] Fails if any scoped page contains:
    - [x] Any fenced block with a disallowed language (explicitly include `rust`/`rs` in the deny list)
    - [x] Any unlabeled fenced block (prevents silent code dumps)
  - [x] Allows `mermaid` and tiny `bash`/`console`/`toml`/`text` fences
  - [x] Emits clear file + line diagnostics
- [x] Add `make docs-lint` target that runs the guard script.
- [x] Wire `docs-lint` into `make lint` (so required gates always enforce the policy), while keeping scope narrow to avoid false positives.

### Phase 6 — Validate docs render + all required quality gates (no skips)
Docs validation (separate from Rust gates):
- [x] `set -o pipefail; make docs-build |& tee .ralph/evidence/task-32-architecture-docs/make-docs-build.log`
- [x] `make docs-hygiene |& tee .ralph/evidence/task-32-architecture-docs/make-docs-hygiene.log`
- [x] `make docs-lint |& tee .ralph/evidence/task-32-architecture-docs/make-docs-lint.log`
- [ ] If Mermaid tooling added: `make docs-serve` smoke test + `curl` proof (optional but recommended)

Required gates (must be 100% green before marking `<passes>true</passes>`):
- [x] `cargo clean` once (reduces mount/link flakes)
- [x] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make check |& tee .ralph/evidence/task-32-architecture-docs/make-check.log`
- [x] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test |& tee .ralph/evidence/task-32-architecture-docs/make-test.log`
- [x] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test |& tee .ralph/evidence/task-32-architecture-docs/make-test.log`
- [x] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make lint |& tee .ralph/evidence/task-32-architecture-docs/make-lint.log`
- [x] Supplemental evidence (do not rely on grep as the source of truth; exit status is):
  - [x] `rg -n \"congratulations|evaluation failed\" .ralph/evidence/task-32-architecture-docs/make-test.log > .ralph/evidence/task-32-architecture-docs/make-test-markers.txt || true`
  - [x] `rg -n \"congratulations|evaluation failed\" .ralph/evidence/task-32-architecture-docs/make-lint.log > .ralph/evidence/task-32-architecture-docs/make-lint-markers.txt || true`

### Phase 7 — Finish task (only after everything is green)
- [x] Update this task file:
  - [x] Tick all acceptance checkboxes
  - [x] Set `<status>completed</status>`, `<passes>true</passes>`, and `<passing>true</passing>`
  - [x] Add a short evidence pointer to `.ralph/evidence/task-32-architecture-docs/`
- Evidence: `.ralph/evidence/task-32-architecture-docs/`
- [x] Run `/bin/bash .ralph/task_switch.sh`
- [x] `git add -A` (including `.ralph` updates, excluding generated docs output)
- [x] Commit:
  - [x] `task finished 32-task-author-complete-architecture-docs-with-diagrams-and-no-code: <summary + evidence>`
- [x] `git push`
- [x] Append learnings/surprises to `AGENTS.md`

NOW EXECUTE
</execution_plan>
